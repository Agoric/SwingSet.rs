use swingset::{Config, Controller, Dispatch, VatName, VatSyscall};

#[derive(Debug)]
struct Vat1Dispatch {}
impl Dispatch for Vat1Dispatch {
    fn deliver(&mut self, _syscall: &mut VatSyscall) -> () {
        println!("Vat1.deliver");
    }
}

fn main() {
    let mut cfg = Config::new();
    let vat1 = Vat1Dispatch {};
    cfg.add_vat(&VatName("vat1".to_string()), vat1);
    let mut c = Controller::new(cfg);
    //println!("controller: {:?}", c);
    println!("controller created");
    c.run();
}
