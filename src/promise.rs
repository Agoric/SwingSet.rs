use super::kernel_types::{CapData, VatID};
use super::presence::PresenceID;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) struct PromiseID(pub usize);

//#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) enum PromiseState {
    Unresolved { subscribers: HashSet<VatID> },
    FulfilledToTarget(PresenceID),
    FulfilledToData(CapData),
    Rejected(CapData),
}

pub(crate) struct Promise {
    pub(crate) decider: VatID,
    pub(crate) allocator: VatID,
    pub(crate) state: PromiseState,
}

pub(crate) struct PromiseTable {
    pub(crate) promises: HashMap<PromiseID, Promise>,
    next_promise_id: usize,
}

impl PromiseTable {
    pub fn allocate_unresolved(&mut self, decider: VatID, allocator: VatID) -> PromiseID {
        let id = PromiseID(self.next_promise_id);
        self.next_promise_id += 1;
        let state = PromiseState::Unresolved {
            subscribers: vec![],
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

    pub fn decider_of(&mut self, id: PromiseID) {
        self.promises.get(&id).decider
    }
}
