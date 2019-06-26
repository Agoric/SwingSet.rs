use swingset::{
    Config, Controller, Dispatch, Syscall, VatExportID, VatImportID, VatName, VatPromiseID,
    VatSendTarget, VatSyscall,
};

#[derive(Debug)]
struct Vat1Dispatch {}
impl Dispatch for Vat1Dispatch {
    fn deliver(&self, syscall: &mut VatSyscall, target: VatExportID) -> () {
        println!("Vat1.deliver {}", target);
        match target {
            VatExportID(0) => {
                println!(" deliver[0]");
                let t = VatSendTarget::Import(VatImportID(1));
                let p = syscall.send(t, "foo");
                println!(" got promise {}", p);
            }
            _ => panic!("unknown target {}", target),
        };
    }
    fn notify_resolve_to_target(&self, id: VatPromiseID, target: u8) {
        println!("Vat1.notifyResolveToTarget {} {}", id, target);
    }
}

fn main() {
    let mut cfg = Config::new();
    let vat1 = Vat1Dispatch {};
    cfg.add_vat(&VatName("bootstrap".to_string()), vat1);
    let mut c = Controller::new(cfg);
    //println!("controller: {:?}", c);
    println!("controller created");
    c.start();
    c.step();
}
