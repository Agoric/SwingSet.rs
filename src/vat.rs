use super::kernel::{CList, PendingDelivery, RunQueue};
use super::kernel_types::{KernelResolverID};
use super::vat_types::{VatExportID, VatPromiseID, VatSendTarget};

pub trait Syscall {
    fn send(&mut self, target: VatSendTarget, name: &str) -> VatPromiseID;
}

#[derive(Debug)]
pub(crate) struct VatSyscall<'a> {
    run_queue: &'a mut RunQueue,
    clist: &'a mut CList,
}
impl<'a> VatSyscall<'a> {
    pub fn new(run_queue: &'a mut RunQueue, clist: &'a mut CList) -> Self {
        VatSyscall { run_queue, clist }
    }
}
impl<'a> Syscall for VatSyscall<'a> {
    fn send(&mut self, vtarget: VatSendTarget, name: &str) -> VatPromiseID {
        println!("syscall.send {}.{}", vtarget, name);
        let ktarget = self.clist.map_outbound_target(vtarget);
        println!(" kt: {}.{}", ktarget, name);
        let pd = PendingDelivery::new(ktarget, name, 0, KernelResolverID(0));
        self.run_queue.0.push_back(pd);
        VatPromiseID(1)
    }
}

pub trait Dispatch {
    fn deliver(&self, syscall: &mut dyn Syscall, target: VatExportID) -> ();
    fn notify_resolve_to_target(&self, id: VatPromiseID, target: u8);
}
