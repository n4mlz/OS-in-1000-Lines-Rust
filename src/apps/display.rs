extern crate alloc;

use core::fmt::Write;

use crate::ipc::{Ipc, Message, Src};
use crate::process::Pid;
use crate::{print, println};

pub const DISPLAY_SERVER_PID: Pid = Pid::new(1);

const QUAD_WIDTH: u8 = 105;
const QUAD_HEIGHT: u8 = 25;
const SEPARATOR_COLOR: u8 = 0;

fn ansi_clear_screen() {
    print!("\x1B[2J");
    ansi_move_cursor(1, 1);
}

fn ansi_move_cursor(row: u16, col: u16) {
    print!("\x1B[{};{}H", row, col);
}

fn ansi_set_fg(color: u8) {
    print!("\x1B[38;5;{}m", color);
}

fn ansi_set_bg(color: u8) {
    print!("\x1B[48;5;{}m", color);
}

fn ansi_reset() {
    print!("\x1B[0m");
}

fn quadrant_origin(display: u8) -> (u16, u16) {
    match display {
        0 => (1, 1),
        1 => (1, QUAD_WIDTH as u16 + 2),
        2 => (QUAD_HEIGHT as u16 + 2, 1),
        3 => (QUAD_HEIGHT as u16 + 2, QUAD_WIDTH as u16 + 2),
        _ => (1, 1),
    }
}

fn draw_separators() {
    let horiz_row = QUAD_HEIGHT as u16 + 1;
    ansi_set_fg(SEPARATOR_COLOR);
    ansi_move_cursor(horiz_row, 1);
    for _ in 0..(QUAD_WIDTH as u16 * 2 + 2) {
        print!("━");
    }

    let vert_col = QUAD_WIDTH as u16 + 1;
    for row in 1..(QUAD_HEIGHT as u16 * 2 + 2) {
        ansi_move_cursor(row, vert_col);
        print!("┃");
    }
    ansi_move_cursor(horiz_row, vert_col);
    print!("╋");
    ansi_reset();
}

fn clear_quadrant(display: u8) {
    let (orow, ocol) = quadrant_origin(display);
    for y in 0..QUAD_HEIGHT {
        ansi_move_cursor(orow + y as u16, ocol);
        for _ in 0..QUAD_WIDTH {
            print!(" ");
        }
    }
}

fn print_line_in_quad(display: u8, line: u8, text: &str) {
    let (orow, ocol) = quadrant_origin(display);
    let line = if line < QUAD_HEIGHT {
        line
    } else {
        QUAD_HEIGHT - 1
    };
    ansi_move_cursor(orow + line as u16, ocol);
    let mut printed = 0;
    for ch in text.chars() {
        if printed >= QUAD_WIDTH {
            break;
        }
        print!("{}", ch);
        printed += 1;
    }
    for _ in printed..QUAD_WIDTH {
        print!(" ");
    }
}

fn draw_cell(display: u8, x: u8, y: u8, fg: u8, bg: u8, ch: char) {
    if x >= QUAD_WIDTH || y >= QUAD_HEIGHT {
        return;
    }
    let (orow, ocol) = quadrant_origin(display);
    ansi_move_cursor(orow + y as u16, ocol + x as u16);
    ansi_set_fg(fg);
    ansi_set_bg(bg);
    print!("{}", ch);
    ansi_reset();
}

pub fn display_server() -> ! {
    ansi_clear_screen();
    draw_separators();

    loop {
        let msg = match Ipc::recv(Src::Any) {
            Ok(pair) => pair,
            Err(e) => {
                println!("DisplayServer: recv_any failed: {:?}", e);
                continue;
            }
        };

        match msg {
            Message::DisplayPrint {
                display,
                line,
                text,
                len,
            } => {
                let slice = &text[..len as usize];
                let s = str::from_utf8(slice).unwrap_or("<invalid utf8>");
                print_line_in_quad(display, line, s);
            }
            Message::DisplayClear(display) => {
                clear_quadrant(display);
            }
            Message::DisplayDrawCell {
                display,
                x,
                y,
                fg,
                bg,
                ch,
            } => {
                draw_cell(display, x, y, fg, bg, ch);
            }
            other => {
                println!("DisplayServer: unexpected message {:?}", other);
            }
        }
    }
}
