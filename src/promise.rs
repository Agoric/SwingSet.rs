use super::kernel_types::{KernelCapData, KernelExport, VatID};
use std::collections::HashSet;

pub(crate) enum KernelPromise {
    Unresolved {
        subscribers: HashSet<VatID>,
        decider: VatID,
    },
    FulfilledToTarget(KernelExport),
    FulfilledToData(KernelCapData),
    Rejected(KernelCapData),
}
