/// This defines the dispatch/syscall API (the two function groups at the
/// boundary between Vat and Kernel), and the types used in them.
///
/// There are six kinds of references that make an appearance in the
/// dispatch/syscall API: the cross-product of two properties (all from the
/// perspective of the Vat holding/sending/receiving the reference):
///
/// allocation:
///   1: I allocate the ID
///   2: Somebody else allocates it
/// resolution:
///   1: I can resolve it
///   2: Somebody else resolves it
///   3: It is already resolved
///
/// We give these six names:
///
/// Local Promise: I allocate the ID, I can resolve it.
/// Send Result: I allocate the ID, somebody else resolves it
/// Export: I allocate the ID, it is already resolved
/// Dispatch Result: Somebody else allocated the ID, but I can resolve it
/// RemotePromise: Somebody else allocated the ID, and they will resolve it
/// Import: Somebody else allocated the ID, and it is already resolved

// TODO: we need a name for the pass-by-presence type. "target"? "export"?

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct LocalPromise(pub usize);
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct SendResult(pub usize);
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Export(pub usize);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct DispatchResult(pub usize);
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct RemotePromise(pub usize);
#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub struct Import(pub usize);

#[derive(Debug, Eq, PartialEq, Hash, Copy, Clone)]
pub enum CapSlot {
    LocalPromise(LocalPromise),
    SendResult(SendResult),
    Export(Export),
    DispatchResult(DispatchResult),
    RemotePromise(RemotePromise),
    Import(Import),
}

/// CapData is capability-bearing data, used for the message arguments and
/// resolving/rejecting promises to non-callable targets
#[derive(Debug, Clone)]
pub struct CapData {
    pub body: Vec<u8>,
    pub slots: Vec<CapSlot>,
}

pub struct OutboundMessage {
    pub name: String,
    pub args: CapData,
    pub result: Option<SendResult>,
}

enum OutboundTarget {
    SendResult(SendResult),
    RemotePromise(RemotePromise),
    Import(Import),
}

enum RemotelyResolvable {
    SendResult(SendResult),
    RemotePromise(RemotePromise),
}

enum LocallyResolvable {
    LocalPromise(LocalPromise),
    DispatchResult(DispatchResult),
}

enum Resolution {
    Reference(CapSlot),
    Data(CapData),
    Rejection(CapData),
}

pub trait Syscall {
    fn send(&mut self, target: OutboundTarget, msg: OutboundMessage);
    //fn invoke(&mut self, target: OutboundDeviceNode, msg: OutboundDeviceMessage) -> CapData;
    fn subscribe(&mut self, id: RemotelyResolvable);
    fn resolve(&mut self, id: LocallyResolvable, to: Resolution);
}

enum InboundTarget {
    LocalPromise(LocalPromise),
    Export(Export),
    DispatchResult(DispatchResult),
}

pub struct InboundMessage {
    pub name: String,
    pub args: CapData,
    pub result: Option<DispatchResult>,
}

pub trait Dispatch {
    fn deliver(&mut self, target: InboundTarget, msg: InboundMessage);
    fn subscribe(&mut self, id: LocallyResolvable);
    fn notify_resolved(&mut self, id: RemotelyResolvable, to: Resolution);
}
