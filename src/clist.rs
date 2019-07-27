#![allow(dead_code)]

use core::hash::Hash;
use std::collections::HashMap;

// The c-lists hold mappings between kernel identifiers and vat identifiers.
// Depending upon the identifier type and the API in which it appears, the
// mapping might be "allocate if necessary", "must already be present", or
// "must not be present".

// The vat-side identifier is a signed integer: positive if the Vat allocated
// the object, negative if the kernel allocated it. The kernel-side
// identifier is always positive.

pub trait CListVatEntry: Eq + Hash + Copy {
    fn new(index: isize) -> Self;
}

pub trait CListKernelEntry: Eq + Hash + Copy {}

#[derive(Debug)]
pub struct CList<KT: CListKernelEntry, VT: CListVatEntry> {
    inbound: HashMap<KT, VT>,
    outbound: HashMap<VT, KT>,
    next_vat_index: isize,
}

impl<KT: CListKernelEntry, VT: CListVatEntry> CList<KT, VT> {
    pub fn new() -> Self {
        CList {
            inbound: HashMap::new(),
            outbound: HashMap::new(),
            next_vat_index: -1,
        }
    }

    pub fn get_outbound(&mut self, vat_object: VT) -> Option<KT> {
        match self.outbound.get(&vat_object) {
            Some(&ko) => Some(ko),
            None => None,
        }
    }

    /// Vat objects like Exports will allocate a kernel object the first time
    /// they are sent outbound, and will re-use that same kernel object next
    /// time. If allocation is necessary, it requires access to a central
    /// table (outside this one Vat's c-list), so we must be given an
    /// allocation closure just in case.
    pub fn map_outbound<A>(&mut self, vat_object: VT, allocate: A) -> KT
    where
        A: FnOnce() -> KT,
    {
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
    pub fn get_inbound(&mut self, kernel_object: KT) -> Option<VT> {
        match self.inbound.get(&kernel_object) {
            Some(&vo) => Some(vo),
            None => None,
        }
    }

    /// kernel objects being sent inbound (from the kernel, into the vat)
    /// might already exist, or they might need to allocate new vat-side
    /// identifiers. The vat-side identifier is just a (negative) number, so
    /// we can allocate it here without an external closure.
    pub fn map_inbound(&mut self, kernel_object: KT) -> VT {
        match self.inbound.get(&kernel_object) {
            Some(&vo) => vo,
            None => {
                let vat_object = VT::new(self.next_vat_index);
                self.next_vat_index -= 1;
                self.inbound.insert(kernel_object, vat_object);
                self.outbound.insert(vat_object, kernel_object);
                vat_object
            }
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
    struct VType(isize);
    impl CListVatEntry for VType {
        fn new(index: isize) -> Self {
            VType(index)
        }
    }

    #[test]
    fn test_add() {
        let mut c = CList::<KType, VType>::new();
        let k1 = KType(101);
        let v1 = VType(201);
        c.add(k1, v1);
        assert_eq!(c.map_inbound(k1), v1);
        assert_eq!(c.map_inbound(k1), v1);
        assert_eq!(c.map_outbound(v1, || panic!()), k1);
        assert_eq!(c.map_outbound(v1, || panic!()), k1);
        let k2 = KType(102);
        let v2 = c.map_inbound(k2);
        assert_eq!(Some(v2), c.get_inbound(k2));
        assert_eq!(Some(v2), c.get_inbound(k2));
        assert_eq!(Some(k2), c.get_outbound(v2));
        assert_eq!(Some(k2), c.get_outbound(v2));
    }

    #[test]
    fn test_missing_outbound() {
        let mut c = CList::<KType, VType>::new();
        let vbad = VType(666);
        assert_eq!(c.get_outbound(vbad), None);
        assert_eq!(c.map_outbound(vbad, || KType(44)), KType(44));
        assert_eq!(c.get_outbound(vbad), Some(KType(44)));
        assert_eq!(c.map_outbound(vbad, || KType(45)), KType(44));
    }

    #[test]
    fn test_missing_inbound() {
        let mut c = CList::<KType, VType>::new();
        let kbad = KType(666);
        assert_eq!(c.get_inbound(kbad), None);
        assert_eq!(c.map_inbound(kbad), VType(-1));
        assert_eq!(c.get_inbound(kbad), Some(VType(-1)));
        assert_eq!(c.map_inbound(kbad), VType(-1));
        assert_eq!(c.map_inbound(KType(333)), VType(-2));
        assert_eq!(c.map_inbound(KType(333)), VType(-2));
        assert_eq!(c.map_inbound(KType(334)), VType(-3));
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
