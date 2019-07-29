use super::vat::{
    CapSlot as VatCapSlot, Message as VatMessage, PromiseID as VatPromiseID,
    Resolution as VatResolution, Syscall,
};

pub struct SyscallHandler {}
impl SyscallHandler {
    pub fn new() -> Self {
        SyscallHandler {}
    }
}
impl Syscall for SyscallHandler {
    fn send(&mut self, target: VatCapSlot, msg: VatMessage) {
        //let pd = map_outbound_send(vd, pt, ot, target, msg);
        //run_queue.push_back(pd);
    }
    fn subscribe(&mut self, id: VatPromiseID) {}
    fn resolve(&mut self, id: VatPromiseID, to: VatResolution) {}
}
