use std::any::{TypeId, Any};
use std::collections::HashMap;
use std::mem;
use std::sync::Arc;
use std::sync::mpsc::{self, Sender, Receiver};
use std::sync::{Once, ONCE_INIT};
use std::thread;
use std::time::Duration;


pub const STACK_SIZE: usize = 64 * 1024;

macro_rules! run_nodes {
    ($node:ident $($nodes:ident)*) => {
        let _ = ::std::thread::Builder::new()
            .name(stringify!($node).to_string())
            .stack_size(::base::node::STACK_SIZE)
            .spawn(::nodes::$node::worker).unwrap();
        info!("Starting {}...", stringify!($node));
        run_nodes!($($nodes)*);
    };

    () => ({
        let (_tx, rx) = ::std::sync::mpsc::channel::<()>();
        rx.recv().unwrap();
    });
}

pub type Input<I> = Receiver<Arc<I>>;
pub struct Output<O: Send + Sync>(Vec<Sender<Arc<O>>>);

impl<O: Send + Sync> Output<O> {
    pub fn send(&self, data: O) {
        match self.0.len() {
            0 => {},
            1 => unsafe { self.0.get_unchecked(0).send(Arc::new(data)).unwrap(); },
            _ => {
                let arc = Arc::new(data);

                for receiver in &self.0 {
                    receiver.send(arc.clone()).unwrap();
                }
            }
        };
    }
}

pub fn advertise<T: Send + Sync + Any>() -> &'static mut Output<T> {
    get_output()
}

pub fn subscribe<T: Send + Sync + Any + 'static>() -> Input<T> {
    let mut output = get_output::<T>();
    let (rx, tx) = mpsc::channel();
    output.0.push(rx);
    tx
}

fn get_output<T: Send + Sync + Any>() -> &'static mut Output<T> {
    // Associated statics are not yet implemented and generics over statics are forbidden, hence
    // we can use one static pointer to `HashMap<TypeId, Box<Any>>` (aka `AnyMap`).
    type AnyMap = HashMap<TypeId, Box<Any>>;
    static mut OUTPUT_MAP: *mut AnyMap = 0 as *mut AnyMap;
    static ONCE: Once = ONCE_INIT;

    ONCE.call_once(|| {
        unsafe { OUTPUT_MAP = mem::transmute(Box::new(AnyMap::new())) };
    });

    let static_ref = unsafe { &mut *OUTPUT_MAP };

    static_ref.entry(TypeId::of::<T>()).or_insert_with(|| {
        let output = Output::<T>(Vec::with_capacity(1));
        Box::new(output)
    }).downcast_mut::<Output<T>>().unwrap()
}

pub fn periodic(rate: f32) -> Receiver<()> {
    let period = (1e9/rate).ceil() as u32;
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        loop {
            thread::sleep(Duration::new(0, period));
            tx.send(()).unwrap();
        }
    });

    rx
}
