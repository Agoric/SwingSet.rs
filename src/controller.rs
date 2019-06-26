//use std::fmt::Debug;
use super::config::Config;
use super::kernel::Kernel;
use super::kernel_types::{KernelExportID, VatName};

//#[derive(Debug)]
pub struct Controller {
    kernel: Kernel,
}

impl Controller {
    pub fn new(cfg: Config) -> Self {
        let kernel = Kernel::new(cfg.vats);
        Controller { kernel }
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
}
