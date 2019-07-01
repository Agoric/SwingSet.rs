use std::cell::RefCell;
use std::rc::Rc;
use swingset::{
    Config, Controller, Dispatch, InboundVatMessage, OutboundVatMessage, Setup, Syscall,
    VatArgSlot, VatCapData, VatExportID, VatImportID, VatName, VatPromiseID,
    VatResolveTarget, VatResolverID, VatSendTarget,
};

//#[derive(Debug)]
struct Vat1Dispatch {
    syscall: Box<dyn Syscall>,
    log: Rc<RefCell<Vec<u32>>>,
    r: Option<VatResolverID>,
}
impl Dispatch for Vat1Dispatch {
    fn deliver(&mut self, target: VatExportID, message: InboundVatMessage) -> () {
        println!("Vat1.deliver {} .{}", target, message.name);
        assert_eq!(target, VatExportID(0), "unexpected target");

        if message.name == "bootstrap" {
            let (p, r) = self.syscall.allocate_promise_and_resolver();
            self.r = Some(r);
            let t = VatSendTarget::Import(VatImportID(1));
            let arg1 = VatArgSlot::Promise(p);
            let vmsg = OutboundVatMessage::new("have_promise", b"body", vec![arg1]);
            self.syscall.send_only(t, vmsg);
            self.log.borrow_mut().push(1);
        } else if message.name == "resolve_data" {
            let slot0 = VatArgSlot::Export(VatExportID(10));
            let slot1 = VatArgSlot::Import(VatImportID(1));
            let data = VatCapData {
                body: b"p2data".to_vec(),
                slots: vec![slot0, slot1],
            };
            self.syscall.fulfill_to_data(self.r.unwrap(), data);
            self.log.borrow_mut().push(3);
        } else if message.name == "resolve_target" {
            let slot0 = VatResolveTarget::Export(VatExportID(10));
            //let slot1 = VatResolveTarget::Import(VatImportID(1));
            self.syscall.fulfill_to_target(self.r.unwrap(), slot0);
            self.log.borrow_mut().push(3);
        } else if message.name == "reject" {
            let slot0 = VatArgSlot::Export(VatExportID(10));
            let slot1 = VatArgSlot::Import(VatImportID(1));
            let data = VatCapData {
                body: b"p2data".to_vec(),
                slots: vec![slot0, slot1],
            };
            self.syscall.reject(self.r.unwrap(), data);
            self.log.borrow_mut().push(3);
        } else {
            panic!("unknown target {}", target);
        }
    }

    fn notify_fulfill_to_target(&mut self, _id: VatPromiseID, _target: VatResolveTarget) {
        panic!();
    }
    fn notify_fulfill_to_data(&mut self, _id: VatPromiseID, _data: VatCapData) {
        panic!();
    }
    fn notify_reject(&mut self, _id: VatPromiseID, _data: VatCapData) {
        panic!()
    }
}

struct Vat2Dispatch {
    syscall: Box<dyn Syscall>,
    log: Rc<RefCell<Vec<u32>>>,
    p: Option<VatPromiseID>,
}
impl Dispatch for Vat2Dispatch {
    fn deliver(&mut self, target: VatExportID, message: InboundVatMessage) -> () {
        println!("Vat2.deliver {} .{}", target, message.name);
        assert_eq!(target, VatExportID(0), "unexpected target");

        if message.name == "have_promise" {
            if let VatArgSlot::Promise(p) = message.args.slots[0] {
                self.p = Some(p);
            } else {
                println!("args.slots[0] was not a Promise: {:?}", message.args.slots);
                panic!("args.slots[0] was not a Promise");
            }
            self.syscall.subscribe(self.p.unwrap());
            self.log.borrow_mut().push(2);
        } else {
            panic!("unknown target {}", target);
        }
    }

    fn notify_fulfill_to_target(&mut self, id: VatPromiseID, target: VatResolveTarget) {
        println!("Vat2.notify_fulfill_to_target {} {:?}", id, target);
        assert_eq!(id, self.p.unwrap());
        assert_eq!(target, VatResolveTarget::Import(VatImportID(0)));
        self.log.borrow_mut().push(41);
    }

    fn notify_fulfill_to_data(&mut self, id: VatPromiseID, data: VatCapData) {
        println!("Vat2.notify_fulfill_to_data {} {:?}", id, data);
        assert_eq!(id, self.p.unwrap());
        assert_eq!(data.body, b"p2data");
        assert_eq!(
            data.slots,
            vec![
                VatArgSlot::Import(VatImportID(0)),
                VatArgSlot::Export(VatExportID(0)),
            ]
        );
        self.log.borrow_mut().push(40);
    }

    fn notify_reject(&mut self, id: VatPromiseID, data: VatCapData) {
        println!("Vat2.notify_reject {} {:?}", id, data);
        assert_eq!(id, self.p.unwrap());
        assert_eq!(data.body, b"p2data");
        assert_eq!(
            data.slots,
            vec![
                VatArgSlot::Import(VatImportID(0)),
                VatArgSlot::Export(VatExportID(0)),
            ]
        );
        self.log.borrow_mut().push(42);
    }
}

fn do_test_subscribe_unresolved(mode: &str, expected_log: u32) {
    let mut cfg = Config::new();
    let log: Vec<u32> = vec![];
    let r = Rc::new(RefCell::new(log));
    let r2 = r.clone();
    let vn = VatName("bootstrap".to_string());
    let setup = |syscall| -> Box<dyn Dispatch> {
        Box::new(Vat1Dispatch {
            syscall,
            log: r2,
            r: None,
        })
    };
    let sb: Box<Setup> = Box::new(setup);
    cfg.add_vat(&vn, sb);

    let r3 = r.clone();
    let setup2 = |syscall| -> Box<dyn Dispatch> {
        Box::new(Vat2Dispatch {
            syscall,
            log: r3,
            p: None,
        })
    };
    let vn2 = VatName("vat2".to_string());
    let sb2: Box<Setup> = Box::new(setup2);
    cfg.add_vat(&vn2, sb2);

    let mut c = Controller::new(cfg);
    c.add_import(&vn, 1, &vn2, 0);
    c.start();

    c.step();
    assert_eq!(*r.borrow(), vec![1]);

    c.step();
    assert_eq!(*r.borrow(), vec![1, 2]);

    c.push("bootstrap", 0, mode, b"body");
    c.step();
    assert_eq!(*r.borrow(), vec![1, 2, 3]);

    c.step();
    assert_eq!(*r.borrow(), vec![1, 2, 3, expected_log]);
}

#[test]
fn test_subscribe_unresolved_data() {
    do_test_subscribe_unresolved("resolve_data", 40);
}

#[test]
fn test_subscribe_unresolved_target() {
    do_test_subscribe_unresolved("resolve_target", 41);
}

#[test]
fn test_subscribe_unresolved_reject() {
    do_test_subscribe_unresolved("reject", 42);
}
