use std::fmt;

#[derive(PartialEq, Eq, Debug, Hash)]
pub struct VatName(pub String);

#[derive(PartialEq, Eq, Debug, Hash, Copy, Clone)]
pub struct VatID(pub u32);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct KernelExportID(pub u32);

// these two refer to the same object
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) struct KernelPromiseID(pub u32);
#[derive(Debug, Eq, PartialEq, Hash)]
pub(crate) struct KernelResolverID(pub u32);

/// "KernelExport" is the kernel's representation of a pass-by-presence
/// object that has been exported by some Vat
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) struct KernelExport(pub VatID, pub KernelExportID);

/// "KernelTarget" is the kernel's representation of something which can be
/// the target of a message send: either a KernelExport or a KernelPromise.
/// This happens to be the same type as KernelArgSlot.
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) enum KernelTarget {
    Export(KernelExport),
    Promise(KernelPromiseID),
}

/// "KernelArgSlot" is the kernel's representation of something which can be
/// an argument of a syscall.send or dispatch.deliver (or other methods).
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) enum KernelArgSlot {
    Export(KernelExport),
    Promise(KernelPromiseID),
}

impl fmt::Display for VatName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "vat-{}", self.0)
    }
}
impl fmt::Display for VatID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "vat{}", self.0)
    }
}
impl fmt::Display for KernelExportID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KernelExportID-{}", self.0)
    }
}

impl fmt::Display for KernelPromiseID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KernelPromiseID-{}", self.0)
    }
}

impl fmt::Display for KernelResolverID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KernelResolverID-{}", self.0)
    }
}

impl fmt::Display for KernelExport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KernelExport({}:{})", self.0, self.1)
    }
}

impl fmt::Display for KernelTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use KernelTarget::*;
        match self {
            Export(ke) => write!(f, "ktarget({}:{})", ke.0, ke.1),
            Promise(id) => write!(f, "ktarget(Promise-{})", id),
        }
    }
}

impl fmt::Display for KernelArgSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use KernelArgSlot::*;
        match self {
            Export(ke) => write!(f, "karg({}:{})", ke.0, ke.1),
            Promise(id) => write!(f, "karg(Promise-{})", id),
        }
    }
}

impl From<KernelPromiseID> for KernelArgSlot {
    fn from(id: KernelPromiseID) -> KernelArgSlot {
        KernelArgSlot::Promise(id)
    }
}
