use super::kernel_types::{
    KernelExport, KernelExportID, KernelPromiseID, KernelResolverID, KernelTarget, VatID,
    VatName,
};
use super::vat::{Dispatch, VatManager, VatSyscall};
use super::vat_types::{VatExportID, VatImportID, VatPromiseID};
use core::hash::Hash;
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

pub(crate) trait CListVatEntry: Eq + Hash + Clone {
    fn new(index: u32) -> Self;
}
impl CListVatEntry for VatImportID {
    fn new(index: u32) -> Self {
        VatImportID(index)
    }
}
impl CListVatEntry for VatPromiseID {
    fn new(index: u32) -> Self {
        VatPromiseID(index)
    }
}

pub(crate) trait CListKernelEntry: Eq + Hash + Clone {}
impl CListKernelEntry for KernelExport {}
impl CListKernelEntry for KernelPromiseID {}

#[derive(Debug, Default)]
pub(crate) struct CList<KT: CListKernelEntry, VT: CListVatEntry> {
    inbound: HashMap<KT, VT>,
    outbound: HashMap<VT, KT>,
    next_index: u32,
}
impl<KT: CListKernelEntry, VT: CListVatEntry> CList<KT, VT> {
    /*pub fn _map_outbound<T: Into<VatArgSlot>>(&self, target: T) -> KernelArgSlot {
        let t = self.outbound.get(&target.into()).unwrap();
        (*t).clone()
    }*/

    pub fn new() -> Self {
        CList {
            inbound: HashMap::new(),
            outbound: HashMap::new(),
            next_index: 0,
        }
    }

    /// vat objects being sent outbound (out of the vat and into the kernel)
    /// must already exist in the clist: this is how we confine vats to only
    /// use previously-granted authorities
    pub fn map_outbound(&self, vat_object: &VT) -> KT {
        (*self.outbound.get(&vat_object).unwrap()).clone()
    }

    /// kernel objects being sent inbound (from the kernel, into the vat)
    /// might already exist, or they might need to allocate new vat-side
    /// identifiers
    pub fn map_inbound(&mut self, kernel_object: &KT) -> VT {
        if let Some(vat_object) = self.inbound.get(&kernel_object) {
            vat_object.clone()
        } else {
            let vat_object = VT::new(self.next_index);
            self.next_index += 1;
            self.inbound
                .insert(kernel_object.clone(), vat_object.clone());
            self.outbound
                .insert(vat_object.clone(), kernel_object.clone());
            vat_object
        }
    }
}

pub(crate) struct VatData {
    pub(crate) import_clist: CList<KernelExport, VatImportID>,
    pub(crate) promise_clist: CList<KernelPromiseID, VatPromiseID>,
}

#[derive(Debug, Default)]
pub struct RunQueue(pub VecDeque<PendingDelivery>);

//#[derive(Debug)]
pub struct Kernel {
    vat_names: HashMap<VatName, VatID>,
    vat_dispatch: HashMap<VatID, Box<dyn Dispatch>>,
    vat_data: HashMap<VatID, VatData>,
    run_queue: RunQueue,
    next_promise_resolver_id: Cell<u32>,
}

impl Kernel {
    pub fn new(vats: HashMap<VatName, Box<dyn Dispatch>>) -> Self {
        let mut vat_names = <HashMap<VatName, VatID>>::new();
        let mut vat_dispatch = <HashMap<VatID, Box<dyn Dispatch>>>::new();
        let mut vat_data = <HashMap<VatID, VatData>>::new();
        let mut id = 0;
        for (key, dispatch) in vats {
            let vat_id = VatID(id);
            id += 1;
            vat_names.insert(VatName(key.0.clone()), vat_id);
            vat_dispatch.insert(vat_id, dispatch);
            let vd = VatData {
                import_clist: CList::new(),
                promise_clist: CList::new(),
            };
            vat_data.insert(vat_id, vd);
        }
        Kernel {
            vat_names,
            vat_dispatch,
            vat_data,
            run_queue: RunQueue::default(),
            next_promise_resolver_id: Cell::new(0),
        }
    }

    /*
    pub fn _add_vat(&mut self, name: &VatName, dispatch: impl Dispatch + 'static) {
        self.vat_dispatch
            .insert(VatName(name.0.clone()), Box::new(dispatch));
        self.vat_data.insert(
            VatName(name.0.clone()),
            VatData {
                import_clist: CList::new(),
                promise_clist: CList::new(),
            },
        );
    }
    */

    pub(crate) fn add_import(
        &mut self,
        for_vat: &VatName,
        for_id: u32,
        to_vat: &VatName,
        to_id: u32,
    ) {
        let for_vat_id = self.vat_names.get(&for_vat).unwrap();
        let to_vat_id = self.vat_names.get(&to_vat).unwrap();
        let to = KernelExport(*to_vat_id, KernelExportID(to_id));
        let clist = &mut self.vat_data.get_mut(for_vat_id).unwrap().import_clist;
        clist.inbound.insert(to.clone(), VatImportID(for_id));
        clist.outbound.insert(VatImportID(for_id), to);
    }

    fn allocate_promise_resolver_pair(&self) -> (KernelPromiseID, KernelResolverID) {
        let id = self.next_promise_resolver_id.get();
        let next_id = id + 1;
        self.next_promise_resolver_id.set(next_id);
        (KernelPromiseID(id), KernelResolverID(id))
    }

    pub fn push(&mut self, name: &VatName, export: KernelExportID, method: String) {
        let vat_id = self.vat_names.get(&name).unwrap();
        let (_pid, rid) = self.allocate_promise_resolver_pair();
        let pd = PendingDelivery {
            target: KernelTarget::Export(KernelExport(*vat_id, export)),
            method,
            args: 0,
            resolver: rid,
        };
        self.run_queue.0.push_back(pd);
    }

    /// exports return home with the same index
    fn map_export_target(&self, id: KernelExportID) -> VatExportID {
        VatExportID(id.0)
    }

    fn _map_inbound(&mut self, _vn: &VatName, id: KernelExportID) -> VatExportID {
        // todo clist mapping
        //let vat_id = self.vat_names.get(&vn).unwrap();
        VatExportID(id.0)
    }

    fn process(&mut self, pd: PendingDelivery) {
        let t = pd.target;
        println!("process: {}.{}", t, pd.method);
        match t {
            KernelTarget::Export(KernelExport(vat_id, kid)) => {
                let vid = self.map_export_target(kid);
                let dispatch = self.vat_dispatch.get_mut(&vat_id).unwrap();
                //let VatData{ clist, dispatch } = self.vats.get_mut(&vat_id).unwrap();
                let mut vd = self.vat_data.get_mut(&vat_id).unwrap();
                let nprid = &self.next_promise_resolver_id;
                let allocate_promise_resolver_pair = || {
                    let id = nprid.get();
                    let next_id = id + 1;
                    nprid.set(next_id);
                    (KernelPromiseID(id), KernelResolverID(id))
                };
                let vm = VatManager {
                    run_queue: &mut self.run_queue,
                    vat_data: &mut vd,
                    allocate_promise_resolver_pair: &allocate_promise_resolver_pair,
                };
                let mut syscall = VatSyscall::new(vm);
                dispatch.deliver(&mut syscall, vid);
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
