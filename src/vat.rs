use super::kernel::{PendingDelivery, RunQueue, VatData};
use super::kernel_types::{KernelPromiseID, KernelResolverID, KernelTarget};
use super::vat_types::{VatExportID, VatPromiseID, VatSendTarget};

pub(crate) struct VatManager<'a> {
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
}

pub trait Syscall {
    fn send(&mut self, target: VatSendTarget, name: &str) -> VatPromiseID;
}

impl<'a> Syscall for VatSyscall<'a> {
    fn send(&mut self, vtarget: VatSendTarget, name: &str) -> VatPromiseID {
        println!("syscall.send {}.{}", vtarget, name);
        let ktarget = self.map_outbound_target(vtarget);
        println!(" kt: {}.{}", ktarget, name);
        let (kpid, krid) = (self.vm.allocate_promise_resolver_pair)();
        let pd = PendingDelivery::new(ktarget, name, 0, krid);
        self.vm.run_queue.0.push_back(pd);
        self.vm.vat_data.promise_clist.map_inbound(kpid)
    }
}

pub trait Dispatch {
    fn deliver(&self, syscall: &mut dyn Syscall, target: VatExportID) -> ();
    fn notify_resolve_to_target(&self, id: VatPromiseID, target: u8);
}
