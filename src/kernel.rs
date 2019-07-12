use super::clist::{CList, CListKernelEntry, CListVatEntry};
use super::config::Config;
use super::dispatch::Dispatch;
use super::kernel_types::{
    KernelArgSlot, KernelCapData, KernelExport, KernelExportID, KernelMessage,
    KernelPromiseResolverID, VatID, VatName,
};
use super::promise::KernelPromise;
use super::vat::VatSyscall;
use super::vat_types::{
    InboundVatMessage, VatArgSlot, VatCapData, VatExportID, VatImportID, VatPromiseID,
    VatResolveTarget, VatResolverID,
};
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

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
impl CListKernelEntry for KernelPromiseResolverID {}

#[derive(Debug)]
pub(crate) enum PendingDelivery {
    Deliver {
        target: KernelExport,
        message: KernelMessage,
    },
    DeliverPromise {
        vat_id: VatID,
        target: KernelPromiseResolverID,
        message: KernelMessage,
    },
    NotifyFulfillToData {
        vat_id: VatID,
        target: KernelPromiseResolverID,
        data: KernelCapData,
    },
    NotifyFulfillToTarget {
        vat_id: VatID,
        target: KernelPromiseResolverID,
        result: KernelExport,
    },
    NotifyReject {
        vat_id: VatID,
        target: KernelPromiseResolverID,
        data: KernelCapData,
    },
}

pub(crate) struct VatData {
    vat_id: VatID,
    pub(crate) import_clist: CList<KernelExport, VatImportID>,
    pub(crate) local_promise_clist: CList<KernelPromiseResolverID, VatPromiseID>,
    pub(crate) remote_promise_clist: CList<KernelPromiseResolverID, VatResolverID>,
}
impl VatData {
    // it's totally legit for vat A to hold a promise, vat B resolves
    // it to one of vat A's exports. Or to vatB's exports, or somebody else's
    // export. So the 'result' in vatA.notify_fulfill_to_target is either a
    // VatExportID or a VatImportID, and we need a new enum to hold that.
    fn map_inbound_resolve_target(&mut self, ktarget: KernelExport) -> VatResolveTarget {
        if ktarget.0 == self.vat_id {
            // the vat's own export, returning home
            let keid: KernelExportID = ktarget.1;
            VatResolveTarget::Export(VatExportID(keid.0))
        } else {
            // another vat's export, get/allocate in clist
            let veid = self.import_clist.map_inbound(ktarget);
            VatResolveTarget::Import(veid)
        }
    }

    pub fn map_outbound_resolve_target(
        &mut self,
        vtarget: VatResolveTarget,
    ) -> KernelExport {
        match vtarget {
            VatResolveTarget::Export(VatExportID(id)) => {
                KernelExport(self.vat_id, KernelExportID(id))
            }
            VatResolveTarget::Import(viid) => self.import_clist.map_outbound(viid),
        }
    }

