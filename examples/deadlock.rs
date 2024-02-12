use std::time::Duration;

use lunatic::ap::{AbstractProcess, Config, ProcessRef};
use lunatic::{abstract_process, Mailbox};

struct A {
    b: Option<ProcessRef<B>>,
}

#[abstract_process]
impl A {
    #[init]
    fn init(_: Config<Self>, _: ()) -> Result<Self, ()> {
        Ok(Self { b: None })
    }

    #[handle_message]
    pub fn connect1(&mut self, b: ProcessRef<B>) {
        println!("A::connect1({b:?})");
        self.b = Some(b);
    }

    #[handle_request]
    pub fn req1(&self) -> u32 {
        println!("A::req1()");
        1
    }

    #[handle_request]
    pub fn test1(&self) -> u32 {
        println!("A::test1()");
        self.b.unwrap().req2()
    }
}

struct B {
    a: Option<ProcessRef<A>>,
}

#[abstract_process]
impl B {
    #[init]
    fn init(_: Config<Self>, _: ()) -> Result<Self, ()> {
        Ok(Self { a: None })
    }

    #[handle_message]
    pub fn connect2(&mut self, a: ProcessRef<A>) {
        println!("B::connect2({a:?})");
        self.a = Some(a);
    }

    #[handle_request]
    pub fn req2(&self) -> u32 {
        println!("B::req2()");
        if let Some(a) = self.a {
            a.req1()
        } else {
            2
        }
    }
}

#[lunatic::main]
fn main(_: Mailbox<()>) {
    let a = A::link().start(()).unwrap();
    let b = B::link().start(()).unwrap();

    a.connect1(b);
    b.connect2(a);

    let res = a.test1();
    println!("res(1)={res:?}");
    let res = a.with_timeout(Duration::from_millis(5000)).test1();
    println!("res(2)={res:?}");
    let res = a.with_timeout(Duration::from_millis(5000)).test1();
    println!("res(2)={res:?}");
}
