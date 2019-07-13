use core::hash::Hash;
use std::collections::HashMap;

// The c-lists hold mappings between kernel identifiers and vat identifiers.
// Depending upon the identifier type and the API in which it appears, the
// mapping might be "allocate if necessary", "must already be present", or
// "must not be present".

pub(crate) trait CListVatEntry: Eq + Hash + Copy {
    fn new(index: usize) -> Self;
}

pub(crate) trait CListKernelEntry: Eq + Hash + Copy {
    fn new(index: usize) -> Self;
}

#[derive(Debug, Default)]
pub(crate) struct CList<KT: CListKernelEntry, VT: CListVatEntry> {
    pub(crate) inbound: HashMap<KT, VT>,
    pub(crate) outbound: HashMap<VT, KT>,
    next_vat_index: usize,
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
            next_vat_index: 0,
        }
    }

    /// use this when the vat object being sent outbound (into the kernel)
    /// must already exist in the table: no allocation
    pub fn get_outbound(&mut self, vat_object: VT) -> KT {
        if let Some(kernel_object) = self.outbound.get(&vat_object) {
            *kernel_object
        } else {
            panic!("vat object not already in table");
        }
    }

    /// Vat objects like Exports will allocate a kernel object the first time
    /// they are sent outbound, and will re-use that same kernel object next
    /// time. If allocation is necessary, it requires access to a central
    /// table (outside this one Vat's c-list), so we must be given an
    /// allocation closure just in case.
    pub fn map_outbound(&self, vat_object: VT, allocate: FnOnce() -> KT) -> KT {
        if let Some(kernel_object) = self.outbound.get(&vat_object) {
            *kernel_object
        } else {
            let kernel_object = allocate();
            self.inbound.insert(kernel_object, vat_object);
            self.outbound.insert(vat_object, kernel_object);
            kernel_object
        }
    }

    /// use this when the kernel objects being sent inbound (from the kernel,
    /// into the vat) must already exist in the table: no allocation
    pub fn get_inbound(&mut self, kernel_object: KT) -> VT {
        if let Some(vat_object) = self.inbound.get(&kernel_object) {
            *vat_object
        } else {
            panic!("kernel object not already in table");
        }
    }

    /*
    /// use this when the kernel objects being sent inbound (from the kernel,
    /// into the vat) might exist in the table, but if not we'll look in some
    /// other table
    pub fn maybe_get_inbound(&mut self, kernel_object: KT) -> Option<VT> {
        if let Some(vat_object) = self.inbound.get(&kernel_object) {
            *vat_object
        } else {
            None
        }
    }*/

    /// kernel objects being sent inbound (from the kernel, into the vat)
    /// might already exist, or they might need to allocate new vat-side
    /// identifiers. The vat-side identifier is just a number, so we can
    /// allocate it here without an external closure.
    pub fn map_inbound(&mut self, kernel_object: KT) -> VT {
        if let Some(vat_object) = self.inbound.get(&kernel_object) {
            *vat_object
        } else {
            let vat_object = VT::new(self.next_vat_index);
            self.next_vat_index += 1;
            self.inbound.insert(kernel_object, vat_object);
            self.outbound.insert(vat_object, kernel_object);
            vat_object
        }
    }

    pub fn add(&mut self, kernel_object: KT, vat_object: VT) {
        if self.inbound.get(&kernel_object).is_some() {
            panic!("already present");
        }
        if self.outbound.get(&vat_object).is_some() {
            panic!("already present");
        }
        self.inbound.insert(kernel_object, vat_object);
        self.outbound.insert(vat_object, kernel_object);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
    struct KType(usize);
    impl CListKernelEntry for KType {}
    #[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
    struct VType(usize);
    impl CListVatEntry for VType {
        fn new(index: usize) -> Self {
            VType(index)
        }
    }

    #[test]
    fn test_clist() {
        let mut c = CList::<KType, VType>::new();
        let k1 = KType(101);
        let v1 = VType(201);
        c.add(k1, v1);
        assert_eq!(c.map_inbound(k1), v1);
        assert_eq!(c.map_inbound(k1), v1);
        assert_eq!(c.map_outbound(v1), k1);
        assert_eq!(c.map_outbound(v1), k1);
        let k2 = KType(102);
        let v2 = c.map_inbound(k2);
        assert_eq!(v2, c.map_inbound(k2));
        assert_eq!(v2, c.map_inbound(k2));
        assert_eq!(k2, c.map_outbound(v2));
        assert_eq!(k2, c.map_outbound(v2));
    }

    #[test]
    #[should_panic]
    fn test_missing_outbound() {
        let c = CList::<KType, VType>::new();
        let vbad = VType(666);
        c.map_outbound(vbad);
    }

    #[test]
    #[should_panic]
    fn test_add_duplicate_ktype() {
        let mut c = CList::<KType, VType>::new();
        let k1 = KType(101);
        let v1 = VType(201);
        c.add(k1, v1);

        c.add(k1, VType(202));
    }

    #[test]
    #[should_panic]
    fn test_add_duplicate_vtype() {
        let mut c = CList::<KType, VType>::new();
        let k1 = KType(101);
        let v1 = VType(201);
        c.add(k1, v1);

        c.add(KType(102), v1);
    }

}
