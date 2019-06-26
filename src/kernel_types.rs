use std::fmt;

#[derive(PartialEq, Eq, Debug, Hash)]
pub struct VatName(pub String);
impl fmt::Display for VatName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct KernelExportID(pub u32);
impl fmt::Display for KernelExportID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KernelExportID-{}", self.0)
    }
}

// these two refer to the same object
#[derive(Debug)]
pub(crate) struct KernelPromiseID(pub u32);
impl fmt::Display for KernelPromiseID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KernelPromiseID-{}", self.0)
    }
}

#[derive(Debug)]
pub(crate) struct KernelResolverID(pub u32);
impl fmt::Display for KernelResolverID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KernelResolverID-{}", self.0)
    }
}

#[derive(Debug)]
pub(crate) enum Target {
    Export(VatName, KernelExportID),
    Promise(KernelPromiseID),
}
impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Target::Export(vn, id) => write!(f, "Target({}:{})", vn, id),
            Target::Promise(id) => write!(f, "Target(Promise-{})", id),
        }
    }
}
