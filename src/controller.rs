//use std::fmt::Debug;
use super::config::Config;
use super::kernel::Kernel;
use super::kernel_types::{KernelCapData, KernelExportID, KernelMessage, VatName};

//#[derive(Debug)]
pub struct Controller {
    kernel: Kernel,
}

impl Controller {
    pub fn new(cfg: Config) -> Self {
        let kernel = Kernel::new(cfg);
        Controller { kernel }
    }

    pub fn add_import(
        &mut self,
        for_vat: &VatName,
        for_id: u32,
        to_vat: &VatName,
        to_id: u32,
    ) {
        self.kernel.add_import(for_vat, for_id, to_vat, to_id);
    }

    pub fn start(&mut self) {
        self.kernel.push(
            &VatName("bootstrap".to_string()),
            KernelExportID(0),
            KernelMessage {
                name: "bootstrap".to_string(),
                args: KernelCapData {
                    body: vec![],
                    slots: vec![],
                },
                resolver: None,
            },
        );
    }

    pub fn push(&mut self, vat_name: &str, target: u32, method: &str, args_body: &[u8]) {
        self.kernel.push(
            &VatName(vat_name.to_string()),
            KernelExportID(target),
            KernelMessage {
                name: method.to_string(),
                args: KernelCapData {
                    body: args_body.to_vec(),
                    slots: vec![],
                },
                resolver: None,
            },
        );
    }

    pub fn step(&mut self) {
        println!("controller.step");
        self.kernel.step();
    }

    pub fn run(&mut self) {
        println!("controller.run");
        self.kernel.run();
    }

    pub fn dump(&self) {
        self.kernel.dump();
    }
}
