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
    _Promise(VatPromiseID),
}

pub trait Syscall {
    fn send(&mut self, target: VatSendTarget, name: &str) -> VatPromiseID;
}

#[derive(Debug)]
pub struct VatSyscall {}
impl VatSyscall {
    pub fn new() -> Self {
        VatSyscall {}
    }
}
impl Syscall for VatSyscall {
    fn send(&mut self, _target: VatSendTarget, _name: &str) -> VatPromiseID {
        VatPromiseID(1)
    }
}

pub trait Dispatch {
    fn deliver(&self, syscall: &mut Box<dyn Syscall>, target: VatExportID) -> ();
    fn notify_resolve_to_target(&self, id: VatPromiseID, target: u8);
}
