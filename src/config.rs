use super::vat::Dispatch;
use super::vatname::VatName;
use std::collections::HashMap;

/*#[derive(PartialEq, Eq, Debug, Hash)]
pub struct DeviceName(pub String);
#[derive(Debug)]
pub struct DeviceSetup(pub Fn(impl Syscall) -> impl Dispatch);*/

//#[derive(Debug)]
pub struct Config {
    pub(crate) vats: HashMap<VatName, Box<dyn Dispatch>>,
    //devices: HashMap<DeviceName, DeviceSetup>,
}
impl Config {
    pub fn new() -> Self {
        Config {
            vats: HashMap::new(),
        }
    }
    pub fn add_vat(&mut self, name: &VatName, dispatch: impl Dispatch + 'static) {
        let vn = VatName(name.0.clone());
        self.vats.insert(vn, Box::new(dispatch));
    }
}
