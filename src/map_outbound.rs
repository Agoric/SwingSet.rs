use super::kernel::{
    CapData as KernelCapData, CapSlot as KernelCapSlot, Message as KernelMessage,
    ObjectID as KernelObjectID, ObjectTable as KernelObjectTable, PendingDelivery,
    PromiseID as KernelPromiseID, PromiseTable as KernelPromiseTable,
    Resolution as KernelResolution, VatID,
};
use super::vat::{
    CapData as VatCapData, CapSlot as VatCapSlot, Message as VatMessage,
    ObjectID as VatObjectID, PromiseID as VatPromiseID, Resolution as VatResolution,
};
use super::vat_data::VatData as KernelVatData;

fn map_outbound_promise(
    vd: &mut KernelVatData,
    pt: &mut KernelPromiseTable,
    id: VatPromiseID,
) -> KernelPromiseID {
    // this is not for answers
    let decider = vd.id;
    let allocator = vd.id;
    let allocate = || pt.allocate_unresolved(decider, allocator);
    vd.promise_clist.map_outbound(id, allocate)
}

pub fn map_outbound_object(
    vd: &mut KernelVatData,
    ot: &mut KernelObjectTable,
    vid: VatObjectID,
) -> KernelObjectID {
    let owner = vd.id;
    let allocate = || ot.allocate(owner);
    vd.object_clist.map_outbound(vid, allocate)
}

fn map_outbound_slot(
    vd: &mut KernelVatData,
    pt: &mut KernelPromiseTable,
    ot: &mut KernelObjectTable,
    slot: VatCapSlot,
) -> KernelCapSlot {
    use VatCapSlot::*;
    match slot {
        Promise(vid) => KernelCapSlot::Promise(map_outbound_promise(vd, pt, vid)),
        Object(vid) => KernelCapSlot::Object(map_outbound_object(vd, ot, vid)),
    }
}

pub fn get_outbound_slot(vd: &mut KernelVatData, slot: VatCapSlot) -> KernelCapSlot {
    // must already exist
    use VatCapSlot::*;
    match slot {
        Promise(id) => KernelCapSlot::Promise(vd.promise_clist.get_outbound(id).unwrap()),
        Object(id) => KernelCapSlot::Object(vd.object_clist.get_outbound(id).unwrap()),
    }
}

fn map_outbound_capdata(
    vd: &mut KernelVatData,
    pt: &mut KernelPromiseTable,
    ot: &mut KernelObjectTable,
    data: VatCapData,
) -> KernelCapData {
    KernelCapData {
        body: data.body,
        slots: data
            .slots
            .iter()
            .map(|s| map_outbound_slot(vd, pt, ot, *s))
            .collect(),
    }
}

fn map_outbound_result(
    vd: &mut KernelVatData,
    pt: &mut KernelPromiseTable,
    target_vatid: VatID,
    id: VatPromiseID,
) -> KernelPromiseID {
    // this is only for answers
    let decider = target_vatid;
    let allocator = vd.id;
    let allocate = || pt.allocate_unresolved(decider, allocator);
    vd.promise_clist.map_outbound(id, allocate)
}

pub fn map_outbound_send(
    vd: &mut KernelVatData,
    pt: &mut KernelPromiseTable,
    ot: &mut KernelObjectTable,
    target: KernelCapSlot,
    decider_vatid: VatID,
    message: VatMessage,
) -> PendingDelivery {
    // we've already mapped the target to a KernelCapSlot, and looked up the
    // decider/owner to use for any result promise that we might allocate

    let km = KernelMessage {
        method: message.method,
        args: map_outbound_capdata(vd, pt, ot, message.args),
        result: message
            .result
            .map(|rp| map_outbound_result(vd, pt, decider_vatid, rp)),
    };
    PendingDelivery::Deliver {
        target,
        message: km,
    }
}

pub fn get_outbound_promise(vd: &mut KernelVatData, id: VatPromiseID) -> KernelPromiseID {
    // this is for resolutions, not for answers. must already exist
    // TODO: check that the sending vat is the decider
    vd.promise_clist.get_outbound(id).unwrap()
}

pub fn map_outbound_resolution(
    vd: &mut KernelVatData,
    pt: &mut KernelPromiseTable,
    ot: &mut KernelObjectTable,
    resolution: VatResolution,
) -> KernelResolution {
    use VatResolution::*;
    match resolution {
        Reference(vslot) => {
            KernelResolution::Reference(map_outbound_slot(vd, pt, ot, vslot))
        }
        Data(vdata) => KernelResolution::Data(map_outbound_capdata(vd, pt, ot, vdata)),
        Rejection(vdata) => {
            KernelResolution::Rejection(map_outbound_capdata(vd, pt, ot, vdata))
        }
    }
}
