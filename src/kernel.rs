#![allow(dead_code)]

use std::collections::{HashMap, HashSet, VecDeque};
use super::clist::{CListVatEntry, CListKernelEntry, CList};

#[derive(PartialEq, Eq, Debug, Hash)]
struct VatName(String);

#[derive(PartialEq, Eq, Debug, Hash, Copy, Clone)]
struct VatID(usize);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
struct PresenceID(usize);

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
struct PromiseID(usize);

#[derive(Debug, Eq, PartialEq, Clone)]
enum PromiseState {
    Unresolved { subscribers: HashSet<VatID> },
    FulfilledToTarget(PresenceID),
    FulfilledToData(CapData),
    Rejected(CapData),
}

struct Promise {
    decider: VatID,
    allocator: VatID,
    state: PromiseState,
}

struct PromiseTable {
    promises: HashMap<PromiseID, Promise>,
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

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
struct VatPresenceID(usize);

impl CListVatEntry for VatPresenceID {
    fn new(index: usize) -> Self {
        VatPresenceID(index)
    }
}

impl CListKernelEntry for PromiseID {}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
struct VatPromiseID(usize);

impl CListVatEntry for VatPromiseID {
    fn new(index: usize) -> Self {
        VatPromiseID(index)
    }
}

#[derive(Debug)]
struct VatData {
    presence_clist: CList<PresenceID, VatPresenceID>,
    promise_clist: CList<PromiseID, VatPromiseID>,
}

trait Dispatch {}

struct Kernel {
    vat_names: HashMap<VatName, VatID>,
    vat_data: HashMap<VatID, VatData>,
    vat_dispatch: HashMap<VatID, Box<dyn Dispatch>>,
    run_queue: RunQueue,
    presences: PresenceTable,
    promises: PromiseTable,
}
