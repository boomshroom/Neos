use volatile::Volatile;
use spin::Once;
use core::cell::RefCell;
use core::convert::AsRef;

use super::super::module::Module;
use super::{Color, CLEAR, PUTC, PUTS, SETCOLOR};

struct FrameBufferImpl {
    fb: &'static mut Buffer,
    column: usize,
    color: ColorCode,
}

pub struct FrameBuffer(Once<RefCell<FrameBufferImpl>>);

unsafe impl Send for FrameBuffer {}

type Buffer = [[Volatile<Cell>; BUF_W]; BUF_H];

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Cell(u8, ColorCode);

#[derive(Debug, Clone, Copy)]
pub struct ColorCode(u8);

const TAB_WIDTH: usize = 8;

const BUF_H: usize = 25;
const BUF_W: usize = 80;

impl FrameBuffer {
    pub const fn new() -> FrameBuffer {
        FrameBuffer(Once::new())
    }

    fn inner<'a>(&'a mut self) -> &'a mut FrameBufferImpl {
        unsafe {
            &mut *self.0
                .call_once(|| RefCell::new(FrameBufferImpl::new()))
                .as_ptr()
        }
    }
}

impl Module for FrameBuffer {
    fn receive_message(&mut self, op: u32, data: &[u8], output: &mut [u8]) -> u32 {
        match op {
            PUTS => {
                self.inner().puts(data);
                output
                    .iter_mut()
                    .zip(data)
                    .for_each(|(out, &input)| *out = input);
                0
            }
            /*SETPOS => {
                output.iter_mut().for_each(|b| *b = 0);
				if data.len() != 2 {
					return 1
				}
				if data[0] >= 80 || data[1] >= 25 {
					return 2
				}
				self.pos = (data[0], data[1]);
				0
			}*/
            SETCOLOR => {
                output.iter_mut().for_each(|b| *b = 0);
                if data.len() != 1 {
                    return 1;
                }
                self.inner().color = ColorCode(data[0]);
                0
            }
            PUTC => {
                output.iter_mut().for_each(|b| *b = 0);
                if data.len() != 1 {
                    return 1;
                }
                output[0] = data[0];

                self.inner().putc(data[0]);
                0
            }
            CLEAR => {
                output.iter_mut().for_each(|b| *b = 0);
                self.inner().clear();
                0
            }
            _ => 3,
        }
    }
}

impl FrameBufferImpl {
    unsafe fn new() -> FrameBufferImpl {
        FrameBufferImpl {
            fb: &mut *(0xb8000 as *mut Buffer),
            column: 0,
            color: ColorCode::new(Color::LightGray, Color::Black),
        }
    }

    fn empty(&self) -> Cell {
        Cell(b' ', self.color)
    }

    fn puts<S: AsRef<[u8]>>(&mut self, s: S) {
        s.as_ref().iter().for_each(|&ch| self.putc(ch));
    }

    fn putc(&mut self, ch: u8) {
        match ch {
            b'\n' => self.newline(),
            b'\t' => self.tab(),
            ch => self.putc_raw(ch),
        }
    }

    fn putc_raw(&mut self, ch: u8) {
        if self.column >= BUF_W {
            self.newline();
        }
        let cell = self.make_cell(ch);
        let column = self.column;
        self.last_row_mut()[column].write(cell);
        self.column += 1;
    }

    fn newline(&mut self) {
        for row in 1..BUF_H {
            for col in 0..BUF_W {
                let ch = self.fb[row][col].read();
                self.fb[row - 1][col].write(ch);
            }
        }
        self.clear_row();
        self.column = 0;
    }

    fn tab(&mut self) {
        let p = self.column + TAB_WIDTH;
        self.column = p - (p % TAB_WIDTH);
    }

    fn clear_row(&mut self) {
        let empty = self.empty();
        self.last_row_mut().iter_mut().for_each(|c| c.write(empty));
    }

    fn last_row_mut(&mut self) -> &mut [Volatile<Cell>; BUF_W as usize] {
        self.fb.last_mut().unwrap()
    }

    fn make_cell(&self, ch: u8) -> Cell {
        Cell(ch, self.color)
    }

    fn clear(&mut self) {
        let empty = self.empty();
        self.fb
            .iter_mut()
            .flat_map(|row| row.iter_mut())
            .for_each(|cell| cell.write(empty));
    }
}

impl ColorCode {
    pub const fn new(fg: Color, bg: Color) -> ColorCode {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}
