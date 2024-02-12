use std::time::Duration;

use lunatic::ap::{AbstractProcess, Config, ProcessRef};
use lunatic::{abstract_process, Mailbox};

pub struct A {
    self_ref: ProcessRef<A>,
    b: Option<ProcessRef<B>>,
}

#[abstract_process(visibility = pub)]
impl A {
    #[init]
    fn init(this: Config<Self>, _: ()) -> Result<Self, ()> {
        this.self_ref()
            .with_delay(Duration::from_millis(100))
            .tick1();
        Ok(Self {
            self_ref: this.self_ref(),
            b: None,
        })
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

    #[handle_message]
    pub fn tick1(&mut self) {
        println!("A::tick1()");
        self.self_ref.with_delay(Duration::from_millis(100)).tick1();
    }

    #[handle_request]
    pub fn test1(&self) -> u32 {
        println!("A::test1()");
        self.b.unwrap().req2()
    }
}

pub struct B {
    self_ref: ProcessRef<B>,
    a: Option<ProcessRef<A>>,
}

#[abstract_process(visibility = pub)]
impl B {
    #[init]
    fn init(this: Config<Self>, _: ()) -> Result<Self, ()> {
        this.self_ref()
            .with_delay(Duration::from_millis(100))
            .tick2();
        Ok(Self {
            self_ref: this.self_ref(),
            a: None,
        })
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

    #[handle_message]
    pub fn tick2(&mut self) {
        println!("A::tick2()");
        self.self_ref.with_delay(Duration::from_millis(100)).tick2();
    }
}

struct Timer {
    interval: Duration,
    self_ref: ProcessRef<Timer>,
}

#[abstract_process]
impl Timer {
    #[init]
    fn init(this: Config<Self>, interval: Duration) -> Result<Self, ()> {
        this.self_ref().with_delay(interval).tick();
        Ok(Self {
            interval,
            self_ref: this.self_ref(),
        })
    }

    #[handle_message]
    pub fn tick(&mut self) {
        println!("Timer::tick()");
        self.self_ref.with_delay(self.interval).tick();
    }
}

#[lunatic::main]
fn main(_: Mailbox<()>) {
    Timer::link().start(Duration::from_millis(1000)).unwrap();
    let a = A::link().start(()).unwrap();
    let b = B::link().start(()).unwrap();

    a.connect1(b);
    b.connect2(a);

    a.tick1();
    let res = a.with_timeout(Duration::from_millis(5000)).test1();
    println!("res(1)={res:?}");
    a.tick1();
    let res = a.with_timeout(Duration::from_millis(5000)).test1();
    println!("res(2)={res:?}");
    a.tick1();
    let res = a.test1();
    println!("res(3)={res:?}");
    a.tick1();
}
