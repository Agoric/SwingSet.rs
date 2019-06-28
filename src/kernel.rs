use super::clist::{CList, CListKernelEntry, CListVatEntry};
use super::dispatch::Dispatch;
use super::kernel_types::{
    KernelArgSlot, KernelExport, KernelExportID, KernelMessage, KernelPromiseID,
    KernelResolverID, KernelTarget, VatID, VatName,
};
use super::vat::{VatManager, VatSyscall};
use super::vat_types::{
    VatArgSlot, VatCapData, VatExportID, VatImportID, VatMessage, VatPromiseID,
    VatResolverID,
};
use std::cell::Cell;
use std::collections::{HashMap, VecDeque};

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
impl CListVatEntry for VatResolverID {
    fn new(index: u32) -> Self {
        VatResolverID(index)
    }
}
impl CListKernelEntry for KernelExport {}
impl CListKernelEntry for KernelPromiseID {}
impl CListKernelEntry for KernelResolverID {}

#[derive(Debug)]
pub struct PendingDelivery {
    target: KernelTarget,
    message: KernelMessage,
    resolver: Option<KernelResolverID>,
}
impl PendingDelivery {
    pub(crate) fn new(
        target: KernelTarget,
        message: KernelMessage,
        resolver: Option<KernelResolverID>,
    ) -> Self {
        PendingDelivery {
            target,
            message,
            resolver,
        }
    }
}

pub(crate) struct VatData {
    vat_id: VatID,
    pub(crate) import_clist: CList<KernelExport, VatImportID>,
    pub(crate) promise_clist: CList<KernelPromiseID, VatPromiseID>,
    pub(crate) resolver_clist: CList<KernelResolverID, VatResolverID>,
}
impl VatData {
    pub fn map_inbound_arg_slot(&mut self, slot: KernelArgSlot) -> VatArgSlot {
        match slot {
            KernelArgSlot::Export(ke) => {
                let vat_id: VatID = ke.0;
                if vat_id == self.vat_id {
                    // the vat's own export, returning home
                    let keid: KernelExportID = ke.1;
                    VatArgSlot::Import(VatImportID(keid.0))
                } else {
                    // another vat's export, get/allocate in clist
                    VatArgSlot::Import(self.import_clist.map_inbound(ke))
                }
            }
            KernelArgSlot::Promise(kp) => {
                VatArgSlot::Promise(self.promise_clist.map_inbound(kp))
            }
        }
    }

    pub fn map_inbound_resolver(&mut self, krid: KernelResolverID) -> VatResolverID {
        self.resolver_clist.map_inbound(krid)
    }
    pub fn map_outbound_resolver(&mut self, vrid: VatResolverID) -> KernelResolverID {
        self.resolver_clist.map_outbound(vrid)
    }
}

#[derive(Debug, Default)]
pub struct RunQueue(pub VecDeque<PendingDelivery>);

pub struct KernelData {
    vat_names: HashMap<VatName, VatID>,
    vat_dispatch: HashMap<VatID, Box<dyn Dispatch>>,
    vat_data: HashMap<VatID, VatData>,
    run_queue: RunQueue,
    next_promise_resolver_id: Cell<u32>,
}

//#[derive(Debug)]
pub struct Kernel {
    kd: KernelData,
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
                vat_id,
                import_clist: CList::new(),
                promise_clist: CList::new(),
                resolver_clist: CList::new(),
            };
            vat_data.insert(vat_id, vd);
        }
        Kernel {
            kd: KernelData {
                vat_names,
                vat_dispatch,
                vat_data,
                run_queue: RunQueue::default(),
                next_promise_resolver_id: Cell::new(0),
            },
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
        let for_vat_id = self.kd.vat_names.get(&for_vat).unwrap();
        let to_vat_id = *self.kd.vat_names.get(&to_vat).unwrap();
        let clist = &mut self.kd.vat_data.get_mut(for_vat_id).unwrap().import_clist;
        clist.add(
            KernelExport(to_vat_id, KernelExportID(to_id)),
            VatImportID(for_id),
        );
    }

    fn allocate_promise_resolver_pair(&self) -> (KernelPromiseID, KernelResolverID) {
        let id = self.kd.next_promise_resolver_id.get();
        let next_id = id + 1;
        self.kd.next_promise_resolver_id.set(next_id);
        (KernelPromiseID(id), KernelResolverID(id))
    }

    pub(crate) fn push(
        &mut self,
        name: &VatName,
        export: KernelExportID,
        message: KernelMessage,
    ) {
        let vat_id = *self.kd.vat_names.get(&name).unwrap();
        let (_pid, rid) = self.allocate_promise_resolver_pair();
        let pd = PendingDelivery {
            target: KernelTarget::Export(KernelExport(vat_id, export)),
            message,
            resolver: Some(rid),
        };
        self.kd.run_queue.0.push_back(pd);
    }

    /// exports return home with the same index
    fn map_inbound_target(&self, id: KernelExportID) -> VatExportID {
        VatExportID(id.0)
    }

    fn process(&mut self, pd: PendingDelivery) {
        let t = pd.target;
        println!("process: {}.{}", t, pd.message.name);
        match t {
            KernelTarget::Export(KernelExport(vat_id, kid)) => {
                let veid = self.map_inbound_target(kid);
                let mut vd = self.kd.vat_data.get_mut(&vat_id).unwrap();
                let kmsg = pd.message;
                let vmsg = VatMessage {
                    name: kmsg.name,
                    args: VatCapData {
                        body: kmsg.args.body,
                        slots: kmsg
                            .args
                            .slots
                            .into_iter()
                            .map(|slot| vd.map_inbound_arg_slot(slot))
                            .collect(),
                    },
                };
                let vrid: Option<VatResolverID> = match pd.resolver {
                    Some(krid) => Some(vd.map_inbound_resolver(krid)),
                    None => None,
                };

                //let VatData{ clist, dispatch } = self.kd.vats.get_mut(&vat_id).unwrap();
                let nprid = &self.kd.next_promise_resolver_id;
                let allocate_promise_resolver_pair = || {
                    let id = nprid.get();
                    let next_id = id + 1;
                    nprid.set(next_id);
                    (KernelPromiseID(id), KernelResolverID(id))
                };
                let vm = VatManager {
                    vat_id,
                    run_queue: &mut self.kd.run_queue,
                    vat_data: &mut vd,
                    allocate_promise_resolver_pair: &allocate_promise_resolver_pair,
                };
                let mut syscall = VatSyscall::new(vm);
                let dispatch = self.kd.vat_dispatch.get_mut(&vat_id).unwrap();
                dispatch.deliver(&mut syscall, veid, vmsg, vrid);
            }
            //KernelTarget::Promise(_pid) => {}
            _ => panic!(),
        };
    }

    pub fn step(&mut self) {
        println!("kernel.step");
        if let Some(pd) = self.kd.run_queue.0.pop_front() {
            self.process(pd);
        }
    }

    pub fn run(&mut self) {
        println!("kernel.run");
    }

    pub fn dump(&self) {
        println!("Kernel Dump:");
        println!(" run-queue:");
        for pd in &self.kd.run_queue.0 {
            println!("  {:?}", pd);
        }
    }
}
