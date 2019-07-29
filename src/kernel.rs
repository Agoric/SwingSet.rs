#![allow(dead_code)]

use super::map_inbound::{
    map_inbound_message, map_inbound_promise, map_inbound_resolution, map_inbound_target,
};
use super::map_outbound::SyscallHandler;
use super::vat::{Dispatch, ObjectID as VatObjectID, PromiseID as VatPromiseID, Syscall};
use super::vat_data::VatData;
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(PartialEq, Eq, Debug, Hash)]
struct VatName(String);

#[derive(PartialEq, Eq, Debug, Hash, Copy, Clone)]
pub struct VatID(pub usize);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct ObjectID(usize);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Object {
    pub owner: VatID,
}

pub struct ObjectTable {
    pub objects: HashMap<ObjectID, Object>,
    next_object_id: usize,
}

impl ObjectTable {
    pub fn new() -> ObjectTable {
        ObjectTable {
            objects: HashMap::default(),
            next_object_id: 0,
        }
    }

    pub fn allocate(&mut self, owner: VatID) -> ObjectID {
        let id = ObjectID(self.next_object_id);
        self.next_object_id += 1;
        let o = Object { owner };
        self.objects.insert(id, o);
        id
    }

    pub fn owner_of(&self, id: ObjectID) -> VatID {
        self.objects.get(&id).unwrap().owner
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct PromiseID(pub usize);

#[derive(Debug, Eq, PartialEq, Clone)]
enum PromiseState {
    Unresolved { subscribers: HashSet<VatID> },
    FulfilledToTarget(ObjectID),
    FulfilledToData(CapData),
    Rejected(CapData),
}

pub struct Promise {
    decider: VatID,
    pub allocator: VatID,
    state: PromiseState,
}

pub struct PromiseTable {
    pub promises: HashMap<PromiseID, Promise>,
    next_promise_id: usize,
}

impl PromiseTable {
    pub fn new() -> PromiseTable {
        PromiseTable {
            promises: HashMap::default(),
            next_promise_id: 0,
        }
    }

    pub fn allocate_unresolved(&mut self, decider: VatID, allocator: VatID) -> PromiseID {
        let id = PromiseID(self.next_promise_id);
        self.next_promise_id += 1;
        let state = PromiseState::Unresolved {
            subscribers: HashSet::default(),
        };
        self.promises.insert(
            id,
            Promise {
                decider,
                allocator,
                state,
            },
        );
        id
    }

    pub fn allocator_of(&self, id: PromiseID) -> VatID {
        self.promises.get(&id).unwrap().allocator
    }

    pub fn decider_of(&self, id: PromiseID) -> VatID {
        self.promises.get(&id).unwrap().decider
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum CapSlot {
    Object(ObjectID),
    Promise(PromiseID),
}

/// CapData is capability-bearing data, used for the message arguments and
/// resolving/rejecting promises to non-callable targets
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CapData {
    pub body: Vec<u8>,
    pub slots: Vec<CapSlot>,
}

#[derive(Debug)]
pub struct Message {
    pub method: String,
    pub args: CapData,
    pub result: Option<PromiseID>,
}

#[derive(Debug)]
pub enum Resolution {
    Reference(CapSlot),
    Data(CapData),
    Rejection(CapData),
}

#[derive(Debug)]
pub enum PendingDelivery {
    Deliver {
        target: CapSlot,
        message: Message,
    },
    Notify {
        vat_id: VatID,
        promise: PromiseID,
        resolution: Resolution,
    },
}

#[derive(Debug, Default)]
struct RunQueue(VecDeque<PendingDelivery>);

struct Kernel {
    vat_names: HashMap<VatName, VatID>,
    vat_dispatch: HashMap<VatID, Box<dyn Dispatch>>,
    vat_data: HashMap<VatID, VatData>,
    objects: ObjectTable,
    promises: PromiseTable,
    run_queue: RunQueue,
}

impl Kernel {
    fn new() -> Self {
        Kernel {
            vat_names: HashMap::default(),
            vat_dispatch: HashMap::default(),
            vat_data: HashMap::default(),
            objects: ObjectTable::new(),
            promises: PromiseTable::new(),
            run_queue: RunQueue::default(),
        }
    }

    fn process(&mut self, pd: PendingDelivery) {
        let ot = &self.objects;
        let pt = &self.promises;
        match pd {
            PendingDelivery::Deliver { target, message } => {
                let vat_id = match target {
                    CapSlot::Object(id) => ot.owner_of(id),
                    CapSlot::Promise(id) => pt.decider_of(id),
                };
                let vd = self.vat_data.get_mut(&vat_id).unwrap();
                let vt = map_inbound_target(vd, ot, pt, target);
                let vmsg = map_inbound_message(vd, ot, pt, message);
                drop(vd);
                let mut s = SyscallHandler::new();
                let dispatch = self.vat_dispatch.get_mut(&vat_id).unwrap();
                dispatch.deliver(&mut s, vt, vmsg)
            }
            PendingDelivery::Notify {
                vat_id,
                promise,
                resolution,
            } => {
                let vd = self.vat_data.get_mut(&vat_id).unwrap();
                let vpid = map_inbound_promise(vd, pt, promise);
                let vres = map_inbound_resolution(vd, ot, pt, resolution);
                drop(vd);
                let mut s = SyscallHandler::new();
                let dispatch = self.vat_dispatch.get_mut(&vat_id).unwrap();
                dispatch.notify_resolved(&mut s, vpid, vres)
            }
        };
    }

    fn get_next(&mut self) -> Option<PendingDelivery> {
        self.run_queue.0.pop_front()
    }

    pub fn step(&mut self) -> bool {
        println!("kernel.step");
        if let Some(pd) = self.get_next() {
            self.process(pd);
            return true;
        }
        false
    }

    pub fn run(&mut self) {
        println!("kernel.run");
        loop {
            if !self.step() {
                return;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_create() {
        let mut _k = Kernel::new();
    }
}
