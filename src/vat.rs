use super::kernel::KernelData;
use super::kernel_types::VatID;
use super::syscall::{CapSlot, Message, Promise, Resolution, Syscall};
use std::cell::RefCell;
//use std::collections::HashSet;
use std::rc::Rc;

/*
enum TargetCategory {
    Export(KernelExport),                    // queue message to an Export
    Promise(VatID, KernelPromiseResolverID), // queue message to exported promise (pipelining)
    ToDataError,                             // error because you cannot send to data
    // TODO might be helpful to summarize the data
    Rejected(KernelCapData), // error: rejected-promise contagion
}
*/

pub(crate) struct VatSyscall {
    #[allow(dead_code)]
    vat_id: VatID,
    #[allow(dead_code)]
    kd: Rc<RefCell<KernelData>>,
}

impl VatSyscall {
    pub fn new(vat_id: VatID, kd: Rc<RefCell<KernelData>>) -> Self {
        VatSyscall { vat_id, kd }
    }

    /*
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

    fn push_notify_fulfill_to_target(
        &mut self,
        vat_id: VatID,
        kprid: KernelPromiseResolverID,
        ktarget: KernelExport,
    ) {
        let notification = PendingDelivery::NotifyFulfillToTarget {
            vat_id,
            target: kprid,
            result: ktarget,
        };
        let mut kd = self.kd.borrow_mut();
        kd.run_queue.0.push_back(notification);
    }

    fn push_notify_fulfill_to_data(
        &mut self,
        vat_id: VatID,
        kprid: KernelPromiseResolverID,
        data: KernelCapData,
    ) {
        let notification = PendingDelivery::NotifyFulfillToData {
            vat_id,
            target: kprid,
            data: data.clone(),
        };
        let mut kd = self.kd.borrow_mut();
        kd.run_queue.0.push_back(notification);
    }

    fn push_notify_reject(
        &mut self,
        vat_id: VatID,
        kprid: KernelPromiseResolverID,
        data: KernelCapData,
    ) {
        let notification = PendingDelivery::NotifyReject {
            vat_id,
            target: kprid,
            data: data.clone(),
        };
        let mut kd = self.kd.borrow_mut();
        kd.run_queue.0.push_back(notification);
    }
     */
}

impl Syscall for VatSyscall {
    fn send(&mut self, _target: CapSlot, _msg: Message) {
        //self.do_send(target, msg);
    }

    fn subscribe(&mut self, _id: Promise) {
        /*
        let mut kd = self.kd.borrow_mut();
        let vd = kd.vat_data.get_mut(&self.vat_id).unwrap();
        let kprid = vd.promise_clist.map_outbound(vpid);
        let p: &mut KernelPromise = kd.promises.get_mut(&kprid).unwrap();
        use KernelPromise::*;
        use PendingDelivery::*;
        let pd = match p {
            Unresolved {
                ref mut subscribers,
                ..
            } => {
                subscribers.insert(self.vat_id);
                return;
            }
            FulfilledToTarget(ktarget) => NotifyFulfillToTarget {
                vat_id: self.vat_id,
                target: kprid,
                result: *ktarget,
            },
            FulfilledToData(data) => NotifyFulfillToData {
                vat_id: self.vat_id,
                target: kprid,
                data: data.clone(),
            },
            Rejected(data) => NotifyReject {
                vat_id: self.vat_id,
                target: kprid,
                data: data.clone(),
            },
        };
        kd.run_queue.0.push_back(pd);
        */
    }

    fn resolve(&mut self, _id: Promise, _to: Resolution) {
        /*self.do_resolve(id, to);*/
    }

