use super::clist::CListVatEntry;
use super::kernel::{CapSlot, ObjectID, ObjectTable, PromiseID, PromiseTable, VatData};
use super::vat::{VatCapSlot, VatObjectID, VatPromiseID};

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
    vd: &mut VatData,
    pt: &PromiseTable,
    id: PromiseID,
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
    vd: &mut VatData,
    ot: &ObjectTable,
    pt: &PromiseTable,
    slot: CapSlot,
) -> VatCapSlot {
    match slot {
        CapSlot::Object(id) => VatCapSlot::Object({
            let owner = ot.objects.get(&id).unwrap().owner;
            if owner == vd.id {
                // this is returning home
                vd.object_clist.get_inbound(id).unwrap()
            } else {
                vd.object_clist.map_inbound(id)
            }
        }),
        CapSlot::Promise(id) => VatCapSlot::Promise(map_inbound_promise(vd, pt, id)),
    }
}
