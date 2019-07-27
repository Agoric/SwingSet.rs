use super::clist::CListVatEntry;
use super::kernel::{PromiseID, PromiseTable, VatData};
use super::vat::{VatObjectID, VatPromiseID};

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

/*
fn map_inbound_slot(vd: &mut VataData, slot: CapSlot) -> VatCapSlot {
    match slot {
        CapSlot::Presence(id) => {
            let mut kd = self.kd.borrow_mut();
            let owner = kd.presences.presences.get(&id).unwrap().owner;
            let vd = kd.vat_data.get_mut(&owner).unwrap();
            if to == owner {
                VatCapSlot::Export(vd.export_clist.get_inbound(id))
            } else {
                VatCapSlot::Import(vd.import_clist.map_inbound(id))
            }
        }
        CapSlot::Promise(id) => match self.map_inbound_promise(to, id) {
            VatPromise::LocalPromise(pid) => VatCapSlot::LocalPromise(pid),
            VatPromise::RemotePromise(pid) => VatCapSlot::RemotePromise(pid),
        },
    }
}
*/
