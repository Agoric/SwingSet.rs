use super::vat_types::{VatMessage, VatPromiseID, VatSendTarget};

pub trait Syscall {
    fn send(&mut self, target: VatSendTarget, vmsg: VatMessage) -> VatPromiseID;
}
