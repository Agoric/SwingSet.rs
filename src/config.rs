use super::dispatch::Dispatch;
use super::kernel_types::VatName;
use super::syscall::Syscall;
use std::collections::HashMap;

/*#[derive(PartialEq, Eq, Debug, Hash)]
pub struct DeviceName(pub String);
#[derive(Debug)]
pub struct DeviceSetup(pub Fn(impl Syscall) -> impl Dispatch);*/

pub type Setup = FnOnce(Box<dyn Syscall>) -> Box<dyn Dispatch>;
#[derive(Default)]
pub struct Config {
    pub(crate) vats: HashMap<VatName, Box<Setup>>,
    //devices: HashMap<DeviceName, DeviceSetup>,
}
impl Config {
    pub fn new() -> Self {
        Config::default()
    }
    pub fn add_vat(&mut self, name: &VatName, setup: Box<Setup>) {
        let vn = VatName(name.0.clone());
        self.vats.insert(vn, setup);
    }
}
