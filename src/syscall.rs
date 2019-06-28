use super::vat_types::{
    VatCapData, VatExportID, VatMessage, VatPromiseID, VatResolverID, VatSendTarget,
};

pub trait Syscall {
    fn send(&mut self, target: VatSendTarget, vmsg: VatMessage) -> VatPromiseID;
    fn send_only(&mut self, target: VatSendTarget, vmsg: VatMessage);
    //fn invoke(&mut self, target: VatDeviceID, vmsg: VatMessage) -> VatCapData;
    fn allocate_promise_and_resolver(&mut self) -> (VatPromiseID, VatResolverID);
    fn subscribe(&mut self, id: VatPromiseID);
    fn fulfill_to_target(&mut self, resolver: VatResolverID, target: VatExportID);
    fn fulfill_to_data(&mut self, resolver: VatResolverID, data: VatCapData);
    fn reject(&mut self, resolver: VatResolverID, data: VatCapData);
    fn forward(&mut self, resolver: VatResolverID, target: VatPromiseID);
}
