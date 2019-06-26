use super::vat::{Dispatch, Syscall, VatExportID, VatSyscall};
use super::vatname::VatName;
use std::collections::{HashMap, VecDeque};
use std::fmt;

#[derive(Debug)]
pub struct KernelExportID(pub u32);
impl fmt::Display for KernelExportID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KernelExportID-{}", self.0)
    }
}

// these two refer to the same object
#[derive(Debug)]
struct KernelPromiseID(u32);
impl fmt::Display for KernelPromiseID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KernelPromiseID-{}", self.0)
    }
}

#[derive(Debug)]
struct KernelResolverID(u32);
impl fmt::Display for KernelResolverID {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "KernelResolverID-{}", self.0)
    }
}

#[derive(Debug)]
enum Target {
    Export(VatName, KernelExportID),
    Promise(KernelPromiseID),
}
impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Target::Export(vn, id) => write!(f, "Target({}:{})", vn, id),
            Target::Promise(id) => write!(f, "Target(Promise-{})", id),
        }
    }
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

    pub fn _add_vat(&mut self, name: &VatName, dispatch: impl Dispatch + 'static) {
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

    fn map_export_target(&self, id: KernelExportID) -> VatExportID {
        VatExportID(id.0)
    }

    fn _map_inbound(&mut self, _vn: &VatName, id: KernelExportID) -> VatExportID {
        // todo clist mapping
        VatExportID(id.0)
    }

    fn process(&mut self, pd: PendingDelivery) {
        let t = pd.target;
        println!("process: {}.{}", t, pd.method);
        match t {
            Target::Export(vn, kid) => {
                //let vid = self.map_inbound(&vn, kid);
                let vid = self.map_export_target(kid);
                let dispatch = self.vats.get(&vn).unwrap();
                let mut syscall: Box<dyn Syscall> = Box::new(VatSyscall::new());
                dispatch.deliver(&mut syscall, vid);
            }
            //Target::Promise(_pid) => {}
            _ => panic!(),
        };
    }

    pub fn step(&mut self) {
        println!("kernel.step");
        match self.run_queue.pop_front() {
            Some(pd) => self.process(pd),
            None => (),
        };
    }

    pub fn run(&mut self) {
        println!("kernel.run");
    }
}
