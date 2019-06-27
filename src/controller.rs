//use std::fmt::Debug;
use super::config::Config;
use super::kernel::Kernel;
use super::kernel_types::{KernelExport, KernelExportID, VatName};
use super::vat_types::VatImportID;

//#[derive(Debug)]
pub struct Controller {
    kernel: Kernel,
}

impl Controller {
    pub fn new(cfg: Config) -> Self {
        let kernel = Kernel::new(cfg.vats);
        Controller { kernel }
    }

    pub fn add_import(
        &mut self,
        for_vat: &VatName,
        for_id: u32,
        to_vat: &VatName,
        to_id: u32,
    ) {
        self.kernel.add_import(
            for_vat,
            VatImportID(for_id),
            KernelExport(to_vat.clone(), KernelExportID(to_id)),
        );
    }

    pub fn start(&mut self) {
        self.kernel.push(
            &VatName("bootstrap".to_string()),
            KernelExportID(0),
            "bootstrap".to_string(),
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
