use super::kernel_types::{KernelCapData, KernelExport, VatID};
use std::collections::HashSet;

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) struct PromiseID(pub usize);

//#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) enum Promise {
    Unresolved {
        subscribers: HashSet<VatID>,
        decider: VatID,
    },
    FulfilledToTarget(KernelExport),
    FulfilledToData(KernelCapData),
    Rejected(KernelCapData),
}

pub(crate) struct PromiseTable {
    pub(crate) promises: HashMap<PromiseID, Promise>,
    next_promise_id: usize,
}

impl PromiseTable {
    pub fn allocate_unresolved(&mut self, decider: VatID, id: ???PromiseID) -> PromiseID {
        let id = PromiseID(self.next_presence_id);
        self.next_presence_id += 1;
        self.presences.insert(id, Promise { owner, id });
        id
    }
}