    pub fn map_inbound_arg_slot(&mut self, slot: KernelArgSlot) -> VatArgSlot {
        match slot {
            KernelArgSlot::Export(ke) => {
                let vat_id: VatID = ke.0;
                if vat_id == self.vat_id {
                    // the vat's own export, returning home
                    let keid: KernelExportID = ke.1;
                    VatArgSlot::Export(VatExportID(keid.0))
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

    pub fn map_inbound_promise(
        &mut self,
        kprid: KernelPromiseResolverID,
    ) -> VatPromiseID {
        self.promise_clist.map_inbound(kprid)
    }
    pub fn get_outbound_promise(
        &mut self,
        vpid: VatPromiseID,
    ) -> KernelPromiseResolverID {
        self.promise_clist.get_outbound(vpid)
    }

    pub fn get_inbound_resolver(
        &mut self,
        krid: KernelPromiseResolverID,
    ) -> VatResolverID {
        self.resolver_clist.get_inbound(krid)
    }

    pub fn map_inbound_resolver(
        &mut self,
        krid: KernelPromiseResolverID,
    ) -> VatResolverID {
        self.resolver_clist.map_inbound(krid)
    }
    pub fn map_outbound_resolver(
        &mut self,
        vrid: VatResolverID,
    ) -> KernelPromiseResolverID {
        self.resolver_clist.map_outbound(vrid)
    }

    pub fn forward_promise(
        &mut self,
        old_id: KernelPromiseResolverID,
        new_id: KernelPromiseResolverID,
    ) {
        let pc = &mut self.promise_clist;
        if pc.inbound.contains_key(&old_id) {
            let vpid = *pc.inbound.get(&old_id).unwrap();
            assert!(pc.outbound.contains_key(&vpid));
            pc.inbound.remove(&old_id);
            pc.inbound.insert(new_id, vpid);
            pc.outbound.insert(vpid, new_id);
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct RunQueue(pub VecDeque<PendingDelivery>);

pub(crate) struct KernelData {
    pub(crate) vat_names: HashMap<VatName, VatID>,
    pub(crate) vat_data: HashMap<VatID, VatData>,
    pub(crate) run_queue: RunQueue,
    pub(crate) next_promise_id: usize,
    pub(crate) presences: PresenceTable,
    pub(crate) promises: HashMap<KernelPromiseID, KernelPromise>,
}

//#[derive(Debug)]
pub struct Kernel {
    pub(crate) vat_dispatch: HashMap<VatID, Box<dyn Dispatch>>,
    pub(crate) kd: Rc<RefCell<KernelData>>,
}

impl Kernel {
    pub fn new(cfg: Config) -> Self {
        let mut vat_dispatch = HashMap::new();
        let kd = Rc::new(RefCell::new(KernelData {
            vat_names: HashMap::new(),
            vat_data: HashMap::new(),
            run_queue: RunQueue::default(),
            promises: HashMap::default(),
            next_promise_resolver_id: 0,
        }));
        let mut id = 0;
        for (key, setup) in cfg.vats {
            let vat_id = VatID(id);
            id += 1;
            kd.borrow_mut()
                .vat_names
                .insert(VatName(key.0.clone()), vat_id);
            let vd = VatData {
                vat_id,
                import_clist: CList::new(),
                promise_clist: CList::new(),
                resolver_clist: CList::new(),
            };
            kd.borrow_mut().vat_data.insert(vat_id, vd);
            let syscall = VatSyscall::new(vat_id, kd.clone());
            let dispatch = setup(Box::new(syscall));
            vat_dispatch.insert(vat_id, dispatch);
        }
        Kernel { vat_dispatch, kd }
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
        // TODO: even though this method is only for setting up unit tests,
        // let's make sure this doesn't conflict with any pre-existing
        // mapping, and let's add code to clist.allocate so some future
        // allocation doesn't conflict with the one we add now (e.g. a loop()
        // that keeps trying higher numbers until it finds a free one)
        let mut kd = self.kd.borrow_mut();
        let for_vat_id = *kd.vat_names.get(&for_vat).unwrap();
        let to_vat_id = *kd.vat_names.get(&to_vat).unwrap();
        kd.vat_data.get_mut(&for_vat_id).unwrap().import_clist.add(
            KernelExport(to_vat_id, KernelExportID(to_id)),
            VatImportID(for_id),
        );
    }

    pub(crate) fn push(
        &mut self,
        name: &VatName,
        export: KernelExportID,
        message: KernelMessage,
    ) {
        let mut kd = self.kd.borrow_mut();
        let vat_id = *kd.vat_names.get(&name).unwrap();
        let pd = PendingDelivery::Deliver {
            target: KernelExport(vat_id, export),
            message,
        };
        kd.run_queue.0.push_back(pd);
    }

    /// exports return home with the same index
    fn map_inbound_target(&self, id: KernelExportID) -> VatExportID {
        VatExportID(id.0)
    }

    fn process(&mut self, pd: PendingDelivery) {
        match pd {
            PendingDelivery::Deliver {
                target,
                message: kmsg,
            } => {
                let vat_id = target.0; // TODO nicer destructuring assignment
                let kid = target.1;
                println!("process.Deliver: {}.{}", target, kmsg.name);
                let veid = self.map_inbound_target(kid);
                let vmsg = {
                    let mut kd = self.kd.borrow_mut();
                    let vd = kd.vat_data.get_mut(&vat_id).unwrap();
                    let ovrid: Option<VatResolverID> = match kmsg.resolver {
                        Some(krid) => Some(vd.map_inbound_resolver(krid)),
                        None => None,
                    };
                    InboundVatMessage {
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
                        resolver: ovrid,
                    }
                };
                let dispatch = self.vat_dispatch.get_mut(&vat_id).unwrap();
                dispatch.deliver(veid, vmsg);
            }

            PendingDelivery::DeliverPromise {
                vat_id,
                target: target_kprid,
                message: kmsg,
            } => {
                println!(
                    "process.DeliverPromise: {} {}.{}",
                    vat_id, target_kprid, kmsg.name
                );
                let (target_vrid, vmsg) = {
                    let mut kd = self.kd.borrow_mut();
                    let vd = kd.vat_data.get_mut(&vat_id).unwrap();
                    let target_vrid = vd.get_inbound_resolver(target_kprid);
                    let ovrid: Option<VatResolverID> = match kmsg.resolver {
                        Some(krid) => Some(vd.map_inbound_resolver(krid)),
                        None => None,
                    };
                    let vmsg = InboundVatMessage {
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
                        resolver: ovrid,
                    };
                    (target_vrid, vmsg)
                };
                let dispatch = self.vat_dispatch.get_mut(&vat_id).unwrap();
                dispatch.deliver_promise(target_vrid, vmsg);
            }

            PendingDelivery::NotifyFulfillToData {
                vat_id,
                target,
                data: kdata,
            } => {
                println!("pd::nftd");
                let (vdata, vpid) = {
                    let mut kd = self.kd.borrow_mut();
                    let vd = kd.vat_data.get_mut(&vat_id).unwrap();
                    let vdata = VatCapData {
                        body: kdata.body,
                        slots: kdata
                            .slots
                            .into_iter()
                            .map(|slot| vd.map_inbound_arg_slot(slot))
                            .collect(),
                    };
                    let vpid = vd.map_inbound_promise(target);
                    (vdata, vpid)
                };
                let dispatch = self.vat_dispatch.get_mut(&vat_id).unwrap();
                dispatch.notify_fulfill_to_data(vpid, vdata);
            }

            PendingDelivery::NotifyFulfillToTarget {
                vat_id,
                target,
                result,
            } => {
                let (vpid, vrt) = {
                    let mut kd = self.kd.borrow_mut();
                    let vd = kd.vat_data.get_mut(&vat_id).unwrap();
                    let vpid = vd.map_inbound_promise(target);
                    let vrt = vd.map_inbound_resolve_target(result);
                    (vpid, vrt)
                };
                let dispatch = self.vat_dispatch.get_mut(&vat_id).unwrap();
                dispatch.notify_fulfill_to_target(vpid, vrt);
            }

            PendingDelivery::NotifyReject {
                vat_id,
                target,
                data: kdata,
            } => {
                let (vdata, vpid) = {
                    let mut kd = self.kd.borrow_mut();
                    let vd = kd.vat_data.get_mut(&vat_id).unwrap();
                    let vdata = VatCapData {
                        body: kdata.body,
                        slots: kdata
                            .slots
                            .into_iter()
                            .map(|slot| vd.map_inbound_arg_slot(slot))
                            .collect(),
                    };
                    let vpid = vd.map_inbound_promise(target);
                    (vdata, vpid)
                };
                let dispatch = self.vat_dispatch.get_mut(&vat_id).unwrap();
                dispatch.notify_reject(vpid, vdata);
            }
        };
    }

    pub fn step(&mut self) {
        println!("kernel.step");
        let pdo = self.kd.borrow_mut().run_queue.0.pop_front();
        if let Some(pd) = pdo {
            self.process(pd);
        }
    }

    pub fn run(&mut self) {
        println!("kernel.run");
        loop {
            if self.kd.borrow_mut().run_queue.0.is_empty() {
                return;
            }
            self.step();
        }
    }

    pub fn dump(&self) {
        println!("Kernel Dump:");
        println!(" run-queue:");
        for pd in &self.kd.borrow().run_queue.0 {
            println!("  {:?}", pd);
        }
    }
}
