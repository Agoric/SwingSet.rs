use super::{
    Config, Controller, Dispatch, InboundVatMessage, OutboundVatMessage, Syscall,
    VatArgSlot, VatCapData, VatExportID, VatImportID, VatName, VatPromiseID,
    VatResolveTarget, VatResolverID, VatSendTarget,
};
use std::cell::RefCell;
use std::rc::Rc;

//#[derive(Debug)]
struct Vat1Dispatch {
    syscall: Box<dyn Syscall>,
    log: Rc<RefCell<Vec<u32>>>,
    p: Option<VatPromiseID>,
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
                let arg1 = VatArgSlot::Export(VatExportID(22));
                let vmsg = OutboundVatMessage::new("foo", b"body", vec![arg1]);
                self.p = Some(self.syscall.send(t, vmsg));
                assert_eq!(self.p, Some(VatPromiseID(0)));
                println!(" got promise {:?}", self.p);
            }
            VatExportID(2) => {
                println!(" deliver[2]");
                assert_eq!(message.name, "foo");
                assert_eq!(message.args.body, b"body");
                assert_eq!(
                    message.args.slots,
                    vec![VatArgSlot::Export(VatExportID(22))]
                );
                assert_eq!(message.resolver, Some(VatResolverID(0)));
                let arg2 = VatArgSlot::Export(VatExportID(23));
                let res = VatCapData {
                    body: b"result".to_vec(),
                    slots: vec![arg2],
                };
                self.syscall.fulfill_to_data(message.resolver.unwrap(), res);
                println!(" did fulfill_to_data");
                self.log.borrow_mut().push(2);
            }
            _ => panic!("unknown target {}", target),
        };
    }

    fn notify_fulfill_to_target(&mut self, id: VatPromiseID, target: VatResolveTarget) {
        println!("Vat1.notify_fulfill_to_target {} {:?}", id, target);
    }
    fn notify_fulfill_to_data(&mut self, id: VatPromiseID, data: VatCapData) {
        println!("Vat1.notify_fulfill_to_data {} {:?}", id, data);
        self.log.borrow_mut().push(3);
    }
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
    let setup = |syscall| -> Box<dyn Dispatch> {
        Box::new(Vat1Dispatch {
            syscall,
            log: r2,
            p: None,
        })
    };
    let sb: Box<Setup> = Box::new(setup);
    cfg.add_vat(&vn, sb);
    let mut c = Controller::new(cfg);
    c.add_import(&vn, 1, &vn, 2);
    //println!("controller: {:?}", c);
    println!("controller created");
    c.start();
    c.dump();

    println!("\ncalling c.step");
    c.step();
    assert_eq!(*r.borrow(), vec![1]);

    c.dump();
    println!("\ncalling c.step");
    c.step();
    assert_eq!(*r.borrow(), vec![1, 2]);

    c.dump();
    println!("\ncalling c.step");
    c.step();
    assert_eq!(*r.borrow(), vec![1, 2, 3]);
}
