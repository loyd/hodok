pub mod ahrs;
pub mod server;
pub mod sysinfo;
pub mod video;

use std::sync::Arc;
use std::sync::mpsc::{self, Sender, Receiver};
use std::thread;


pub trait Node: Send + Sized + 'static {
    fn new() -> Self;

    fn main(&mut self);

    fn start(mut self) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            self.main();
        })
    }
}

pub struct Input<I: Send + Sync>(Sender<Arc<I>>, Receiver<Arc<I>>);

impl<I: Send + Sync> Input<I> {
    fn new() -> Input<I> {
        let chann = mpsc::channel();
        Input(chann.0, chann.1)
    }

    fn recv(&self) -> Arc<I> {
        self.1.recv().unwrap()
    }
}

pub struct Output<O: Send + Sync>(Vec<Sender<Arc<O>>>);

impl<O: Send + Sync> Output<O> {
    fn new() -> Output<O> {
        Output(vec![])
    }

    pub fn pipe(&mut self, dest: &Input<O>) {
        self.0.push(dest.0.clone())
    }

    fn send(&self, data: O) {
        let arc = Arc::new(data);

        for receiver in self.0.iter() {
            receiver.send(arc.clone()).unwrap();
        }
    }
}


pub fn timer(rate: f32) -> Receiver<()> {
    let period = (1000./rate).ceil() as u32;
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        thread::sleep_ms(period);
        tx.send(()).unwrap();
    });

    rx
}
