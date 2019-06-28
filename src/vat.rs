use super::kernel::{PendingDelivery, RunQueue, VatData};
use super::kernel_types::{
    KernelArgSlot, KernelCapData, KernelExport, KernelExportID, KernelMessage,
    KernelPromiseID, KernelResolverID, KernelTarget, VatID,
};
use super::syscall::Syscall;
use super::vat_types::{VatArgSlot, VatMessage, VatPromiseID, VatSendTarget};

pub(crate) struct VatManager<'a> {
    pub vat_id: VatID,
    pub run_queue: &'a mut RunQueue,
    pub vat_data: &'a mut VatData,
    pub allocate_promise_resolver_pair: &'a Fn() -> (KernelPromiseID, KernelResolverID),
}

pub(crate) struct VatSyscall<'a> {
    vm: VatManager<'a>,
}

impl<'a> VatSyscall<'a> {
    pub fn new(manager: VatManager<'a>) -> Self {
        VatSyscall { vm: manager }
    }
    fn map_outbound_target(&self, vtarget: VatSendTarget) -> KernelTarget {
        match vtarget {
            VatSendTarget::Import(viid) => {
                let ke = self.vm.vat_data.import_clist.map_outbound(viid);
                KernelTarget::Export(ke)
            }
            VatSendTarget::Promise(vpid) => {
                let kpid = self.vm.vat_data.promise_clist.map_outbound(vpid);
                KernelTarget::Promise(kpid)
            }
        }
    }

    fn map_outbound_arg_slot(&self, varg: VatArgSlot) -> KernelArgSlot {
        match varg {
            VatArgSlot::Import(viid) => {
                let ke = self.vm.vat_data.import_clist.map_outbound(viid);
                KernelArgSlot::Export(ke)
            }
            VatArgSlot::Export(veid) => {
                let keid = KernelExportID(veid.0);
                KernelArgSlot::Export(KernelExport(self.vm.vat_id, keid))
            }
            VatArgSlot::Promise(vpid) => {
                let kpid = self.vm.vat_data.promise_clist.map_outbound(vpid);
                KernelArgSlot::Promise(kpid)
            }
        }
    }
}

impl<'a> Syscall for VatSyscall<'a> {
    fn send(&mut self, vtarget: VatSendTarget, vmsg: VatMessage) -> VatPromiseID {
        println!("syscall.send {}.{}", vtarget, vmsg.name);
        let ktarget = self.map_outbound_target(vtarget);
        let (kpid, krid) = (self.vm.allocate_promise_resolver_pair)();
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
        self.vm.run_queue.0.push_back(pd);
        self.vm.vat_data.promise_clist.map_inbound(kpid)
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
        self.vm.run_queue.0.push_back(pd);
    }
}
