use core::fmt::Write;

use crate::ipc::{Ipc, Message, Src};
use crate::process::{PM, Pid};
use crate::{print, println};

pub fn proc_a() -> ! {
    let target = Pid::new(2);
    println!("A: sending Data to B");
    let data = Message::Data { a: 100, b: 200 };
    if let Err(e) = Ipc::send(target, data) {
        println!("A: send failed: {:?}", e);
    }

    println!("A: waiting for reply from B");
    match Ipc::recv(Src::Specific(target)) {
        Ok(Message::Ping) => {
            println!("A: got Ping from B -- success");
        }
        Ok(other) => {
            println!("A: unexpected reply: {:?}", other);
        }
        Err(e) => {
            println!("A: recv failed: {:?}", e);
        }
    }

    loop {
        PM.switch();
    }
}

pub fn proc_b() -> ! {
    let source = Pid::new(1);
    println!("B: waiting for Data from A");
    match Ipc::recv(Src::Specific(source)) {
        Ok(Message::Data { a, b }) => {
            println!("B: received Data from A: a={}, b={}", a, b);
        }
        Ok(other) => {
            println!("B: unexpected message: {:?}", other);
        }
        Err(e) => {
            println!("B: recv failed: {:?}", e);
        }
    }

    println!("B: replying with Ping to A");
    if let Err(e) = Ipc::send(source, Message::Ping) {
        println!("B: send failed: {:?}", e);
    }

    loop {
        PM.switch();
    }
}
