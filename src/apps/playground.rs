use crate::apps::display::DISPLAY_SERVER_PID;
use crate::ipc::{Ipc, Message};
use crate::process::PM;

pub fn send_print(display: u8, line: u8, text: &str) {
    let mut buf = [0u8; 32];
    let bytes = text.as_bytes();
    let len = bytes.len().min(32);
    buf[..len].copy_from_slice(&bytes[..len]);
    let _ = Ipc::send(
        DISPLAY_SERVER_PID,
        Message::DisplayPrint {
            display,
            line,
            text: buf,
            len: len as u8,
        },
    );
}

pub fn send_draw_cell(display: u8, x: u8, y: u8, fg: u8, bg: u8, ch: char) {
    let _ = Ipc::send(
        DISPLAY_SERVER_PID,
        Message::DisplayDrawCell {
            display,
            x,
            y,
            fg,
            bg,
            ch,
        },
    );
}

pub fn send_clear(display: u8) {
    let _ = Ipc::send(DISPLAY_SERVER_PID, Message::DisplayClear(display));
}

fn lfsr_next(state: &mut u32) -> u8 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    (x & 0xFF) as u8
}

pub fn proc_a() -> ! {
    let display = 0;
    send_clear(display);
    send_print(display, 0, "Matrix");

    const WIDTH: usize = 80;
    const HEIGHT: i8 = 20;
    let mut heads: [i8; WIDTH] = [-1; WIDTH];
    let mut lengths: [u8; WIDTH] = [0; WIDTH];
    let mut seed: u32 = 0x1234_5678;

    loop {
        for col in 0..WIDTH {
            if heads[col] < 0 {
                if (lfsr_next(&mut seed) & 7) == 0 {
                    heads[col] = 0;
                    lengths[col] = 3 + (lfsr_next(&mut seed) % 4);
                }
            } else {
                let head = heads[col];
                let len = lengths[col] as i8;

                let ch = {
                    let r = lfsr_next(&mut seed);
                    let idx = r % 36;
                    if idx < 10 {
                        (b'0' + idx) as char
                    } else {
                        (b'A' + (idx - 10)) as char
                    }
                };
                if head < HEIGHT {
                    send_draw_cell(display, col as u8, head as u8, 10, 0, ch);
                }

                for t in 1..len {
                    let y = head - t;
                    if (0..HEIGHT).contains(&y) {
                        send_draw_cell(display, col as u8, y as u8, 2, 0, ch);
                    }
                }

                if head - len >= 0 && head - len < HEIGHT {
                    send_draw_cell(display, col as u8, (head - len) as u8, 0, 0, ' ');
                }

                heads[col] += 1;
                if heads[col] - len > HEIGHT {
                    heads[col] = -1;
                }
            }
        }

        PM.switch();
    }
}

pub fn proc_b() -> ! {
    let display = 1;
    send_clear(display);
    send_print(display, 0, "Game of Life");

    const W: usize = 80;
    const H: usize = 20;
    const SIZE: usize = W * H;

    static mut CUR: [u8; SIZE] = [0; SIZE];
    static mut NEXT: [u8; SIZE] = [0; SIZE];

    unsafe {
        CUR[W + 2] = 1;
        CUR[2 * W + 3] = 1;
        CUR[3 * W + 1] = 1;
        CUR[3 * W + 2] = 1;
        CUR[3 * W + 3] = 1;
        let bx = 10;
        let by = 2;
        CUR[by * W + bx + 1] = 1;
        CUR[(by + 1) * W + bx + 1] = 1;
        CUR[(by + 2) * W + bx + 1] = 1;
    }

    loop {
        unsafe {
            for y in 0..H {
                for x in 0..W {
                    let idx = y * W + x;
                    if CUR[idx] != 0 {
                        send_draw_cell(display, x as u8, (y + 1) as u8, 2, 0, '■');
                    } else {
                        send_draw_cell(display, x as u8, (y + 1) as u8, 0, 0, ' ');
                    }
                }
            }

            for y in 0..H {
                for x in 0..W {
                    let mut neighbors = 0;
                    for dy in [-1isize, 0, 1] {
                        for dx in [-1isize, 0, 1] {
                            if dx == 0 && dy == 0 {
                                continue;
                            }
                            let nx = ((x as isize + dx + W as isize) % W as isize) as usize;
                            let ny = ((y as isize + dy + H as isize) % H as isize) as usize;
                            if CUR[ny * W + nx] != 0 {
                                neighbors += 1;
                            }
                        }
                    }
                    let idx = y * W + x;
                    NEXT[idx] = match (CUR[idx], neighbors) {
                        (1, 2) | (1, 3) => 1,
                        (0, 3) => 1,
                        _ => 0,
                    };
                }
            }

            for i in 0..SIZE {
                CUR[i] = NEXT[i];
            }
        }

        PM.switch();
    }
}

pub fn proc_c() -> ! {
    let display = 2;
    send_clear(display);
    send_print(display, 0, "Plasma effect");

    let mut t: u8 = 0;
    loop {
        for y in 0..20 {
            for x in 0..80 {
                let v = ((x as u8)
                    .wrapping_mul(3)
                    .wrapping_add((y as u8).wrapping_mul(5))
                    .wrapping_add(t.wrapping_mul(2)))
                    & 7;
                let bg = 1 + v;
                send_draw_cell(display, x as u8, (y + 1) as u8, 0, bg, ' ');
            }
        }

        t = t.wrapping_add(1);

        PM.switch();
    }
}

pub fn proc_d() -> ! {
    let display = 3;
    send_clear(display);
    send_print(display, 0, "Clock + heartbeat");

    let mut seconds: u32 = 0;
    let mut seed: u32 = 0xdead_beef;
    const HEARTBEAT_PERIOD: u32 = 8;

    fn xorshift32(state: &mut u32) -> u8 {
        let mut x = *state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        *state = x;
        (x & 0xFF) as u8
    }

    loop {
        let total = seconds % 86400;
        let hour = (total / 3600) as u8;
        let minute = ((total % 3600) / 60) as u8;
        let sec = (total % 60) as u8;

        let mut buf = [b'0'; 8];
        fn two_digits(out: &mut [u8], v: u8) {
            out[0] = b'0' + (v / 10);
            out[1] = b'0' + (v % 10);
        }
        two_digits(&mut buf[0..2], hour);
        buf[2] = b':';
        two_digits(&mut buf[3..5], minute);
        buf[5] = b':';
        two_digits(&mut buf[6..8], sec);

        let mut display_str = alloc::string::String::new();
        if (seconds % 30) == 0 && (xorshift32(&mut seed) & 3) == 0 {
            for &c in buf.iter().rev() {
                if (xorshift32(&mut seed) & 7) == 0 {
                    display_str.push('?');
                } else {
                    display_str.push(c as char);
                }
            }
        } else {
            for &c in buf.iter() {
                display_str.push(c as char);
            }
        }

        send_print(display, 1, &display_str);

        let beat_on = (seconds % HEARTBEAT_PERIOD) < 4;
        let heart_char = if beat_on { '♥' } else { ' ' };
        let color = if beat_on { 9 } else { 8 };
        send_draw_cell(display, 0, 3, color, 0, heart_char);

        seconds = seconds.wrapping_add(1);

        for _ in 0..20 {
            PM.switch();
        }
    }
}
