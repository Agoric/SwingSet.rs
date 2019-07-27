#![allow(dead_code)]

use super::clist::{CList, CListKernelEntry};
use super::vat::Dispatch;
use super::vat::{VatObjectID, VatPromiseID};
use std::collections::{HashMap, HashSet, VecDeque};

#[derive(PartialEq, Eq, Debug, Hash)]
struct VatName(String);

#[derive(PartialEq, Eq, Debug, Hash, Copy, Clone)]
pub struct VatID(usize);

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
    fn new() -> ObjectTable {
        ObjectTable {
            objects: HashMap::default(),
            next_object_id: 0,
        }
    }

    fn allocate(&mut self, owner: VatID) -> ObjectID {
        let id = ObjectID(self.next_object_id);
        self.next_object_id += 1;
        let o = Object { owner };
        self.objects.insert(id, o);
        id
    }

    fn owner_of(&mut self, id: ObjectID) -> VatID {
        self.objects.get(&id).unwrap().owner
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct PromiseID(usize);

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
    fn new() -> PromiseTable {
        PromiseTable {
            promises: HashMap::default(),
            next_promise_id: 0,
        }
    }

    fn allocate_unresolved(&mut self, decider: VatID, allocator: VatID) -> PromiseID {
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

    fn decider_of(&mut self, id: PromiseID) -> VatID {
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
struct CapData {
    body: Vec<u8>,
    slots: Vec<CapSlot>,
}

#[derive(Debug)]
struct Message {
    method: String,
    args: CapData,
    result: Option<PromiseID>,
}

#[derive(Debug)]
enum Resolution {
    Reference(CapSlot),
    Data(CapData),
    Rejection(CapData),
}

#[derive(Debug)]
enum PendingDelivery {
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

#[derive(Debug)]
struct RunQueue(VecDeque<PendingDelivery>);

impl CListKernelEntry for ObjectID {}
impl CListKernelEntry for PromiseID {}

#[derive(Debug)]
pub struct VatData {
    pub id: VatID,
    pub object_clist: CList<ObjectID, VatObjectID>,
    pub promise_clist: CList<PromiseID, VatPromiseID>,
}

struct Kernel {
    vat_names: HashMap<VatName, VatID>,
    vat_dispatch: HashMap<VatID, Box<dyn Dispatch>>,
    vat_data: HashMap<VatID, VatData>,
    objects: ObjectTable,
    promises: PromiseTable,
    run_queue: RunQueue,
}
