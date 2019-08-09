use super::super::kernel::{CapSlot as KernelCapSlot, Kernel};
use super::super::vat::{
    CapSlot, Dispatch, InboundTarget, Message, ObjectID, PromiseID, Resolution, Syscall,
};
use std::cell::RefCell;
use std::rc::Rc;

//#[derive(Debug)]
struct Vat1Dispatch {
    log: Rc<RefCell<Vec<u32>>>,
    //p: Option<PromiseID>,
}
impl Dispatch for Vat1Dispatch {
    fn deliver(
        &mut self,
        syscall: &mut dyn Syscall,
        target: InboundTarget,
        msg: Message,
    ) -> () {
        println!("Vat1.deliver {:?} {:?}", target, msg);
        match target {
            InboundTarget::Object(ObjectID(0)) => {
                println!(" deliver[0]");
                assert_eq!(msg.method, "bootstrap");
                assert_eq!(msg.args.body, b"body"); // TODO json
                assert_eq!(msg.args.slots, vec![CapSlot::Object(ObjectID(0))]);
                assert_eq!(msg.result, None);
                self.log.borrow_mut().push(1);
            }

            InboundTarget::Object(ObjectID(2)) => {
                println!(" deliver[2]");
                assert_eq!(msg.method, "foo");
                assert_eq!(msg.args.body, b"body");
                assert_eq!(msg.args.slots, vec![CapSlot::Object(ObjectID(2))],);
                assert_eq!(msg.result, None);
                self.log.borrow_mut().push(2);
            }

            InboundTarget::Object(ObjectID(3)) => {
                println!(" deliver[3]");
                assert_eq!(msg.method, "foo");
                assert_eq!(msg.args.body, b"foobody");
                assert_eq!(msg.args.slots, vec![CapSlot::Object(ObjectID(-4))],);
                assert_eq!(msg.result, None);
                self.log.borrow_mut().push(3);

                // o4!bar(barbody, [o22])
                let t = *msg.args.slots.get(0).unwrap();
                let arg1 = CapSlot::Object(ObjectID(22));
                let vmsg = Message::new("bar", b"barbody", &vec![arg1], None);
                syscall.send(t, vmsg);
                //assert_eq!(self.p, Some(VatPromiseID(0)));
                //println!(" got promise {:?}", self.p);

                /*
                assert_eq!(msg.resolver, Some(VatResolverID(0)));
                let arg2 = VatArgSlot::Export(VatExportID(23));
                let res = VatCapData {
                    body: b"result".to_vec(),
                    slots: vec![arg2],
                };
                self.syscall.fulfill_to_data(msg.resolver.unwrap(), res);
                println!(" did fulfill_to_data");
                self.log.borrow_mut().push(2);
                */
            }

            InboundTarget::Object(ObjectID(4)) => {
                assert_eq!(msg.method, "bar");
                assert_eq!(msg.args.body, b"barbody");
                assert_eq!(msg.args.slots, vec![CapSlot::Object(ObjectID(-1))],);
                assert_eq!(msg.result, None);
                self.log.borrow_mut().push(4);
            }

            _ => panic!("unknown target {:?}", target),
        };
    }

    fn notify_resolved(
        &mut self,
        _syscall: &mut dyn Syscall,
        id: PromiseID,
        to: Resolution,
    ) {
        println!("Vat1.notify_resolved {:?} {:?}", id, to);
    }
}

#[test]
fn test_basic() {
    let log: Vec<u32> = vec![];
    let r = Rc::new(RefCell::new(log));
    let r2 = r.clone();
    let d = Vat1Dispatch { log: r2 };
    let mut k = Kernel::new();
    let v1 = k.add_vat("bootstrap", Box::new(d));
    let o1 = k.add_export(v1, 2);

    k.push_bootstrap(v1);
    k.dump();

    assert_eq!(*r.borrow(), vec![]);
    k.run();
    k.dump();
    assert_eq!(*r.borrow(), vec![1]);

    k.push_deliver(
        o1,
        "foo",
        Vec::from("body"),
        &vec![KernelCapSlot::Object(o1)],
    );
    assert_eq!(*r.borrow(), vec![1]);
    k.run();
    assert_eq!(*r.borrow(), vec![1, 2]);
}

#[test]
fn test_syscall() {
    let log: Vec<u32> = vec![];
    let r = Rc::new(RefCell::new(log));
    let dleft = Vat1Dispatch { log: r.clone() };
    let dright = Vat1Dispatch { log: r.clone() };
    let mut k = Kernel::new();
    let vleft = k.add_vat("left", Box::new(dleft));
    let vright = k.add_vat("right", Box::new(dright));
    let o3 = k.add_export(vleft, 3);
    let o4 = k.add_import_export_pair(vleft, -4, vright, 4);

    k.push_deliver(
        o3,
        "foo",
        Vec::from("foobody"),
        &vec![KernelCapSlot::Object(o4)],
    );
    assert_eq!(*r.borrow(), vec![]);
    k.step();
    assert_eq!(*r.borrow(), vec![3]);
    k.dump();
    k.step();
    assert_eq!(*r.borrow(), vec![3, 4]);
}