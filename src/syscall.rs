use super::kernel::{ObjectTable, PromiseTable, RunQueue, VatID};
use super::map_outbound::{get_outbound_promise, map_outbound_send};
use super::vat::{
    CapSlot as VatCapSlot, Message as VatMessage, PromiseID as VatPromiseID,
    Resolution as VatResolution, Syscall,
};
use super::vat_data::VatData;
use std::collections::HashMap;

// The SyscallHandler holds references to a subset of kernel data, the pieces
// necessary to map outbound messages and get them onto the runqueue (which
// is most of it, but notably not vat_dispatch). The handler is short-lived:
// created just before we invoke dispatch(), and deleted just afterwards, so
// we don't need long-term shared ownership of the kernel data structures.

pub struct SyscallHandler<'a> {
    for_vat: VatID,
    vat_data: &'a mut HashMap<VatID, VatData>,
    objects: &'a mut ObjectTable,
    promises: &'a mut PromiseTable,
    run_queue: &'a mut RunQueue,
}
impl<'a> SyscallHandler<'a> {
    pub fn new(
        for_vat: VatID,
        vat_data: &'a mut HashMap<VatID, VatData>,
        objects: &'a mut ObjectTable,
        promises: &'a mut PromiseTable,
        run_queue: &'a mut RunQueue,
    ) -> Self {
        SyscallHandler {
            for_vat,
            vat_data,
            objects,
            promises,
            run_queue,
        }
    }
}
impl<'a> Syscall for SyscallHandler<'a> {
    fn send(&mut self, target: VatCapSlot, msg: VatMessage) {
        let vd = self.vat_data.get_mut(&self.for_vat).unwrap();
        let pd = map_outbound_send(vd, self.promises, self.objects, target, msg);
        self.run_queue.add(pd);
    }
    fn subscribe(&mut self, id: VatPromiseID) {
        let vd = self.vat_data.get_mut(&self.for_vat).unwrap();
        let kpid = get_outbound_promise(vd, self.promises, id);
        self.promises.subscribe(kpid, self.for_vat);
    }
    fn resolve(&mut self, id: VatPromiseID, to: VatResolution) {}
}
