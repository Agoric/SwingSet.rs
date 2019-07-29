use super::kernel::{
    CapData as KernelCapData, CapSlot as KernelCapSlot, Message as KernelMessage,
    ObjectTable as KernelObjectTable, PromiseID as KernelPromiseID,
    PromiseTable as KernelPromiseTable, Resolution as KernelResolution,
};
use super::vat::{
    CapData as VatCapData, CapSlot as VatCapSlot, InboundTarget, Message as VatMessage,
    PromiseID as VatPromiseID, Resolution as VatResolution,
};
use super::vat_data::VatData as KernelVatData;

// These functions map the arguments of "inbound" kernel->vat dispatch calls.
// This may require allocation in the target Vat's c-lists, but not the
// kernel tables.

pub fn map_inbound_promise(
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

pub fn map_inbound_target(
    vd: &mut KernelVatData,
    ot: &KernelObjectTable,
    pt: &KernelPromiseTable,
    target: KernelCapSlot,
) -> InboundTarget {
    match map_inbound_slot(vd, ot, pt, target) {
        VatCapSlot::Object(id) => InboundTarget::Object(id),
        VatCapSlot::Promise(id) => InboundTarget::Promise(id),
    }
}

fn map_inbound_capdata(
    vd: &mut KernelVatData,
    ot: &KernelObjectTable,
    pt: &KernelPromiseTable,
    data: KernelCapData,
) -> VatCapData {
    VatCapData {
        body: data.body,
        slots: data
            .slots
            .iter()
            .map(|s| map_inbound_slot(vd, ot, pt, *s))
            .collect(),
    }
}

pub fn map_inbound_message(
    vd: &mut KernelVatData,
    ot: &KernelObjectTable,
    pt: &KernelPromiseTable,
    message: KernelMessage,
) -> VatMessage {
    VatMessage {
        method: message.method,
        args: map_inbound_capdata(vd, ot, pt, message.args),
        result: message.result.map(|rp| map_inbound_promise(vd, pt, rp)),
    }
}

pub fn map_inbound_resolution(
    vd: &mut KernelVatData,
    ot: &KernelObjectTable,
    pt: &KernelPromiseTable,
    resolution: KernelResolution,
) -> VatResolution {
    match resolution {
        KernelResolution::Reference(s) => {
            VatResolution::Reference(map_inbound_slot(vd, ot, pt, s))
        }
        KernelResolution::Data(d) => {
            VatResolution::Data(map_inbound_capdata(vd, ot, pt, d))
        }
        KernelResolution::Rejection(d) => {
            VatResolution::Rejection(map_inbound_capdata(vd, ot, pt, d))
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::kernel::{ObjectTable, PromiseTable, VatID};
    use super::super::vat::{ObjectID as VatObjectID, PromiseID as VatPromiseID};
    use super::super::vat_data::VatData;
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
