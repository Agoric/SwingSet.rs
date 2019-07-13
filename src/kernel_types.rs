use super::presence::PresenceID;
use super::promise::PromiseID;
use std::collections::VecDeque;

#[derive(PartialEq, Eq, Debug, Hash)]
pub struct VatName(pub String);

#[derive(PartialEq, Eq, Debug, Hash, Copy, Clone)]
pub(crate) struct VatID(pub usize);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub(crate) enum CapSlot {
    Presence(PresenceID),
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
    pub(crate) name: String,
    pub(crate) args: CapData,
    pub(crate) result: Option<PromiseID>,
}

pub enum Resolution {
    Reference(CapSlot),
    Data(CapData),
    Rejection(CapData),
}

#[derive(Debug)]
pub(crate) enum PendingDelivery {
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

#[derive(Debug, Default)]
pub(crate) struct RunQueue(pub VecDeque<PendingDelivery>);
