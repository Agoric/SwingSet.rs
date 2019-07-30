use super::super::kernel::Kernel;
use super::super::vat::{
    CapData, CapSlot, Dispatch, InboundTarget, Message, ObjectID, PromiseID, Resolution,
    Syscall,
};
use std::cell::RefCell;
use std::rc::Rc;

struct VatLeftDispatch {
    log: Rc<RefCell<Vec<u32>>>,
}
impl Dispatch for VatLeftDispatch {
    fn deliver(
        &mut self,
        syscall: &mut dyn Syscall,
        target: InboundTarget,
        msg: Message,
    ) -> () {
        println!("VatLeft.deliver {:?} {:?}", target, msg);
        match target {
            InboundTarget::Object(ObjectID(0)) => {
                // p1 = o1!bar(args)
                let t = CapSlot::Object(ObjectID(-1));
                let p1 = PromiseID(1);
                let vmsg = Message::new("bar", b"", &vec![], Some(p1));
                syscall.send(t, vmsg);
                syscall.subscribe(p1);
                self.log.borrow_mut().push(1);
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
        println!("VatLeft.notify_resolved {:?} {:?}", id, to);
        self.log.borrow_mut().push(3);
    }
}

struct VatRightDispatch {
    log: Rc<RefCell<Vec<u32>>>,
}
impl Dispatch for VatRightDispatch {
    fn deliver(
        &mut self,
        syscall: &mut dyn Syscall,
        target: InboundTarget,
        msg: Message,
    ) -> () {
        println!("VatRight.deliver {:?} {:?}", target, msg);
        match target {
            InboundTarget::Object(ObjectID(1)) => {
                assert_eq!(msg.result, Some(PromiseID(-1)));
                let o4 = CapSlot::Object(ObjectID(4));
                let r = Resolution::Data(CapData::new(b"resbody", &[o4]));
                syscall.resolve(msg.result.unwrap(), r);
                self.log.borrow_mut().push(2);
            }

            _ => panic!("unknown target {:?}", target),
        };
    }

    fn notify_resolved(&mut self, _: &mut dyn Syscall, _: PromiseID, _: Resolution) {
        panic!();
    }
}

#[test]
fn test_result_basic() {
    let log: Vec<u32> = vec![];
    let r = Rc::new(RefCell::new(log));
    let mut k = Kernel::new();
    let vleft = k.add_vat("left", Box::new(VatLeftDispatch { log: r.clone() }));
    let vright = k.add_vat("right", Box::new(VatRightDispatch { log: r.clone() }));
    let oleft = k.add_export(vleft, 0);
    k.add_import_export_pair(vleft, -1, vright, 1);

    k.push_deliver(oleft, "foo", Vec::from(""), &vec![]);
    assert_eq!(*r.borrow(), vec![]);
    k.step();
    assert_eq!(*r.borrow(), vec![1]);
    k.dump();

    k.step();
    assert_eq!(*r.borrow(), vec![1, 2]);

    k.dump();
    k.step();
    assert_eq!(*r.borrow(), vec![1, 2, 3]);
}
