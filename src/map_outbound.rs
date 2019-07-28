use super::kernel::{
    CapData as KernelCapData, CapSlot as KernelCapSlot, Message as KernelMessage,
    ObjectID as KernelObjectID, ObjectTable as KernelObjectTable,
    PromiseID as KernelPromiseID, PromiseTable as KernelPromiseTable,
    Resolution as KernelResolution,
};
use super::vat::{
    CapData as VatCapData, CapSlot as VatCapSlot, InboundTarget, Message as VatMessage,
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

fn map_outbound_slot(
    vd: &mut KernelVatData,
    pt: &mut KernelPromiseTable,
    ot: &mut KernelObjectTable,
    slot: VatCapSlot,
) -> KernelCapSlot {
    match slot {
        VatCapSlot::Promise(id) => {
            KernelCapSlot::Promise(map_outbound_promise(vd, pt, id))
        }
        VatCapSlot::Object(id) => KernelCapSlot::Object({
            let owner = vd.id;
            let allocate = || ot.allocate(owner);
            vd.object_clist.map_outbound(id, allocate)
        }),
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

/*
fn map_outbound_result(
    vd: &mut KernelVatData,
    pt: &mut KernelPromiseTable,
    id: VatPromiseID,
) -> KernelPromiseID {
    // this is only for answers
    let decider = vd.id; // TODO
    let allocator = vd.id;
    let allocate = || pt.allocate_unresolved(decider, allocator);
    vd.promise_clist.map_outbound(id, allocate)
}

fn map_outbound_message(
    vd: &mut KernelVatData,
    pt: &mut KernelPromiseTable,
    ot: &mut KernelObjectTable,
    message: VatMessage,
) -> KernelMessage {
    // look up the target first, promise or object, and find it's decider/owner
    // then if a result promise must be allocated, use that as the decider
    KernelMessage {
        method: message.method,
        args: map_outbound_capdata(vd, pt, ot, message.args),
        result: message.result.map(|rp| map_outbound_result(vd, pt, rp)),
    }
}
*/

/*
fn map_outbound_resolution(
    vd: &mut KernelVatData,
    pt: &mut KernelPromiseTable,
    ot: &mut KernelObjectTable,
    resolution: VatResolution,
) -> KernelResolution {
}
*/
