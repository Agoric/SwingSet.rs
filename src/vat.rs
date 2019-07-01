use super::kernel::{KernelData, PendingDelivery};
use super::kernel_types::{
    KernelArgSlot, KernelCapData, KernelExport, KernelExportID, KernelMessage,
    KernelPromiseResolverID, KernelTarget, VatID,
};
use super::promise::KernelPromise;
use super::syscall::Syscall;
use super::vat_types::{
    OutboundVatMessage, VatArgSlot, VatCapData, VatPromiseID, VatResolveTarget,
    VatResolverID, VatSendTarget,
};
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

enum TargetCategory {
    Export(KernelExport),                    // queue message to an Export
    Promise(VatID, KernelPromiseResolverID), // queue message to exported promise (pipelining)
    ToDataError,                             // error because you cannot send to data
    // TODO might be helpful to summarize the data
    Rejected(KernelCapData), // error: rejected-promise contagion
}

pub(crate) struct VatSyscall {
    vat_id: VatID,
    kd: Rc<RefCell<KernelData>>,
}

impl VatSyscall {
    pub fn new(vat_id: VatID, kd: Rc<RefCell<KernelData>>) -> Self {
        VatSyscall { vat_id, kd }
    }

    fn map_outbound_target(&self, vtarget: VatSendTarget) -> KernelTarget {
        let kd = self.kd.borrow_mut();
        let vd = kd.vat_data.get(&self.vat_id).unwrap();
        match vtarget {
            VatSendTarget::Import(viid) => {
                let ke = vd.import_clist.map_outbound(viid);
                KernelTarget::Export(ke)
            }
            VatSendTarget::Promise(vpid) => {
                let kpid = vd.promise_clist.map_outbound(vpid);
                KernelTarget::Promise(kpid)
            }
        }
    }

    fn classify_target(&self, ktarget: KernelTarget) -> TargetCategory {
        use TargetCategory::*;
        match ktarget {
            KernelTarget::Export(ke) => Export(ke),
            KernelTarget::Promise(kprid) => {
                let kd = self.kd.borrow_mut();
                let kp = kd.promises.get(&kprid).unwrap();
                use KernelPromise::*;
                match kp {
                    Unresolved { decider, .. } => Promise(*decider, kprid),
                    FulfilledToTarget(ke) => Export(*ke),
                    FulfilledToData(_) => ToDataError,
                    KernelPromise::Rejected(d) => TargetCategory::Rejected(d.clone()),
                }
            }
        }
    }

    fn allocate_promise(
        &self,
        p: KernelPromise,
    ) -> (VatPromiseID, KernelPromiseResolverID) {
        let mut kd = self.kd.borrow_mut();
        let id = kd.next_promise_resolver_id;
        kd.next_promise_resolver_id = id + 1;
        let kprid = KernelPromiseResolverID(id);
        kd.promises.insert(kprid, p);
        let vd = kd.vat_data.get_mut(&self.vat_id).unwrap();
        let vpid = vd.promise_clist.map_inbound(kprid);
        (vpid, kprid)
    }

    fn allocate_result_promise(
        &self,
        sender: VatID,
        receiver: VatID,
    ) -> (VatPromiseID, KernelPromiseResolverID) {
        let mut subscribers = HashSet::new();
        subscribers.insert(sender);
        let p = KernelPromise::Unresolved {
            subscribers,
            decider: receiver,
        };
        self.allocate_promise(p)
    }

    fn send_data_error_promise(&self) -> (VatPromiseID, KernelPromiseResolverID) {
        let d = KernelCapData {
            body: b"cannot send to data".to_vec(),
            slots: vec![],
        };
        let p = KernelPromise::Rejected(d);
        self.allocate_promise(p)
    }

    fn send_rejected_error_promise(
        &self,
        data: &KernelCapData,
    ) -> (VatPromiseID, KernelPromiseResolverID) {
        let p = KernelPromise::Rejected(data.clone());
        self.allocate_promise(p)
    }

