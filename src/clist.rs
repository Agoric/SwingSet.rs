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
    struct KType(u32);
    impl CListKernelEntry for KType {}
    #[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
    struct VType(u32);
    impl CListVatEntry for VType {
        fn new(index: u32) -> Self {
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
