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
use std::fmt;

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
#[derive(Clone)]
pub struct CapData {
    pub body: Vec<u8>,
    pub slots: Vec<CapSlot>,
}

impl fmt::Debug for CapData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::str;
        let body = str::from_utf8(&self.body).unwrap_or("<<non-utf8 body>>");
        write!(
            f,
            "(vat)CapData [ body: <{}>, slots: {:?} ]",
            body, self.slots
        )
    }
}

#[derive(Debug)]
pub struct Message {
    pub method: String,
    pub args: CapData,
    pub result: Option<PromiseID>,
}
impl Message {
    pub fn new(
        method: &str,
        body: &[u8],
        slots: &[CapSlot],
        result: Option<PromiseID>,
    ) -> Self {
        Message {
            method: String::from(method),
            args: CapData {
                body: Vec::from(body),
                slots: Vec::from(slots),
            },
            result,
        }
    }
}

#[derive(Debug)]
pub enum Resolution {
    Reference(CapSlot),
    Data(CapData),
    Rejection(CapData),
}

#[derive(Debug)]
pub enum InboundTarget {
    Promise(PromiseID),
    Object(ObjectID),
}

pub trait Dispatch {
    fn deliver(&mut self, syscall: &mut dyn Syscall, target: InboundTarget, msg: Message);
    //fn subscribe(&mut self, syscall: &mut dyn Syscall, id: PromiseID);
    fn notify_resolved(
        &mut self,
        syscall: &mut dyn Syscall,
        id: PromiseID,
        to: Resolution,
    );
}

pub trait Syscall {
    fn send(&mut self, target: CapSlot, msg: Message);
    //fn invoke(&mut self, target: OutboundDeviceNode, msg: DeviceMessage) -> CapData;
    fn subscribe(&mut self, id: PromiseID);
    fn resolve(&mut self, id: PromiseID, to: Resolution);
}
