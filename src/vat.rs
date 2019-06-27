use super::kernel::{CList, PendingDelivery, RunQueue};
use super::kernel_types::{KernelPromiseID, KernelResolverID};
use super::vat_types::{VatExportID, VatPromiseID, VatSendTarget};

pub(crate) struct VatManager<'a> {
    pub run_queue: &'a mut RunQueue,
    pub clist: &'a mut CList,
    pub allocate_promise_resolver_pair: &'a Fn() -> (KernelPromiseID, KernelResolverID),
}

pub trait Syscall {
    fn send(&mut self, target: VatSendTarget, name: &str) -> VatPromiseID;
}

pub(crate) struct VatSyscall<'a> {
    run_queue: &'a mut RunQueue,
    clist: &'a mut CList,
    allocate_promise_resolver_pair: &'a Fn() -> (KernelPromiseID, KernelResolverID),
}
impl<'a> VatSyscall<'a> {
    pub fn new(manager: VatManager<'a>) -> Self {
        VatSyscall {
            run_queue: manager.run_queue,
            clist: manager.clist,
            allocate_promise_resolver_pair: manager.allocate_promise_resolver_pair,
        }
    }
}
impl<'a> Syscall for VatSyscall<'a> {
    fn send(
        &mut self,
        vtarget: VatSendTarget,
        name: &str, /*,
                    apr: u8*/
    ) -> VatPromiseID {
        println!("syscall.send {}.{}", vtarget, name);
        let ktarget = self.clist.map_outbound_target(vtarget);
        println!(" kt: {}.{}", ktarget, name);
        let (kpid, krid) = (self.allocate_promise_resolver_pair)();
        let pd = PendingDelivery::new(ktarget, name, 0, krid);
        self.run_queue.0.push_back(pd);
        self.clist.map_inbound_promise(kpid)
    }
}

pub trait Dispatch {
    fn deliver(&self, syscall: &mut dyn Syscall, target: VatExportID) -> ();
    fn notify_resolve_to_target(&self, id: VatPromiseID, target: u8);
}
