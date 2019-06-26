use std::fmt;

#[derive(Debug)]
pub struct VatPromiseID(pub u32);
impl fmt::Display for VatPromiseID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VatPromiseID-{}", self.0)
    }
}

#[derive(Debug)]
pub struct VatExportID(pub u32);
impl fmt::Display for VatExportID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VatExportID-{}", self.0)
    }
}

#[derive(Debug)]
pub struct VatImportID(pub u32);
impl fmt::Display for VatImportID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VatImportID-{}", self.0)
    }
}

pub enum VatSendTarget {
    Import(VatImportID),
    Promise(VatPromiseID),
}
impl fmt::Display for VatSendTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VatSendTarget::Import(id) => write!(f, "VatSendTarget({})", id),
            VatSendTarget::Promise(id) => write!(f, "VatSendTarget(Promise-{})", id),
        }
    }
}
