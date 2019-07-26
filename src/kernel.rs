use std::collections::{HashMap, HashSet, VecDeque};

#[derive(PartialEq, Eq, Debug, Hash)]
pub struct VatName(pub String);

#[derive(PartialEq, Eq, Debug, Hash, Copy, Clone)]
pub(crate) struct VatID(pub usize);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) struct PresenceID(pub usize);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) struct Presence {
    pub(crate) owner: VatID,
}

pub(crate) struct PresenceTable {
    pub(crate) presences: HashMap<PresenceID, Presence>,
    next_presence_id: usize,
}

impl PresenceTable {
    pub fn default() -> PresenceTable {
        PresenceTable {
            presences: HashMap::default(),
            next_presence_id: 0,
        }
    }

    pub fn allocate(&mut self, owner: VatID) -> PresenceID {
        let id = PresenceID(self.next_presence_id);
        self.next_presence_id += 1;
        let p = Presence { owner };
        self.presences.insert(id, p);
        id
    }

    #[allow(dead_code)]
    pub fn owner_of(&mut self, id: PresenceID) -> VatID {
        self.presences.get(&id).unwrap().owner
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) struct PromiseID(pub usize);

//#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) enum PromiseState {
    #[allow(dead_code)]
    Unresolved { subscribers: HashSet<VatID> },
    #[allow(dead_code)]
    FulfilledToTarget(PresenceID),
    #[allow(dead_code)]
    FulfilledToData(CapData),
    #[allow(dead_code)]
    Rejected(CapData),
}

pub(crate) struct Promise {
    pub(crate) decider: VatID,
    pub(crate) allocator: VatID,
    #[allow(dead_code)]
    pub(crate) state: PromiseState,
}

pub(crate) struct PromiseTable {
    pub(crate) promises: HashMap<PromiseID, Promise>,
    pub(crate) next_promise_id: usize,
}

impl PromiseTable {
    pub fn default() -> PromiseTable {
        PromiseTable {
            promises: HashMap::default(),
            next_promise_id: 0,
        }
    }

    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn decider_of(&mut self, id: PromiseID) -> VatID {
        self.promises.get(&id).unwrap().decider
    }
}



#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) enum CapSlot {
    #[allow(dead_code)]
    Presence(PresenceID),
    #[allow(dead_code)]
    Promise(PromiseID),
}

/// CapData is capability-bearing data, used for the message arguments and
/// resolving/rejecting promises to non-callable targets
#[derive(Debug, Clone)]
pub(crate) struct CapData {
    pub(crate) body: Vec<u8>,
    pub(crate) slots: Vec<CapSlot>,
}

#[derive(Debug)]
pub(crate) struct Message {
    pub(crate) method: String,
    pub(crate) args: CapData,
    pub(crate) result: Option<PromiseID>,
}

#[derive(Debug)]
pub(crate) enum Resolution {
    #[allow(dead_code)]
    Reference(CapSlot),
    #[allow(dead_code)]
    Data(CapData),
    #[allow(dead_code)]
    Rejection(CapData),
}



#[derive(Debug)]
pub(crate) enum PendingDelivery {
    #[allow(dead_code)]
    Deliver { target: CapSlot, message: Message },
    #[allow(dead_code)]
    Notify {
        vat_id: VatID,
        promise: PromiseID,
        resolution: Resolution,
    },
}

#[derive(Debug, Default)]
pub(crate) struct RunQueue(pub VecDeque<PendingDelivery>);

pub(crate) struct Kernel {
    pub(crate) vat_names: HashMap<VatName, VatID>,
    pub(crate) vat_data: HashMap<VatID, VatData>,
    pub(crate) vat_dispatch: HashMap<VatID, Box<dyn Dispatch>>,
    pub(crate) run_queue: RunQueue,
    pub(crate) presences: PresenceTable,
    pub(crate) promises: PromiseTable,
}
