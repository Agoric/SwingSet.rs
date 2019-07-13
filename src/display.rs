use std::fmt;
use super::promise::PromiseID;
use super::presence::PresenceID;

impl fmt::Display for PromiseID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "prom{}", self.0)
    }
}

impl fmt::Display for PresenceID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "pres{}", self.0)
    }
}
