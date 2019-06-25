#[derive(Debug)]
pub struct VatPromiseID(pub u32);

pub trait Syscall {
    fn send(&mut self, target: u8, name: u8) -> VatPromiseID;
}

#[derive(Debug)]
pub struct VatSyscall {}
impl Syscall for VatSyscall {
    fn send(&mut self, _target: u8, _name: u8) -> VatPromiseID {
        VatPromiseID(1)
    }
}

pub trait Dispatch {
    fn deliver(&mut self, syscall: &mut VatSyscall) -> ();
}
