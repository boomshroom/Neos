use spin::Mutex;
use lazy_static::lazy::Lazy;

use core::ops::Deref;

use super::terminal::vga::FrameBuffer;
use super::terminal::Terminal;
use super::terminal::serial::SerialPort;

pub trait Module {
	fn receive_message(&mut self, op: u32, data: &[u8], out: &mut [u8]) -> u32;
}

pub type ModHandle = usize;

struct Handler {
	mods: &'static [&'static Mutex<Module>],
    current: ModHandle,
}

static MAIN : MainMod = MainMod;
static COUNT : Mutex<Counter> = Mutex::new(Counter(0));
static TERMINAL: Mutex<Terminal> = Mutex::new(Terminal::new(1, 4));
static HANDLER : Mutex<Handler> = Mutex::new(Handler{ mods: &[&MAIN, &FB, &COUNT, &TERMINAL, &SERIAL], current: 0 });

lazy_static! {
	static ref FB: Mutex<FrameBuffer> = Mutex::new(unsafe { FrameBuffer::new() });
    static ref SERIAL: Mutex<SerialPort> = Mutex::new(SerialPort::init());
}

impl Module for FB {
	fn receive_message(&self, op: u32, data: &[u8], output: &mut[u8]) -> u32 {
		FrameBuffer::receive_message(self.deref(), op, data, output)
	}
}

impl Module for SERIAL {
    fn receive_message(&self, op: u32, data: &[u8], output: &mut[u8]) -> u32 {
        SerialPort::receive_message(self.deref(), op, data, output)
    }
}

pub fn send_message<T: Module>(reciever: ModHandle, op: u32, data: &[u8], callback: Option<fn(&mut T, &[u8], u32)>) {
    let mut buffer = [0; 256];
    let (old_id, old, new) = {
    	let mut handler = HANDLER.lock();
    	let old = handler.current;
    	handler.current = reciever;
    	(old, handler.mods[old], handler.mods[reciever])
	};

    let status = new.receive_message(op, data, &mut buffer);
    {
    	HANDLER.lock().current = old_id;
    }

    callback.map(|callback| callback(old, &mut buffer, status));
}

pub fn pause() {
    // TODO
}

struct Counter (u32);

impl Module for Counter {
    fn receive_message(&mut self, _: u32, _: &[u8], out: &mut[u8]) -> u32 {
        self.0 += 1;
        if out.len() < 4 {
            1
        } else {
            out[0] = (self.0 & 0xFF) as u8;
            out[1] = ((self.0 >> 8) & 0xFF) as u8;
            out[2] = ((self.0 >> 16) & 0xFF) as u8;
            out[3] = ((self.0 >> 24) & 0xFF) as u8;
            0
        }
    }
}

struct MainMod;

impl Module for MainMod {
	fn receive_message(&mut self, _: u32, _: &[u8], _: &mut[u8]) -> u32 {
		0
	}
}

impl LockedModule for MainMod {
	fn receive_message(&self, _: u32, _: &[u8], _: &mut[u8]) -> u32 {
		0
	}
}