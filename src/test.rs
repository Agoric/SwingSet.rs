use super::{
    Config, Controller, Dispatch, InboundVatMessage, OutboundVatMessage, Syscall,
    VatCapData, VatExportID, VatImportID, VatName, VatPromiseID, VatSendTarget,
};
use std::cell::RefCell;
use std::rc::Rc;

//#[derive(Debug)]
struct Vat1Dispatch {
    syscall: Box<dyn Syscall>,
    log: Rc<RefCell<Vec<u32>>>,
}
impl Dispatch for Vat1Dispatch {
    fn deliver(&mut self, target: VatExportID, message: InboundVatMessage) -> () {
        println!("Vat1.deliver {}", target);
        match target {
            VatExportID(0) => {
                println!(" deliver[0]");
                assert_eq!(message.name, "bootstrap");
                assert_eq!(message.args.body, b"");
                assert_eq!(message.args.slots, vec![]);
                self.log.borrow_mut().push(1);
                let t = VatSendTarget::Import(VatImportID(1));
                let vmsg = OutboundVatMessage::new("foo", b"body", vec![]);
                let p = self.syscall.send(t, vmsg);
                println!(" got promise {}", p);
            }
            VatExportID(2) => {
                println!(" deliver[2]");
                assert_eq!(message.name, "foo");
                assert_eq!(message.args.body, b"body");
                assert_eq!(message.args.slots, vec![]);
                self.log.borrow_mut().push(2);
            }
            _ => panic!("unknown target {}", target),
        };
    }

    fn notify_fulfill_to_target(&mut self, id: VatPromiseID, target: VatSendTarget) {
        println!("Vat1.notifyResolveToTarget {} {}", id, target);
    }
    fn notify_fulfill_to_data(&mut self, _id: VatPromiseID, _data: VatCapData) {}
    fn notify_reject(&mut self, _id: VatPromiseID, _data: VatCapData) {}
}

use super::config::Setup;
#[test]
fn test_build() {
    let mut cfg = Config::new();
    let log: Vec<u32> = vec![];
    let r = Rc::new(RefCell::new(log));
    let r2 = r.clone();
    let vn = VatName("bootstrap".to_string());
    let setup =
        |syscall| -> Box<dyn Dispatch> { Box::new(Vat1Dispatch { syscall, log: r2 }) };
    let sb: Box<Setup> = Box::new(setup);
    cfg.add_vat(&vn, sb);
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