    /*
    fn fulfill_to_target(&mut self, resolver: VatResolverID, vtarget: VatResolveTarget) {
        use KernelPromise::{FulfilledToTarget, Unresolved};

        let kprid: KernelPromiseResolverID;
        let ktarget: KernelExport;
        let subscribers: Vec<VatID>;
        {
            let mut kd = self.kd.borrow_mut();
            let vd = kd.vat_data.get_mut(&self.vat_id).unwrap();
            kprid = vd.map_outbound_resolver(resolver);
            ktarget = vd.map_outbound_resolve_target(vtarget);
            let p = kd.promises.get(&kprid).unwrap();
            if let Unresolved {
                subscribers: subs,
                decider,
            } = p
            {
                // resolvers are not transferrable
                assert_eq!(*decider, self.vat_id);
                // TODO: HashSet.iter is nondeterministic
                subscribers = subs.iter().cloned().collect();
            } else {
                panic!(); // TODO: DuplicateFulfillError
            }
            let new_promise = FulfilledToTarget(ktarget);
            kd.promises.remove(&kprid);
            kd.promises.insert(kprid, new_promise);
        };

        for s in subscribers {
            self.push_notify_fulfill_to_target(s, kprid, ktarget);
        }
    }

    fn fulfill_to_data(&mut self, resolver: VatResolverID, vdata: VatCapData) {
        use KernelPromise::{FulfilledToData, Unresolved};
        let kdata = self.map_outbound_capdata(vdata);
        let kprid: KernelPromiseResolverID;
        let subscribers: Vec<VatID>;
        {
            let mut kd = self.kd.borrow_mut();
            let vd = kd.vat_data.get_mut(&self.vat_id).unwrap();
            kprid = vd.map_outbound_resolver(resolver);
            let p = kd.promises.get(&kprid).unwrap();
            if let Unresolved {
                subscribers: subs,
                decider,
            } = p
            {
                // resolvers are not transferrable
                assert_eq!(*decider, self.vat_id);
                // TODO: HashSet.iter is nondeterministic
                subscribers = subs.iter().cloned().collect();
            } else {
                panic!(); // TODO: DuplicateFulfillError
            }
            let new_promise = FulfilledToData(kdata.clone());
            kd.promises.remove(&kprid);
            kd.promises.insert(kprid, new_promise);
        };
        for s in subscribers {
            self.push_notify_fulfill_to_data(s, kprid, kdata.clone());
        }
    }

    fn reject(&mut self, resolver: VatResolverID, vdata: VatCapData) {
        use KernelPromise::{Rejected, Unresolved};
        let kdata = self.map_outbound_capdata(vdata);
        let kprid: KernelPromiseResolverID;
        let subscribers: Vec<VatID>;
        {
            let mut kd = self.kd.borrow_mut();
            let vd = kd.vat_data.get_mut(&self.vat_id).unwrap();
            kprid = vd.map_outbound_resolver(resolver);
            let p = kd.promises.get(&kprid).unwrap();
            if let Unresolved {
                subscribers: subs,
                decider,
            } = p
            {
                // resolvers are not transferrable
                assert_eq!(*decider, self.vat_id);
                // TODO: HashSet.iter is nondeterministic
                subscribers = subs.iter().cloned().collect();
            } else {
                panic!(); // TODO: DuplicateFulfillError
            }
            let new_promise = Rejected(kdata.clone());
            kd.promises.remove(&kprid);
            kd.promises.insert(kprid, new_promise);
        };
        for s in subscribers {
            self.push_notify_reject(s, kprid, kdata.clone());
        }
    }

    fn forward(&mut self, resolver: VatResolverID, vtarget: VatPromiseID) {
        use KernelPromise::*;

        let mut kd = self.kd.borrow_mut();
        let vd = kd.vat_data.get_mut(&self.vat_id).unwrap();
        let old_id = vd.map_outbound_resolver(resolver);
        let new_id = vd.get_outbound_promise(vtarget);

        // the old promise (the one being replaced/forwarded/resolved) must
        // be in the Unresolved state, and thus might have some subscribers
        let old_subscribers: Vec<VatID>;
        {
            let old_promise = kd.promises.get(&old_id).unwrap();
            match old_promise {
                Unresolved {
                    subscribers: subs,
                    decider,
                } => {
                    // resolvers are not transferrable
                    assert_eq!(*decider, self.vat_id);
                    // TODO: HashSet.iter is nondeterministic
                    old_subscribers = subs.iter().cloned().collect();
                }
                _ => panic!(), // TODO: DuplicateFulfillError
            };
        }
        kd.promises.remove(&old_id);

        // Walk through all clists and replace every mention of the old
        // promise with the new target
        for vd in kd.vat_data.values_mut() {
            vd.forward_promise(old_id, new_id);
        }

        // The new promise might have already been fulfilled, so the old
        // subscribers must be notified about the fulfillment. Or, if the new
        // promise is still unresolved, make the old subscribers watch the
        // new promise instead.
        use PendingDelivery::*;
        let new_promise = kd.promises.get_mut(&new_id).unwrap();
        let pds: Vec<PendingDelivery> = match new_promise {
            Unresolved {
                subscribers: new_subscribers,
                ..
            } => {
                for s in old_subscribers {
                    new_subscribers.insert(s);
                }
                return;
            }
            FulfilledToTarget(ktarget) => old_subscribers
                .iter()
                .map(|s| NotifyFulfillToTarget {
                    vat_id: *s,
                    target: old_id,
                    result: *ktarget,
                })
                .collect(),
            FulfilledToData(data) => old_subscribers
                .iter()
                .map(|s| NotifyFulfillToData {
                    vat_id: *s,
                    target: old_id,
                    data: data.clone(),
                })
                .collect(),
            Rejected(data) => old_subscribers
                .iter()
                .map(|s| NotifyReject {
                    vat_id: *s,
                    target: old_id,
                    data: data.clone(),
                })
                .collect(),
        };
        for pd in pds {
            kd.run_queue.0.push_back(pd);
        }
    }
    */
}
