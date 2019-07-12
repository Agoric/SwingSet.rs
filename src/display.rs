use std::fmt;
use super::syscall::{LocalPromise, SendResult, Export, DispatchResult, RemotePromise, Import,
                     CapSlot, OutboundTarget, RemotelyResolvable, LocallyResolvable, InboundTarget};

impl fmt::Display for LocalPromise {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "lp{}", self.0)
    }
}

impl fmt::Display for SendResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "sr{}", self.0)
    }
}

impl fmt::Display for Export {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "e{}", self.0)
    }
}

impl fmt::Display for DispatchResult {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "dr{}", self.0)
    }
}

impl fmt::Display for RemotePromise {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "rp{}", self.0)
    }
}

impl fmt::Display for Import {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "i{}", self.0)
    }
}

impl fmt::Display for CapSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CapSlot::*;
        match self {
            LocalPromise(id) => write!(f, "lp{}", id.0),
            SendResult(id) => write!(f, "sr{}", id.0),
            Export(id) => write!(f, "e{}", id.0),
            DispatchResult(id) -> write!(f, "dr{}", id.0),
            RemotePromise(id) -> write!(f, "rp{}", id.0),
            Import(id) => write!(f, "i{}", id.0),
        }
    }
}

impl fmt::Display for OutboundTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use OutboundTarget::*;
        match self {
            SendResult(id) => write!(f, "sr{}", id.0),
            RemotePromise(id) -> write!(f, "rp{}", id.0),
            Import(id) => write!(f, "i{}", id.0),
        }
    }
}

impl fmt::Display for RemotelyResolvable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CapSlot::*;
        match self {
            SendResult(id) => write!(f, "sr{}", id.0),
            RemotePromise(id) -> write!(f, "rp{}", id.0),
        }
    }
}

impl fmt::Display for LocallyResolvable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CapSlot::*;
        match self {
            LocalPromise(id) => write!(f, "lp{}", id.0),
            DispatchResult(id) -> write!(f, "dr{}", id.0),
        }
    }
}

impl fmt::Display for InboundTarget {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CapSlot::*;
        match self {
            LocalPromise(id) => write!(f, "lp{}", id.0),
            Export(id) => write!(f, "e{}", id.0),
            DispatchResult(id) -> write!(f, "dr{}", id.0),
        }
    }
}
