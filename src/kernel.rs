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
pub struct PresenceID(usize);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
struct Presence {
    owner: VatID,
}

struct PresenceTable {
    presences: HashMap<PresenceID, Presence>,
    next_presence_id: usize,
}

impl PresenceTable {
    fn new() -> PresenceTable {
        PresenceTable {
            presences: HashMap::default(),
            next_presence_id: 0,
        }
    }

    fn allocate(&mut self, owner: VatID) -> PresenceID {
        let id = PresenceID(self.next_presence_id);
        self.next_presence_id += 1;
        let p = Presence { owner };
        self.presences.insert(id, p);
        id
    }

    fn owner_of(&mut self, id: PresenceID) -> VatID {
        self.presences.get(&id).unwrap().owner
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct PromiseID(usize);

#[derive(Debug, Eq, PartialEq, Clone)]
enum PromiseState {
    Unresolved { subscribers: HashSet<VatID> },
    FulfilledToTarget(PresenceID),
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
enum CapSlot {
    Presence(PresenceID),
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

impl CListKernelEntry for PresenceID {}
impl CListKernelEntry for PromiseID {}

#[derive(Debug)]
pub struct VatData {
    pub id: VatID,
    pub object_clist: CList<PresenceID, VatObjectID>,
    pub promise_clist: CList<PromiseID, VatPromiseID>,
}

struct Kernel {
    vat_names: HashMap<VatName, VatID>,
    vat_dispatch: HashMap<VatID, Box<dyn Dispatch>>,
    vat_data: HashMap<VatID, VatData>,
    presences: PresenceTable,
    promises: PromiseTable,
    run_queue: RunQueue,
}
