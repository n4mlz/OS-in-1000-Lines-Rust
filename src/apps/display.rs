use core::fmt::Write;

use crate::{
    print, println,
    process::{PM, Pid},
};

pub fn display_server() -> ! {
    println!("Display server started");

    if PM.current_pid() != Pid::new(1) {
        panic!("Display server must run as PID 1");
    }

    loop {
        PM.switch();
    }
}
