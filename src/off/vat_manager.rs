use super::clist::{CList, CListKernelEntry, CListVatEntry};
use super::kernel_types::VatID;
use super::presence::PresenceID;
use super::promise::PromiseID;
use super::syscall::{
    ExportID,
    //Message as VatMessage, CapSlot as VatCapSlot,
    //Promise as VatPromise, InboundTarget,
    ImportID,
    LocalPromiseID,
    RemotePromiseID,
};

impl CListVatEntry for ImportID {
    fn new(index: usize) -> Self {
        ImportID(index)
    }
}
impl CListVatEntry for ExportID {
    fn new(index: usize) -> Self {
        ExportID(index)
    }
}
impl CListVatEntry for LocalPromiseID {
    fn new(index: usize) -> Self {
        LocalPromiseID(index)
    }
}
impl CListVatEntry for RemotePromiseID {
    fn new(index: usize) -> Self {
        RemotePromiseID(index)
    }
}

impl CListKernelEntry for PresenceID {
    fn new(index: usize) -> PresenceID {
        PresenceID(index)
    }
}
impl CListKernelEntry for PromiseID {
    fn new(index: usize) -> PromiseID {
        PromiseID(index)
    }
}

pub(crate) struct VatData {
    #[allow(dead_code)]
    pub(crate) vat_id: VatID,
    pub(crate) import_clist: CList<PresenceID, ImportID>,
    pub(crate) export_clist: CList<PresenceID, ExportID>,
    pub(crate) local_promise_clist: CList<PromiseID, LocalPromiseID>,
    pub(crate) remote_promise_clist: CList<PromiseID, RemotePromiseID>,
}

impl VatData {
    pub fn new(vat_id: VatID) -> Self {
        VatData {
            vat_id,
            import_clist: CList::new(),
            export_clist: CList::new(),
            local_promise_clist: CList::new(),
            remote_promise_clist: CList::new(),
        }
    }

    /*
    pub fn map_outbound_target(&mut self, vtarget: VatCapSlot) -> CapSlot {
    }

    pub fn map_outbound_resolve_target(
        &mut self,
        vtarget: VatResolveTarget,
    ) -> KernelExport {
        match vtarget {
            VatResolveTarget::Export(VatExportID(id)) => {
                KernelExport(self.vat_id, KernelExportID(id))
            }
            VatResolveTarget::Import(viid) => self.import_clist.map_outbound(viid),
        }
    }

    pub fn get_outbound_promise(
        &mut self,
        vpid: VatPromiseID,
    ) -> KernelPromiseResolverID {
        self.promise_clist.get_outbound(vpid)
    }

    pub fn map_outbound_resolver(
        &mut self,
        vrid: VatResolverID,
    ) -> KernelPromiseResolverID {
        self.resolver_clist.map_outbound(vrid)
    }

    pub fn forward_promise(
        &mut self,
        old_id: KernelPromiseResolverID,
        new_id: KernelPromiseResolverID,
    ) {
        let pc = &mut self.promise_clist;
        if pc.inbound.contains_key(&old_id) {
            let vpid = *pc.inbound.get(&old_id).unwrap();
            assert!(pc.outbound.contains_key(&vpid));
            pc.inbound.remove(&old_id);
            pc.inbound.insert(new_id, vpid);
            pc.outbound.insert(vpid, new_id);
        }
    }
    */
}
