use super::config::Config;
use super::kernel_types::{
    CapData, CapSlot, Message, PendingDelivery, Resolution, RunQueue, VatID, VatName,
};
use super::presence::PresenceTable;
use super::promise::{PromiseID, PromiseTable};
use super::syscall::{
    CapData as VatCapData,
    CapSlot as VatCapSlot,
    Dispatch,
    ExportID,
    ImportID,
    InboundTarget,
    //LocalPromiseID, RemotePromiseID,
    Message as VatMessage,
    Promise as VatPromise,
    Resolution as VatResolution,
};
use super::vat::VatSyscall;
use super::vat_manager::VatData;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub(crate) struct KernelData {
    pub(crate) vat_names: HashMap<VatName, VatID>,
    pub(crate) vat_data: HashMap<VatID, VatData>,
    pub(crate) run_queue: RunQueue,
    pub(crate) presences: PresenceTable,
    pub(crate) promises: PromiseTable,
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
            presences: PresenceTable::default(),
            promises: PromiseTable::default(),
        }));
        let mut id = 0;
        for (key, setup) in cfg.vats {
            let vat_id = VatID(id);
            id += 1;
            kd.borrow_mut()
                .vat_names
                .insert(VatName(key.0.clone()), vat_id);
            let vd = VatData::new(vat_id);
            kd.borrow_mut().vat_data.insert(vat_id, vd);
            let syscall = VatSyscall::new(vat_id, kd.clone());
            let dispatch = setup(Box::new(syscall));
            vat_dispatch.insert(vat_id, dispatch);
        }
        Kernel { vat_dispatch, kd }
    }

    /*
    fn vat_id_of_target(&mut self, target: CapSlot) -> VatID {
        let mut kd = self.kd.borrow_mut();
        match target {
            CapSlot::Presence(id) => kd.presences.owner_of(id),
            CapSlot::Promise(id) => kd.promises.decider_of(id),
        }
    }*/

    fn map_inbound_promise(&mut self, to: VatID, id: PromiseID) -> VatPromise {
        let mut kd = self.kd.borrow_mut();
        let allocator = kd.promises.promises.get(&id).unwrap().allocator;
        let vd = kd.vat_data.get_mut(&allocator).unwrap();
        if to == allocator {
            VatPromise::LocalPromise(vd.local_promise_clist.map_inbound(id))
        } else {
            VatPromise::RemotePromise(vd.remote_promise_clist.map_inbound(id))
        }
    }

    fn map_inbound_slot(&mut self, to: VatID, slot: CapSlot) -> VatCapSlot {
        match slot {
            CapSlot::Presence(id) => {
                let mut kd = self.kd.borrow_mut();
                let owner = kd.presences.presences.get(&id).unwrap().owner;
                let vd = kd.vat_data.get_mut(&owner).unwrap();
                if to == owner {
                    VatCapSlot::Export(vd.export_clist.get_inbound(id))
                } else {
                    VatCapSlot::Import(vd.import_clist.map_inbound(id))
                }
            }
            CapSlot::Promise(id) => match self.map_inbound_promise(to, id) {
                VatPromise::LocalPromise(pid) => VatCapSlot::LocalPromise(pid),
                VatPromise::RemotePromise(pid) => VatCapSlot::RemotePromise(pid),
            },
        }
    }

    fn map_inbound_capdata(&mut self, to: VatID, data: CapData) -> VatCapData {
        VatCapData {
            body: data.body,
            slots: data
                .slots
                .into_iter()
                .map(|slot| self.map_inbound_slot(to, slot))
                .collect(),
        }
    }

    fn map_inbound_resolution(
        &mut self,
        to: VatID,
        resolution: Resolution,
    ) -> VatResolution {
        use Resolution::*;
        match resolution {
            Reference(slot) => VatResolution::Reference(self.map_inbound_slot(to, slot)),
            Data(data) => VatResolution::Data(self.map_inbound_capdata(to, data)),
            Rejection(data) => {
                VatResolution::Rejection(self.map_inbound_capdata(to, data))
            }
        }
    }

    fn map_inbound_result(
        &mut self,
        to: VatID,
        kresult: Option<PromiseID>,
    ) -> Option<VatPromise> {
        match kresult {
            None => None,
            Some(id) => {
                let decider = {
                    let kd = self.kd.borrow_mut();
                    kd.promises.promises.get(&id).unwrap().decider
                };
                if decider != to {
                    panic!("result {} is not being decided by recipient {}", id, to);
                }
                Some(self.map_inbound_promise(to, id))
            }
        }
    }

    fn map_inbound_message(&mut self, to: VatID, kmsg: Message) -> VatMessage {
        VatMessage {
            method: kmsg.method,
            args: self.map_inbound_capdata(to, kmsg.args),
            result: self.map_inbound_result(to, kmsg.result),
        }
    }

    /*
    fn map_outbound_promise(&mut self, to: VatID, id: PromiseID) -> VatPromise {
        let mut kd = self.kd.borrow_mut();
        let allocator = kd.promises.promises.get(&id).unwrap().allocator;
        let vd = kd.vat_data.get_mut(&allocator).unwrap();
        if to == allocator {
            VatPromise::LocalPromise(vd.local_promise_clist.map_inbound(id))
        } else {
            VatPromise::RemotePromise(vd.remote_promise_clist.map_inbound(id))
        }
    }
    */

    //pub fn map_outbound_target(&mut self, vtarget: VatCapSlot) -> CapSlot {}

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
        for_id: usize,
        to_vat: &VatName,
        to_id: usize,
    ) {
        let mut kd = self.kd.borrow_mut();
        let for_vat_id = *kd.vat_names.get(&for_vat).unwrap();
        let to_vat_id = *kd.vat_names.get(&to_vat).unwrap();
        let ve = ExportID(to_id);
        /*
        vd.export_clist
        .map_outbound(ExportID(to_id), &|| kd.presences.allocate(to_vat_id))
         */
        let opid = {
            let vd = kd.vat_data.get_mut(&to_vat_id).unwrap();
            vd.export_clist.maybe_get_outbound(ve)
        };
        let pid = match opid {
            Some(pid) => pid,
            None => {
                let pid = kd.presences.allocate(to_vat_id);
                let vd = kd.vat_data.get_mut(&to_vat_id).unwrap();
                vd.export_clist.add(pid, ve);
                pid
            }
        };
        let vd = kd.vat_data.get_mut(&for_vat_id).unwrap();
        vd.import_clist.add(pid, ImportID(for_id));
    }

    /*
    pub(crate) fn push_deliver(
        &mut self,
        name: &VatName,
        target: ExportID,
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
    */

    fn process(&mut self, pd: PendingDelivery) {
        match pd {
            PendingDelivery::Deliver { target, message } => {
                println!("process.Deliver: {}.{}", target, message.method);
                let (vat_id, vtarget) = match target {
                    CapSlot::Presence(id) => {
                        let owner = self.kd.borrow().presences.presences.get(&id).unwrap().owner;
                        let mut kd = self.kd.borrow_mut();
                        let vd = kd.vat_data.get_mut(&owner).unwrap();
                        let vid = vd.export_clist.get_inbound(id);
                        (owner, InboundTarget::Export(vid))
                    }
                    CapSlot::Promise(id) => {
                        let decider = self.kd.borrow().promises.promises.get(&id).unwrap().decider;
                        let p = self.map_inbound_promise(decider, id);
                        (
                            decider,
                            match p {
                                // the message is sent to a locally-created Promise
                                VatPromise::LocalPromise(pid) => {
                                    InboundTarget::LocalPromise(pid)
                                }
                                // target is the result for an earlier deliver()
                                VatPromise::RemotePromise(pid) => {
                                    InboundTarget::RemotePromise(pid)
                                }
                            },
                        )
                    }
                };
                let vmessage = self.map_inbound_message(vat_id, message);
                let dispatch = self.vat_dispatch.get_mut(&vat_id).unwrap();
                dispatch.deliver(vtarget, vmessage);
            }

            PendingDelivery::Notify {
                vat_id,
                promise,
                resolution,
            } => {
                println!("Process.Notify");
                let vpromise = self.map_inbound_promise(vat_id, promise);
                let vresolution = self.map_inbound_resolution(vat_id, resolution);
                let dispatch = self.vat_dispatch.get_mut(&vat_id).unwrap();
                dispatch.notify_resolved(vpromise, vresolution);
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
