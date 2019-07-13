use super::kernel_types::{CapSlot, Resolution, VatID, VatName};
use std::fmt;

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

impl fmt::Display for CapSlot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use CapSlot::*;
        match self {
            Promise(id) => write!(f, "promise{}", id.0),
            Presence(id) => write!(f, "presence{}", id.0),
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
