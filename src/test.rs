use super::{
    Config, Controller, Dispatch, Syscall, VatExportID, VatImportID, VatMessage, VatName,
    VatPromiseID, VatSendTarget,
};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct Vat1Dispatch {
    log: Rc<RefCell<Vec<u32>>>,
}
impl Dispatch for Vat1Dispatch {
    fn deliver(&mut self, syscall: &mut dyn Syscall, target: VatExportID) -> () {
        println!("Vat1.deliver {}", target);
        match target {
            VatExportID(0) => {
                println!(" deliver[0]");
                self.log.borrow_mut().push(1);
                let t = VatSendTarget::Import(VatImportID(1));
                let vmsg = VatMessage {
                    name: "foo".to_string(),
                    body: vec![],
                    slots: vec![],
                };
                let p = syscall.send(t, vmsg);
                println!(" got promise {}", p);
            }
            VatExportID(2) => {
                println!(" deliver[2]");
                self.log.borrow_mut().push(2);
            }
            _ => panic!("unknown target {}", target),
        };
    }

    fn notify_resolve_to_target(&mut self, id: VatPromiseID, target: u8) {
        println!("Vat1.notifyResolveToTarget {} {}", id, target);
    }
}

#[test]
fn test_build() {
    let mut cfg = Config::new();
    let log: Vec<u32> = vec![];
    let r = Rc::new(RefCell::new(log));
    let vat1 = Vat1Dispatch { log: r.clone() };
    let vn = VatName("bootstrap".to_string());
    cfg.add_vat(&vn, vat1);
    let mut c = Controller::new(cfg);
    c.add_import(&vn, 1, &vn, 2);
    //println!("controller: {:?}", c);
    println!("controller created");
    c.start();
    c.dump();

    println!("calling c.step");
    c.step();
    c.dump();
    assert_eq!(*r.borrow(), vec![1]);

    c.step();
    assert_eq!(*r.borrow(), vec![1, 2]);
}
