use super::vat::Dispatch;
use super::vatname::VatName;
use std::collections::{HashMap, VecDeque};

pub struct KernelExportID(pub u32);
// these two refer to the same object
struct KernelPromiseID(u32);
struct KernelResolverID(u32);

enum Target {
    Export(VatName, KernelExportID),
    Promise(KernelPromiseID),
}

struct PendingDelivery {
    target: Target,
    method: String,
    args: u8,
    resolver: KernelResolverID,
}

//#[derive(Debug)]
pub struct Kernel {
    vats: HashMap<VatName, Box<dyn Dispatch>>,
    run_queue: VecDeque<PendingDelivery>,
    next_promise_resolver_id: u32,
}

impl Kernel {
    pub fn new(vats: HashMap<VatName, Box<dyn Dispatch>>) -> Self {
        Kernel {
            vats,
            run_queue: VecDeque::new(),
            next_promise_resolver_id: 0,
        }
    }

    pub fn add_vat(&mut self, name: &VatName, dispatch: impl Dispatch + 'static) {
        let vn = VatName(name.0.clone());
        self.vats.insert(vn, Box::new(dispatch));
    }

    fn allocate_promise_resolver_pair(&mut self) -> (KernelPromiseID, KernelResolverID) {
        let id = self.next_promise_resolver_id;
        self.next_promise_resolver_id += 1;
        (KernelPromiseID(id), KernelResolverID(id))
    }

    pub fn push(&mut self, name: &VatName, export: KernelExportID, method: String) {
        let (_pid, rid) = self.allocate_promise_resolver_pair();
        let pd = PendingDelivery {
            target: Target::Export(VatName(name.0.clone()), export),
            method: method,
            args: 0,
            resolver: rid,
        };
        self.run_queue.push_back(pd);
    }

    pub fn step(&mut self) {
        println!("kernel.step");
    }

    pub fn run(&mut self) {
        println!("kernel.run");
    }
}
