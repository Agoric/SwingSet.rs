use super::kernel_types::{
    KernelExport, KernelExportID, KernelPromiseID, KernelResolverID, KernelTarget,
    VatName,
};
use super::vat::{Dispatch, VatManager, VatSyscall};
use super::vat_types::{VatExportID, VatImportID, VatPromiseID, VatSendTarget};
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};

#[derive(Debug)]
pub struct PendingDelivery {
    target: KernelTarget,
    method: String,
    args: u8,
    resolver: KernelResolverID,
}
impl PendingDelivery {
    pub(crate) fn new(
        target: KernelTarget,
        method: &str,
        args: u8,
        resolver: KernelResolverID,
    ) -> Self {
        PendingDelivery {
            target,
            method: method.to_string(),
            args,
            resolver,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct CList {
    pub inbound_imports: HashMap<KernelExport, VatImportID>,
    pub outbound_imports: HashMap<VatImportID, KernelExport>,
    pub inbound_promises: HashMap<KernelPromiseID, VatPromiseID>,
    pub outbound_promises: HashMap<VatPromiseID, KernelPromiseID>,
}
impl CList {
    /*pub fn _map_outbound<T: Into<VatArgSlot>>(&self, target: T) -> KernelArgSlot {
        let t = self.outbound.get(&target.into()).unwrap();
        (*t).clone()
    }*/

    pub fn map_outbound_target(&self, target: VatSendTarget) -> KernelTarget {
        match target {
            VatSendTarget::Import(viid) => {
                let ke = self.outbound_imports.get(&viid).unwrap();
                KernelTarget::Export(ke.clone())
            }
            VatSendTarget::Promise(vpid) => {
                let kpid = self.outbound_promises.get(&vpid).unwrap();
                KernelTarget::Promise(kpid.clone())
            }
        }
    }

    pub fn map_inbound_promise(&mut self, kpid: KernelPromiseID) -> VatPromiseID {
        if let Some(vpid) = self.inbound_promises.get(&kpid) {
            vpid.clone()
        } else {
            let vpid = VatPromiseID(4); // TODO allocate
            self.inbound_promises.insert(kpid.clone(), vpid.clone());
            self.outbound_promises.insert(vpid.clone(), kpid);
            vpid
        }
    }
}

pub struct VatData {
    clist: CList,
    dispatch: Box<dyn Dispatch>,
}

#[derive(Debug, Default)]
pub struct RunQueue(pub VecDeque<PendingDelivery>);

//#[derive(Debug)]
pub struct Kernel {
    vats: HashMap<VatName, VatData>,
    run_queue: RunQueue,
    next_promise_resolver_id: Cell<u32>,
}

impl Kernel {
    pub fn new(vats: HashMap<VatName, Box<dyn Dispatch>>) -> Self {
        let mut kvats = <HashMap<VatName, VatData>>::new();
        for (key, dispatch) in vats {
            kvats.insert(
                VatName(key.to_string()),
                VatData {
                    clist: CList::default(),
                    dispatch,
                },
            );
        }
        Kernel {
            vats: kvats,
            run_queue: RunQueue::default(),
            next_promise_resolver_id: Cell::new(0),
        }
    }

    pub fn _add_vat(&mut self, name: &VatName, dispatch: impl Dispatch + 'static) {
        let vn = VatName(name.0.clone());
        self.vats.insert(
            vn,
            VatData {
                clist: CList::default(),
                dispatch: Box::new(dispatch),
            },
        );
    }

    pub(crate) fn add_import(
        &mut self,
        for_vat: &VatName,
        for_id: VatImportID,
        to: KernelExport,
    ) {
        let clist = &mut self.vats.get_mut(&for_vat).unwrap().clist;
        clist.inbound_imports.insert(to.clone(), for_id.clone());
        clist.outbound_imports.insert(for_id, to);
    }

    fn allocate_promise_resolver_pair(&self) -> (KernelPromiseID, KernelResolverID) {
        let id = self.next_promise_resolver_id.get();
        let next_id = id + 1;
        self.next_promise_resolver_id.set(next_id);
        (KernelPromiseID(id), KernelResolverID(id))
    }

    pub fn push(&mut self, name: &VatName, export: KernelExportID, method: String) {
        let (_pid, rid) = self.allocate_promise_resolver_pair();
        let pd = PendingDelivery {
            target: KernelTarget::Export(KernelExport(VatName(name.0.clone()), export)),
            method,
            args: 0,
            resolver: rid,
        };
        self.run_queue.0.push_back(pd);
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
            KernelTarget::Export(KernelExport(vn, kid)) => {
                //let vid = self.map_inbound(&vn, kid);
                let vid = self.map_export_target(kid);
                //let VatData{ clist, dispatch } = self.vats.get_mut(&vn).unwrap();
                let vd = self.vats.get_mut(&vn).unwrap();
                let nprid = &self.next_promise_resolver_id;
                let allocate_promise_resolver_pair = || {
                    let id = nprid.get();
                    let next_id = id + 1;
                    nprid.set(next_id);
                    (KernelPromiseID(id), KernelResolverID(id))
                };
                let vm = VatManager {
                    run_queue: &mut self.run_queue,
                    clist: &mut vd.clist,
                    allocate_promise_resolver_pair: &allocate_promise_resolver_pair,
                };
                let mut syscall = VatSyscall::new(vm);
                vd.dispatch.deliver(&mut syscall, vid);
            }
            //KernelTarget::Promise(_pid) => {}
            _ => panic!(),
        };
    }

    pub fn step(&mut self) {
        println!("kernel.step");
        if let Some(pd) = self.run_queue.0.pop_front() {
            self.process(pd);
        }
    }

    pub fn run(&mut self) {
        println!("kernel.run");
    }

    pub fn dump(&self) {
        println!("Kernel Dump:");
        println!(" run-queue:");
        for pd in &self.run_queue.0 {
            println!("  {:?}", pd);
        }
    }
}
