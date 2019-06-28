use super::kernel::{KernelData, PendingDelivery};
use super::kernel_types::{
    KernelArgSlot, KernelCapData, KernelExport, KernelExportID, KernelMessage,
    KernelPromiseID, KernelResolverID, KernelTarget, VatID,
};
use super::syscall::Syscall;
use super::vat_types::{
    VatArgSlot, VatCapData, VatExportID, VatMessage, VatPromiseID, VatResolverID,
    VatSendTarget,
};
use std::cell::RefCell;
use std::rc::Rc;

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

    fn allocate_promise_resolver_pair(&self) -> (VatPromiseID, KernelResolverID) {
        let mut kd = self.kd.borrow_mut();
        let id = kd.next_promise_resolver_id;
        kd.next_promise_resolver_id = id + 1;
        let krid = KernelResolverID(id);
        let kpid = KernelPromiseID(id);
        let vd = kd.vat_data.get_mut(&self.vat_id).unwrap();
        let vpid = vd.promise_clist.map_inbound(kpid);
        (vpid, krid)
    }
}

impl Syscall for VatSyscall {
    fn send(&mut self, vtarget: VatSendTarget, vmsg: VatMessage) -> VatPromiseID {
        println!("syscall.send {}.{}", vtarget, vmsg.name);
        let ktarget = self.map_outbound_target(vtarget);
        let (vpid, krid) = self.allocate_promise_resolver_pair();

        let kmsg = KernelMessage {
            name: vmsg.name.to_string(),
            args: KernelCapData {
                body: vmsg.args.body,
                slots: vmsg
                    .args
                    .slots
                    .into_iter()
                    .map(|slot| self.map_outbound_arg_slot(slot))
                    .collect(),
            },
        };
        println!(" kt: {}.{}", ktarget, kmsg.name);
        let pd = PendingDelivery::new(ktarget, kmsg, Some(krid));

        self.kd.borrow_mut().run_queue.0.push_back(pd);
        vpid
    }

    fn send_only(&mut self, vtarget: VatSendTarget, vmsg: VatMessage) {
        println!("syscall.send {}.{}", vtarget, vmsg.name);
        let ktarget = self.map_outbound_target(vtarget);
        let kmsg = KernelMessage {
            name: vmsg.name.to_string(),
            args: KernelCapData {
                body: vmsg.args.body,
                slots: vmsg
                    .args
                    .slots
                    .into_iter()
                    .map(|slot| self.map_outbound_arg_slot(slot))
                    .collect(),
            },
        };
        println!(" kt: {}.{}", ktarget, kmsg.name);
        let pd = PendingDelivery::new(ktarget, kmsg, None);
        self.kd.borrow_mut().run_queue.0.push_back(pd);
    }

    fn allocate_promise_and_resolver(&mut self) -> (VatPromiseID, VatResolverID) {
        panic!();
    }
    fn subscribe(&mut self, _id: VatPromiseID) {
        panic!();
    }
    fn fulfill_to_target(&mut self, _resolver: VatResolverID, _target: VatExportID) {
        panic!();
    }
    fn fulfill_to_data(&mut self, _resolver: VatResolverID, _data: VatCapData) {
        panic!();
    }
    fn reject(&mut self, _resolver: VatResolverID, _data: VatCapData) {
        panic!();
    }
    fn forward(&mut self, _resolver: VatResolverID, _target: VatPromiseID) {
        panic!();
    }
}
