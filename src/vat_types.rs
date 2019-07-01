use std::fmt;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct VatPromiseID(pub u32);
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct VatResolverID(pub u32);
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct VatExportID(pub u32);
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct VatImportID(pub u32);

/// dispatch.notify_fulfill_to_target gives us a VatResolveTarget
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum VatResolveTarget {
    Import(VatImportID),
    Export(VatExportID),
}
/// syscall.send must point at a VatSendTarget
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum VatSendTarget {
    Import(VatImportID),
    Promise(VatPromiseID),
}
/// VatCapData can contain VatArgSlots
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum VatArgSlot {
    Import(VatImportID),
    Export(VatExportID),
    Promise(VatPromiseID),
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

impl From<VatSendTarget> for VatArgSlot {
    fn from(target: VatSendTarget) -> VatArgSlot {
        match target {
            VatSendTarget::Import(id) => VatArgSlot::Import(id),
            VatSendTarget::Promise(id) => VatArgSlot::Promise(id),
        }
    }
}

impl fmt::Display for VatPromiseID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VatPromiseID-{}", self.0)
    }
}

impl fmt::Display for VatResolverID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VatResolverID-{}", self.0)
    }
}

impl fmt::Display for VatExportID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VatExportID-{}", self.0)
    }
}

impl fmt::Display for VatImportID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VatImportID-{}", self.0)
    }
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

/// VatCapData is used for the arguments of syscall.send, dispatch.deliver,
/// fulfill_to_data, and reject
#[derive(Debug, Clone)]
pub struct VatCapData {
    pub body: Vec<u8>,
    pub slots: Vec<VatArgSlot>,
}

#[derive(Debug)]
pub struct OutboundVatMessage {
    pub name: String,
    pub args: VatCapData,
}
impl OutboundVatMessage {
    pub fn new(name: &str, body: &[u8], slots: Vec<VatArgSlot>) -> Self {
        OutboundVatMessage {
            name: name.to_string(),
            args: VatCapData {
                body: body.to_vec(),
                slots,
            },
        }
    }
}

#[derive(Debug)]
pub struct InboundVatMessage {
    pub name: String,
    pub args: VatCapData,
    pub resolver: Option<VatResolverID>,
}
impl InboundVatMessage {
    pub fn new(
        name: &str,
        body: &[u8],
        slots: Vec<VatArgSlot>,
        resolver: Option<VatResolverID>,
    ) -> Self {
        InboundVatMessage {
            name: name.to_string(),
            args: VatCapData {
                body: body.to_vec(),
                slots,
            },
            resolver,
        }
    }
}
