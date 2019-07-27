use super::clist::CListVatEntry;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct VatPresenceID(usize);

impl CListVatEntry for VatPresenceID {
    fn new(index: usize) -> Self {
        VatPresenceID(index)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct VatPromiseID(usize);

impl CListVatEntry for VatPromiseID {
    fn new(index: usize) -> Self {
        VatPromiseID(index)
    }
}
