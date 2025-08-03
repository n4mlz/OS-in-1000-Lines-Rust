use crate::constants::PROCS_MAX;
use crate::process::Pid;
use crate::process::{PM, State};

#[derive(Clone, Copy, Debug)]
pub enum Message {
    Ping,
    Data {
        a: usize,
        b: usize,
    },

    DisplayPrint {
        display: u8, // 0..3
        line: u8,
        text: [u8; 32],
        len: u8,
    },
    DisplayClear(u8), // 0..3
    DisplayDrawCell {
        display: u8,
        x: u8,
        y: u8,
        fg: u8,
        bg: u8,
        ch: char,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Src {
    Specific(Pid),
    Any,
}

#[derive(Debug)]
pub enum IpcError {
    SelfSend,
    DeadlockDetected,
    SendQueueFull,
    UnexpectedState,
}

#[derive(Clone, Copy, Debug)]
pub struct SenderEntry {
    src: Pid,
    msg: Message,
}

#[derive(Clone, Copy, Debug)]
pub struct Ipc {
    // receiver 用
    pub waiting_for: Option<Src>,
    // sender 用
    pub pending_send: Option<(Pid, Message)>,
    // receiver 用
    pub senders: [Option<SenderEntry>; PROCS_MAX],
    pub inbox: Option<Message>,
}

impl Ipc {
    pub const fn new() -> Self {
        Ipc {
            waiting_for: None,
            pending_send: None,
            senders: [None; PROCS_MAX],
            inbox: None,
        }
    }

    pub fn send(dst: Pid, msg: Message) -> Result<(), IpcError> {
        let me = PM.current_pid();
        if me == dst {
            return Err(IpcError::SelfSend);
        }

        let me_idx = me.as_usize();
        let dst_idx = dst.as_usize();

        // deadlock detection
        {
            let dst_proc = PM.procs[dst_idx].borrow();
            if let Some((pending_dst, _)) = dst_proc.ipc.pending_send
                && pending_dst == me
            {
                let me_proc = PM.procs[me_idx].borrow();
                if let Some((my_dst, _)) = me_proc.ipc.pending_send
                    && my_dst == dst
                {
                    return Err(IpcError::DeadlockDetected);
                }
            }
        }

        let mut should_unblock = false;
        {
            let mut dst_proc = PM.procs[dst_idx].borrow_mut();
            if dst_proc.state == State::Blocked
                && let Some(waiting) = dst_proc.ipc.waiting_for
            {
                match waiting {
                    Src::Specific(p) if p == me => {
                        dst_proc.ipc.inbox = Some(msg);
                        dst_proc.ipc.waiting_for = None;
                        should_unblock = true;
                    }
                    Src::Any => {
                        dst_proc.ipc.inbox = Some(msg);
                        dst_proc.ipc.waiting_for = None;
                        should_unblock = true;
                    }
                    _ => {}
                }
            }
        }
        if should_unblock {
            PM.unblock(dst);
            return Ok(());
        }

        {
            let mut dst_proc = PM.procs[dst_idx].borrow_mut();
            for slot in dst_proc.ipc.senders.iter() {
                if let Some(entry) = slot
                    && entry.src == me
                {
                    return Err(IpcError::DeadlockDetected);
                }
            }
            if let Some(entry_slot) = dst_proc.ipc.senders.iter_mut().find(|s| s.is_none()) {
                *entry_slot = Some(SenderEntry { src: me, msg });
            } else {
                return Err(IpcError::SendQueueFull);
            }
        }

        {
            {
                let mut me_proc = PM.procs[me_idx].borrow_mut();
                if me_proc.ipc.pending_send.is_some() {
                    return Err(IpcError::DeadlockDetected);
                }
                me_proc.ipc.pending_send = Some((dst, msg));
            }
            PM.block_current();
        }

        PM.switch();

        {
            let mut me_proc = PM.procs[me_idx].borrow_mut();
            if me_proc.ipc.pending_send.is_some() {
                me_proc.ipc.pending_send = None;
                return Err(IpcError::UnexpectedState);
            }
        }

        Ok(())
    }
    pub fn recv(src: Src) -> Result<Message, IpcError> {
        let me = PM.current_pid();
        let me_idx = me.as_usize();

        if let Some((msg, sender)) = {
            let mut me_proc = PM.procs[me_idx].borrow_mut();

            if let Src::Specific(expected) = src {
                if let Some(pos) = me_proc
                    .ipc
                    .senders
                    .iter()
                    .position(|slot| matches!(slot, Some(entry) if entry.src == expected))
                {
                    if let Some(entry) = me_proc.ipc.senders[pos].take() {
                        {
                            let mut sender_proc = PM.procs[entry.src.as_usize()].borrow_mut();
                            sender_proc.ipc.pending_send = None;
                        }
                        me_proc.ipc.waiting_for = None;
                        Some((entry.msg, entry.src))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else if let Some(pos) = me_proc.ipc.senders.iter().position(|s| s.is_some()) {
                if let Some(entry) = me_proc.ipc.senders[pos].take() {
                    {
                        let mut sender_proc = PM.procs[entry.src.as_usize()].borrow_mut();
                        sender_proc.ipc.pending_send = None;
                    }
                    me_proc.ipc.waiting_for = None;
                    Some((entry.msg, entry.src))
                } else {
                    None
                }
            } else if let Some(msg) = me_proc.ipc.inbox.take() {
                return Ok(msg);
            } else {
                None
            }
        } {
            PM.unblock(sender);
            return Ok(msg);
        }

        {
            let mut me_proc = PM.procs[me_idx].borrow_mut();
            if me_proc.ipc.waiting_for.is_some() {
                return Err(IpcError::DeadlockDetected);
            }
            me_proc.ipc.waiting_for = Some(src);
        }
        PM.block_current();
        PM.switch();

        {
            let mut me_proc = PM.procs[me_idx].borrow_mut();

            if let Some(msg) = me_proc.ipc.inbox.take() {
                me_proc.ipc.waiting_for = None;
                return Ok(msg);
            }

            if let Some(Src::Specific(expected)) = me_proc.ipc.waiting_for
                && let Some(pos) = me_proc
                    .ipc
                    .senders
                    .iter()
                    .position(|slot| matches!(slot, Some(entry) if entry.src == expected))
                && let Some(entry) = me_proc.ipc.senders[pos].take()
            {
                {
                    let mut sender_proc = PM.procs[entry.src.as_usize()].borrow_mut();
                    sender_proc.ipc.pending_send = None;
                }
                me_proc.ipc.waiting_for = None;
                PM.unblock(entry.src);
                return Ok(entry.msg);
            }

            if matches!(me_proc.ipc.waiting_for, Some(Src::Any))
                && let Some(pos) = me_proc.ipc.senders.iter().position(|s| s.is_some())
                && let Some(entry) = me_proc.ipc.senders[pos].take()
            {
                {
                    let mut sender_proc = PM.procs[entry.src.as_usize()].borrow_mut();
                    sender_proc.ipc.pending_send = None;
                }
                me_proc.ipc.waiting_for = None;
                PM.unblock(entry.src);
                return Ok(entry.msg);
            }
        }

        Err(IpcError::UnexpectedState)
    }
}
