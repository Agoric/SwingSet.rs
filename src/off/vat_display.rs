use super::syscall::{
    CapSlot, ExportID, ImportID, InboundTarget, LocalPromiseID, Promise, RemotePromiseID,
    Resolution,
};
use std::fmt;

impl fmt::Display for LocalPromiseID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "lp{}", self.0)
    }
}

impl fmt::Display for ExportID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "e{}", self.0)
    }
}

impl fmt::Display for RemotePromiseID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rp{}", self.0)
    }
}

impl fmt::Display for ImportID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "i{}", self.0)
    }
}

impl fmt::Display for CapSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CapSlot::*;
        match self {
            LocalPromise(id) => write!(f, "lp{}", id.0),
            Export(id) => write!(f, "e{}", id.0),
            RemotePromise(id) => write!(f, "rp{}", id.0),
            Import(id) => write!(f, "i{}", id.0),
        }
    }
}

impl fmt::Display for Promise {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Promise::*;
        match self {
            LocalPromise(id) => write!(f, "lp{}", id.0),
            RemotePromise(id) => write!(f, "rp{}", id.0),
        }
    }
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Resolution::*;
        match self {
            Reference(slot) => write!(f, "ref-{}", slot),
            Data(_) => write!(f, "data-.."),
            Rejection(_) => write!(f, "reject-.."),
        }
    }
}

impl fmt::Display for InboundTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use InboundTarget::*;
        match self {
            LocalPromise(id) => write!(f, "lp{}", id.0),
            RemotePromise(id) => write!(f, "rp{}", id.0),
            Export(id) => write!(f, "e{}", id.0),
        }
    }
}