    fn map_outbound_arg_slot(&self, varg: VatArgSlot) -> KernelArgSlot {
        let kd = self.kd.borrow_mut();
        let vd = kd.vat_data.get(&self.vat_id).unwrap();
        match varg {
            VatArgSlot::Import(viid) => {
                let ke = vd.import_clist.map_outbound(viid);
                KernelArgSlot::Export(ke)
            }
            VatArgSlot::Export(veid) => {
                let keid = KernelExportID(veid.0);
                KernelArgSlot::Export(KernelExport(self.vat_id, keid))
            }
            VatArgSlot::Promise(vpid) => {
                let kpid = vd.promise_clist.map_outbound(vpid);
                KernelArgSlot::Promise(kpid)
            }
        }
    }

    fn map_outbound_capdata(&self, vdata: VatCapData) -> KernelCapData {
        KernelCapData {
            body: vdata.body,
            slots: vdata
                .slots
                .into_iter()
                .map(|slot| self.map_outbound_arg_slot(slot))
                .collect(),
        }
    }

    fn map_outbound_message(
        &self,
        vmsg: OutboundVatMessage,
        okprid: Option<KernelPromiseResolverID>,
    ) -> KernelMessage {
        KernelMessage {
            name: vmsg.name.to_string(),
            args: self.map_outbound_capdata(vmsg.args),
            resolver: okprid,
        }
    }

    fn do_send(
        &mut self,
        vtarget: VatSendTarget,
        vmsg: OutboundVatMessage,
        send_only: bool,
    ) -> Option<VatPromiseID> {
        println!("syscall.send {}.{}", vtarget, vmsg.name);

        // convert and classify the target
        let ktarget = self.map_outbound_target(vtarget);
        let tc: TargetCategory = self.classify_target(ktarget);
        use TargetCategory::*;

        // Now construct the return promise, if any. The state of the promise
        // depends upon the category of target: sending to an Export creates
        // an unresolved promise with the "decider" set to the target vat, as
        // does pipelining to a promise with some decider vat of its own.
        // Error cases create a rejected promise.

        let (ovpid, okprid) = if send_only {
            (None, None)
        } else {
            let (vpid, kprid) = match tc {
                Export(ke) => self.allocate_result_promise(self.vat_id, ke.0),
                Promise(decider, _) => self.allocate_result_promise(self.vat_id, decider),
                ToDataError => self.send_data_error_promise(),
                Rejected(ref d) => self.send_rejected_error_promise(d),
            };
            (Some(vpid), Some(kprid))
        };

        // now that we have the result promise, build the KernelMessage
        // around it, if necessary, and push it onto the run queue

        use PendingDelivery::*;
        match tc {
            Export(ke) => {
                let kmsg = self.map_outbound_message(vmsg, okprid);
                let pd = Deliver {
                    target: ke,
                    message: kmsg,
                };
                self.kd.borrow_mut().run_queue.0.push_back(pd);
            }
            Promise(vat_id, kprid) => {
                let kmsg = self.map_outbound_message(vmsg, okprid);
                let pd = DeliverPromise {
                    vat_id,
                    target: kprid,
                    message: kmsg,
                };
                self.kd.borrow_mut().run_queue.0.push_back(pd);
            }
            ToDataError | Rejected(..) => (),
        };

        // and finally return the result promise (or None if send_only)
        ovpid
    }
}

impl Syscall for VatSyscall {
    fn send(&mut self, vtarget: VatSendTarget, vmsg: OutboundVatMessage) -> VatPromiseID {
        let ovpid = self.do_send(vtarget, vmsg, false);
        ovpid.unwrap()
    }

    fn send_only(&mut self, vtarget: VatSendTarget, vmsg: OutboundVatMessage) {
        self.do_send(vtarget, vmsg, true);
    }

