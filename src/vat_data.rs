use super::clist::{CList, CListKernelEntry, CListVatEntry};
use super::kernel::{ObjectID as KernelObjectID, PromiseID as KernelPromiseID, VatID};
use super::vat::{ObjectID as VatObjectID, PromiseID as VatPromiseID};

impl CListKernelEntry for KernelObjectID {}
impl CListKernelEntry for KernelPromiseID {}

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

#[derive(Debug)]
pub struct VatData {
    pub id: VatID,
    pub object_clist: CList<KernelObjectID, VatObjectID>,
    pub promise_clist: CList<KernelPromiseID, VatPromiseID>,
}
impl VatData {
    pub fn new(id: VatID) -> Self {
        VatData {
            id,
            object_clist: CList::new(),
            promise_clist: CList::new(),
        }
    }
}
