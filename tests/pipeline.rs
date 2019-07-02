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
    p_foo: Option<VatPromiseID>,
    p_bar: Option<VatPromiseID>,
}
impl Dispatch for Vat1Dispatch {
    fn deliver(&mut self, target: VatExportID, message: InboundVatMessage) -> () {
        println!("Vat1.deliver {} .{}", target, message.name);
        assert_eq!(target, VatExportID(0), "unexpected target");

        if message.name == "bootstrap" {
            println!("in bootstrap");
            let vmsg1 = OutboundVatMessage::new("foo", b"body", vec![]);
            let v1 = VatImportID(1);
            let p_foo = self.syscall.send(VatSendTarget::Import(v1), vmsg1);
            assert_eq!(p_foo, VatPromiseID(0));
            self.p_foo = Some(p_foo);

            let vmsg2 = OutboundVatMessage::new("bar", b"", vec![]);
            let p_bar = self.syscall.send(VatSendTarget::Promise(p_foo), vmsg2);
            self.p_bar = Some(p_bar);
            self.log.borrow_mut().push(100);
        } else {
            panic!("unknown target {}", target);
        }
    }

    fn deliver_promise(&mut self, _target: VatResolverID, _message: InboundVatMessage) {
        panic!();
    }

    fn notify_fulfill_to_target(&mut self, id: VatPromiseID, target: VatResolveTarget) {
        println!("Vat2.notify_fulfill_to_target {} {:?}", id, target);
        assert_eq!(id, self.p_foo.unwrap());
        assert_eq!(target, VatResolveTarget::Import(VatImportID(0)));
        self.log.borrow_mut().push(140);
    }

    fn notify_fulfill_to_data(&mut self, id: VatPromiseID, data: VatCapData) {
        println!("Vat2.notify_fulfill_to_data {} {:?}", id, data);
        assert_eq!(id, self.p_bar.unwrap());
        assert_eq!(data.body, b"bar_data");
        assert_eq!(data.slots, vec![]);
        self.log.borrow_mut().push(141);
    }

    fn notify_reject(&mut self, id: VatPromiseID, data: VatCapData) {
        println!("Vat2.notify_reject {} {:?}", id, data);
        assert_eq!(id, self.p_foo.unwrap());
        assert_eq!(data.body, b"p2data");
        assert_eq!(
            data.slots,
            vec![
                VatArgSlot::Import(VatImportID(0)),
                VatArgSlot::Export(VatExportID(0)),
            ]
        );
        self.log.borrow_mut().push(142);
    }
}

struct Vat2Dispatch {
    syscall: Box<dyn Syscall>,
    log: Rc<RefCell<Vec<u32>>>,
    r_foo: Option<VatResolverID>,
    r_bar: Option<VatResolverID>,
}
impl Dispatch for Vat2Dispatch {
    fn deliver(&mut self, target: VatExportID, message: InboundVatMessage) {
        println!("Vat2.deliver {} .{}", target, message.name);
        if target.0 == 0 {
            if message.name == "foo" {
                self.r_foo = message.resolver;
                self.log.borrow_mut().push(200);
            } else {
                panic!("unexpected message");
            }
        } else if target.0 == 1 {
            if message.name == "resolve_foo" {
                let t1 = VatResolveTarget::Export(VatExportID(12));
                self.syscall.fulfill_to_target(self.r_foo.unwrap(), t1);
                self.log.borrow_mut().push(201);
            } else if message.name == "resolve_bar" {
                let data = VatCapData {
                    body: b"bar_data".to_vec(),
                    slots: vec![],
                };
                self.syscall.fulfill_to_data(self.r_bar.unwrap(), data);
                self.log.borrow_mut().push(202);
            } else {
                panic!("unexpected message");
            }
        } else {
            panic!("unexpected target");
        }
    }

    fn deliver_promise(&mut self, target: VatResolverID, message: InboundVatMessage) {
        println!("Vat2.deliver_promise {} .{}", target, message.name);
        assert_eq!(target, self.r_foo.unwrap());
        assert_eq!(message.name, "bar");
        self.r_bar = message.resolver;
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

#[test]
fn test_pipeline() {
    let mut cfg = Config::new();
    let log: Vec<u32> = vec![];
    let r = Rc::new(RefCell::new(log));
    let r2 = r.clone();
    let vn = VatName("bootstrap".to_string());
    let setup = |syscall| -> Box<dyn Dispatch> {
        Box::new(Vat1Dispatch {
            syscall,
            log: r2,
            p_foo: None,
            p_bar: None,
        })
    };
    let sb: Box<Setup> = Box::new(setup);
    cfg.add_vat(&vn, sb);

    let r3 = r.clone();
    let setup2 = |syscall| -> Box<dyn Dispatch> {
        Box::new(Vat2Dispatch {
            syscall,
            log: r3,
            r_foo: None,
            r_bar: None,
        })
    };
    let vn2 = VatName("vat2".to_string());
    let sb2: Box<Setup> = Box::new(setup2);
    cfg.add_vat(&vn2, sb2);

    let mut c = Controller::new(cfg);
    c.add_import(&vn, 1, &vn2, 0);
    c.start();
    c.run();
    assert_eq!(*r.borrow(), vec![100, 200]);

    c.push("vat2", 1, "resolve_foo", b"body");

    c.run();
    assert_eq!(*r.borrow(), vec![100, 200, 201, 140]);

    c.push("vat2", 1, "resolve_bar", b"body");
    c.run();
    assert_eq!(*r.borrow(), vec![100, 200, 201, 140, 202, 141]);
}