    fn allocate_promise_and_resolver(&mut self) -> (VatPromiseID, VatResolverID) {
        let p = KernelPromise::Unresolved {
            subscribers: HashSet::new(),
            decider: self.vat_id,
        };
        let (vpid, kprid) = self.allocate_promise(p);
        let mut kd = self.kd.borrow_mut();
        let vd = kd.vat_data.get_mut(&self.vat_id).unwrap();
        let vrid = vd.resolver_clist.map_inbound(kprid);
        (vpid, vrid)
    }

    fn subscribe(&mut self, vpid: VatPromiseID) {
        let mut kd = self.kd.borrow_mut();
        let vd = kd.vat_data.get_mut(&self.vat_id).unwrap();
        let kprid = vd.promise_clist.map_outbound(vpid);
        let p: &mut KernelPromise = kd.promises.get_mut(&kprid).unwrap();
        use KernelPromise::*;
        match p {
            Unresolved {
                ref mut subscribers,
                ..
            } => subscribers.insert(self.vat_id),
            _ => panic!("not implemented yet"),
            /*FulfilledToTarget(..) => notify1(),
            FulfilledToData(..) => notify2(),
            Rejected(..) => notify3(),*/
        };
    }

    fn fulfill_to_target(&mut self, resolver: VatResolverID, vtarget: VatResolveTarget) {
        use KernelPromise::{FulfilledToTarget, Unresolved};
        use PendingDelivery::NotifyFulfillToTarget;

        let mut kd = self.kd.borrow_mut();
        let vd = kd.vat_data.get_mut(&self.vat_id).unwrap();
        let kprid = vd.map_outbound_resolver(resolver);
        let ktarget = vd.map_outbound_resolve_target(vtarget);

        let notifications: Vec<PendingDelivery> = {
            let p = kd.promises.get(&kprid).unwrap();
            if let Unresolved {
                subscribers,
                decider,
            } = p
            {
                // resolvers are not transferrable
                assert_eq!(*decider, self.vat_id);
                // TODO: HashSet.iter is nondeterministic
                subscribers
                    .iter()
                    .map(|s| NotifyFulfillToTarget {
                        vat_id: *s,
                        target: kprid,
                        result: ktarget,
                    })
                    .collect()
            } else {
                panic!(); // TODO: DuplicateFulfillError
            }
        };
        kd.run_queue.0.extend(notifications);

        let new_promise = FulfilledToTarget(ktarget);
        kd.promises.remove(&kprid);
        kd.promises.insert(kprid, new_promise);
    }

    fn fulfill_to_data(&mut self, resolver: VatResolverID, vdata: VatCapData) {
        use KernelPromise::{FulfilledToData, Unresolved};
        use PendingDelivery::NotifyFulfillToData;
        let kdata = self.map_outbound_capdata(vdata);
        let mut kd = self.kd.borrow_mut();
        let vd = kd.vat_data.get_mut(&self.vat_id).unwrap();
        let kprid = vd.map_outbound_resolver(resolver);

        let notifications: Vec<PendingDelivery> = {
            let p = kd.promises.get(&kprid).unwrap();
            if let Unresolved {
                subscribers,
                decider,
            } = p
            {
                // resolvers are not transferrable
                assert_eq!(*decider, self.vat_id);
                // TODO: HashSet.iter is nondeterministic
                subscribers
                    .iter()
                    .map(|s| NotifyFulfillToData {
                        vat_id: *s,
                        target: kprid,
                        data: kdata.clone(),
                    })
                    .collect()
            } else {
                panic!(); // TODO: DuplicateFulfillError
            }
        };
        kd.run_queue.0.extend(notifications);

        let new_promise = FulfilledToData(kdata.clone());
        kd.promises.remove(&kprid);
        kd.promises.insert(kprid, new_promise);
    }

    fn reject(&mut self, _resolver: VatResolverID, _data: VatCapData) {
        panic!();
    }
    fn forward(&mut self, _resolver: VatResolverID, _target: VatPromiseID) {
        panic!();
    }
}
