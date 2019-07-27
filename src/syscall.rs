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
    if pt.allocator_of(id) == vd.id {
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
            if ot.owner_of(id) == vd.id {
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

#[cfg(test)]
mod test {
    use super::super::kernel::{
        ObjectTable, PromiseID as KernelPromiseID, PromiseTable, VatData, VatID,
    };
    use super::super::vat::PromiseID as VatPromiseID;
    use super::*;

    #[test]
    fn test_map_inbound_promise() {
        let us = VatID(1);
        let them = VatID(2);
        let decider = VatID(3);
        let mut vd = VatData::new(us);
        let mut pt = PromiseTable::new();
        let p1 = pt.allocate_unresolved(decider, us); // ours
        vd.promise_clist.add(p1, VatPromiseID(10));
        let p2 = pt.allocate_unresolved(decider, them); // not ours

        assert_eq!(map_inbound_promise(&mut vd, &mut pt, p1), VatPromiseID(10));

        assert_eq!(map_inbound_promise(&mut vd, &mut pt, p2), VatPromiseID(-1));
        // mapping should be stable
        assert_eq!(map_inbound_promise(&mut vd, &mut pt, p2), VatPromiseID(-1));
    }

    #[test]
    fn test_map_inbound_slot() {
        let us = VatID(1);
        let them = VatID(2);
        let decider = VatID(3);
        let mut vd = VatData::new(us);
        let mut pt = PromiseTable::new();
        let mut ot = ObjectTable::new();

        let o1 = ot.allocate(us); // ours
        vd.object_clist.add(o1, VatObjectID(10));
        let o2 = ot.allocate(them); // not ours
        let ko1 = KernelCapSlot::Object(o1);
        let ko2 = KernelCapSlot::Object(o2);

        let p1 = pt.allocate_unresolved(decider, us); // ours
        vd.promise_clist.add(p1, VatPromiseID(20));
        let p2 = pt.allocate_unresolved(decider, them); // not ours
        let kp1 = KernelCapSlot::Promise(p1);
        let kp2 = KernelCapSlot::Promise(p2);

        assert_eq!(
            map_inbound_slot(&mut vd, &ot, &pt, ko1),
            VatCapSlot::Object(VatObjectID(10))
        );
        assert_eq!(
            map_inbound_slot(&mut vd, &ot, &pt, ko2),
            VatCapSlot::Object(VatObjectID(-1))
        );
        assert_eq!(
            map_inbound_slot(&mut vd, &ot, &pt, ko2),
            VatCapSlot::Object(VatObjectID(-1))
        );

        assert_eq!(
            map_inbound_slot(&mut vd, &ot, &pt, kp1),
            VatCapSlot::Promise(VatPromiseID(20))
        );
        assert_eq!(
            map_inbound_slot(&mut vd, &ot, &pt, kp2),
            VatCapSlot::Promise(VatPromiseID(-1))
        );
        assert_eq!(
            map_inbound_slot(&mut vd, &ot, &pt, kp2),
            VatCapSlot::Promise(VatPromiseID(-1))
        );
    }

}
