use core::hash::Hash;
use std::collections::HashMap;

pub(crate) trait CListVatEntry: Eq + Hash + Copy {
    fn new(index: u32) -> Self;
}

pub(crate) trait CListKernelEntry: Eq + Hash + Copy {}

#[derive(Debug, Default)]
pub(crate) struct CList<KT: CListKernelEntry, VT: CListVatEntry> {
    inbound: HashMap<KT, VT>,
    outbound: HashMap<VT, KT>,
    next_index: u32,
}
impl<KT: CListKernelEntry, VT: CListVatEntry> CList<KT, VT> {
    /*pub fn _map_outbound<T: Into<VatArgSlot>>(&self, target: T) -> KernelArgSlot {
        let t = self.outbound.get(&target.into()).unwrap();
        (*t).clone()
    }*/

    pub fn new() -> Self {
        CList {
            inbound: HashMap::new(),
            outbound: HashMap::new(),
            next_index: 0,
        }
    }

    /// vat objects being sent outbound (out of the vat and into the kernel)
    /// must already exist in the clist: this is how we confine vats to only
    /// use previously-granted authorities
    pub fn map_outbound(&self, vat_object: VT) -> KT {
        *self.outbound.get(&vat_object).unwrap()
    }

    /// kernel objects being sent inbound (from the kernel, into the vat)
    /// might already exist, or they might need to allocate new vat-side
    /// identifiers
    pub fn map_inbound(&mut self, kernel_object: KT) -> VT {
        if let Some(vat_object) = self.inbound.get(&kernel_object) {
            *vat_object
        } else {
            let vat_object = VT::new(self.next_index);
            self.next_index += 1;
            self.inbound.insert(kernel_object, vat_object);
            self.outbound.insert(vat_object, kernel_object);
            vat_object
        }
    }

    pub fn add(&mut self, kernel_object: KT, vat_object: VT) {
        if let Some(_) = self.inbound.get(&kernel_object) {
            panic!("already present");
        }
        if let Some(_) = self.outbound.get(&vat_object) {
            panic!("already present");
        }
        self.inbound.insert(kernel_object, vat_object);
        self.outbound.insert(vat_object, kernel_object);
    }
}
