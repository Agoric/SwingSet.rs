use super::kernel_types::VatID;
use std::collections::HashMap;

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
    pub fn allocate(&mut self, owner: VatID) -> PresenceID {
        let id = PresenceID(self.next_presence_id);
        self.next_presence_id += 1;
        let p = Presence { owner };
        self.presences.insert(id, p);
        id
    }

    pub fn owner_of(&mut self, id: PresenceID) {
        *self.presences.get(&id)
    }
}
