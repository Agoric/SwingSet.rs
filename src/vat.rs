/// This file defines everything that comes into contact with Vat code: the
/// dispatch/syscall API (the two function groups at the boundary between Vat
/// and Kernel), and the types used in them.
///
/// There are four kinds of references that make an appearance in the
/// dispatch/syscall API: the cross-product of two properties (all from the
/// perspective of the Vat holding/sending/receiving the reference):
///
/// allocation:
///   1: I allocate the ID
///   2: Somebody else allocates it
/// resolution:
///   1: It is already resolved
///   2: It is not yet resolved
///
/// We give these four names:
///
/// Local Promise: I allocate the ID, someone (maybe me) can resolve it.
/// Export: I allocate the ID, it is already resolved
/// RemotePromise: Somebody else allocated the ID, someone (maybe me) will resolve it
/// Import: Somebody else allocated the ID, and it is already resolved

// TODO: we need a name for the pass-by-presence type. "target"? "export"?

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct PromiseID(pub isize);
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct ObjectID(pub isize);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum CapSlot {
    Promise(PromiseID),
    Object(ObjectID),
}

/// CapData is capability-bearing data, used for the message arguments and
/// resolving/rejecting promises to non-callable targets
#[derive(Debug, Clone)]
pub struct CapData {
    pub body: Vec<u8>,
    pub slots: Vec<CapSlot>,
}

pub struct Message {
    pub method: String,
    pub args: CapData,
    pub result: Option<PromiseID>,
}

pub enum Resolution {
    Reference(CapSlot),
    Data(CapData),
    Rejection(CapData),
}

pub enum InboundTarget {
    Promise(PromiseID),
    Object(ObjectID),
}

pub trait Dispatch {
    fn deliver(&mut self, target: InboundTarget, msg: Message);
    fn subscribe(&mut self, id: PromiseID);
    fn notify_resolved(&mut self, id: PromiseID, to: Resolution);
}
