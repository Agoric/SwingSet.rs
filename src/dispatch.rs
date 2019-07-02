use super::vat_types::{
    InboundVatMessage, VatCapData, VatExportID, VatPromiseID, VatResolveTarget,
    VatResolverID,
};

// TODO: we need a name for the pass-by-presence type. "target"? "export"?

pub trait Dispatch {
    fn deliver(&mut self, target: VatExportID, message: InboundVatMessage);
    fn deliver_promise(&mut self, target: VatResolverID, message: InboundVatMessage);
    fn notify_fulfill_to_target(&mut self, id: VatPromiseID, target: VatResolveTarget);
    fn notify_fulfill_to_data(&mut self, id: VatPromiseID, data: VatCapData);
    fn notify_reject(&mut self, id: VatPromiseID, data: VatCapData);
}
