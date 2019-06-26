use super::kernel::PendingDelivery;
use super::vat_types::{VatExportID, VatPromiseID, VatSendTarget};
use std::collections::VecDeque;

pub trait Syscall {
    fn send(&mut self, target: VatSendTarget, name: &str) -> VatPromiseID;
}

#[derive(Debug)]
pub struct VatSyscall {}
impl VatSyscall {
    pub fn new(_run_queue: &mut VecDeque<PendingDelivery>) -> Self {
        VatSyscall {}
    }
}
impl Syscall for VatSyscall {
    fn send(&mut self, target: VatSendTarget, name: &str) -> VatPromiseID {
        println!("syscall.send {}.{}", target, name);
        VatPromiseID(1)
    }
}

pub trait Dispatch {
    fn deliver(&self, syscall: &mut dyn Syscall, target: VatExportID) -> ();
    fn notify_resolve_to_target(&self, id: VatPromiseID, target: u8);
}
