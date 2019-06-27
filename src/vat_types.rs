use std::fmt;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct VatPromiseID(pub u32);
impl fmt::Display for VatPromiseID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VatPromiseID-{}", self.0)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct VatExportID(pub u32);
impl fmt::Display for VatExportID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VatExportID-{}", self.0)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct VatImportID(pub u32);
impl fmt::Display for VatImportID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VatImportID-{}", self.0)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum VatSendTarget {
    Import(VatImportID),
    Promise(VatPromiseID),
}
impl fmt::Display for VatSendTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use VatSendTarget::*;
        match self {
            Import(id) => write!(f, "VatSendTarget({})", id),
            Promise(id) => write!(f, "VatSendTarget(Promise-{})", id),
        }
    }
}

impl From<VatImportID> for VatSendTarget {
    fn from(target: VatImportID) -> VatSendTarget {
        VatSendTarget::Import(target)
    }
}

impl From<VatPromiseID> for VatSendTarget {
    fn from(target: VatPromiseID) -> VatSendTarget {
        VatSendTarget::Promise(target)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum VatArgSlot {
    Import(VatImportID),
    Export(VatExportID),
    Promise(VatPromiseID),
}

impl fmt::Display for VatArgSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use VatArgSlot::*;
        match self {
            Import(id) => write!(f, "varg-import-{}", id),
            Export(id) => write!(f, "varg-export-{}", id),
            Promise(id) => write!(f, "varg-promise-{}", id),
        }
    }
}

impl From<VatSendTarget> for VatArgSlot {
    fn from(target: VatSendTarget) -> VatArgSlot {
        match target {
            VatSendTarget::Import(id) => VatArgSlot::Import(id),
            VatSendTarget::Promise(id) => VatArgSlot::Promise(id),
        }
    }
}
