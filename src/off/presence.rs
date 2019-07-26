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
