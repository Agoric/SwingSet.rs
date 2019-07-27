/// This contains the functions which translate between kernel-space and
/// vat-space.
use super::clist::CListVatEntry;
use super::kernel::{
    CapSlot as KernelCapSlot, ObjectID as KernelObjectID,
    ObjectTable as KernelObjectTable, PromiseID as KernelPromiseID,
    PromiseTable as KernelPromiseTable, VatData as KernelVatData,
};
use super::vat::{
    CapSlot as VatCapSlot, ObjectID as VatObjectID, PromiseID as VatPromiseID,
};

impl CListVatEntry for VatObjectID {
    fn new(index: isize) -> Self {
        VatObjectID(index)
    }
}

impl CListVatEntry for VatPromiseID {
    fn new(index: isize) -> Self {
        VatPromiseID(index)
    }
}

// These functions map the arguments of "inbound" kernel->vat dispatch calls.
// This may require allocation in the target Vat's c-lists, but not the
// kernel tables.

fn map_inbound_promise(
    vd: &mut KernelVatData,
    pt: &KernelPromiseTable,
    id: KernelPromiseID,
) -> VatPromiseID {
    let allocator = pt.promises.get(&id).unwrap().allocator;
    if allocator == vd.id {
        // this is returning home. It should be in the clist already.
        vd.promise_clist.get_inbound(id).unwrap()
    } else {
        // this is coming from afar, so allocate if necessary
        vd.promise_clist.map_inbound(id)
    }
}

fn map_inbound_slot(
    vd: &mut KernelVatData,
    ot: &KernelObjectTable,
    pt: &KernelPromiseTable,
    slot: KernelCapSlot,
) -> VatCapSlot {
    match slot {
        KernelCapSlot::Object(id) => VatCapSlot::Object({
            let owner = ot.objects.get(&id).unwrap().owner;
            if owner == vd.id {
                // this is returning home
                vd.object_clist.get_inbound(id).unwrap()
            } else {
                vd.object_clist.map_inbound(id)
            }
        }),
        KernelCapSlot::Promise(id) => {
            VatCapSlot::Promise(map_inbound_promise(vd, pt, id))
        }
    }
}
