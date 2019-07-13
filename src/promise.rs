use super::kernel_types::{CapData, VatID};
use super::presence::PresenceID;
use std::collections::{HashMap, HashSet};

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
