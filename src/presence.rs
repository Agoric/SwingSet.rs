
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) struct PresenceID(pub usize);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) struct Presence {
    pub(crate) owner: VatID,
    pub(crate) id: ExportID,
}

pub(crate) struct PresenceTable {
    pub(crate) presences: HashMap<KernelPresenceID, KernelPresence>,
    next_presence_id: usize,
}

impl PresenceTable {
    pub fn allocate(&mut self, owner: VatID, id: ExportID) -> PresenceID {
        let id = PresenceID(self.next_presence_id);
        self.next_presence_id += 1;
        self.presences.insert(id, Presence { owner, id });
        id
    }
}
