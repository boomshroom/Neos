use spin::Mutex;

use core::ops::DerefMut;
use core::any::Any;

use super::terminal::vga::FrameBuffer;
use super::terminal::Terminal;
use super::terminal::serial::SerialPort;

pub trait Module: Any + Send {
	fn receive_message(&mut self, op: u32, data: &[u8], out: &mut [u8]) -> u32;
}


pub type ModHandle = usize;

struct Handler {
	mods: &'static [&'static Mutex<Module>],
    current: ModHandle,
}

static MAIN : Mutex<MainMod> = Mutex::new(MainMod);
static COUNT : Mutex<Counter> = Mutex::new(Counter(0));
static TERMINAL: Mutex<Terminal> = Mutex::new(Terminal::new(1, 4));
static HANDLER : Mutex<Handler> = Mutex::new(Handler{ mods: &[&MAIN, &FB, &COUNT, &TERMINAL, &SERIAL], current: 0 });
static FB : Mutex<FB_LAZY> = Mutex::new(FB_LAZY);
static SERIAL : Mutex<SERIAL_LAZY> = Mutex::new(SERIAL_LAZY);

lazy_static! {
	static ref FB_LAZY: FrameBuffer = unsafe { FrameBuffer::new() };
    static ref SERIAL_LAZY: SerialPort = SerialPort::init();
}

impl Module for FB_LAZY {
	fn receive_message(&mut self, op: u32, data: &[u8], output: &mut[u8]) -> u32 {
		FrameBuffer::receive_message(self.deref_mut(), op, data, output)
	}
}

impl Module for SERIAL_LAZY {
    fn receive_message(&mut self, op: u32, data: &[u8], output: &mut[u8]) -> u32 {
        SerialPort::receive_message(self.deref_mut(), op, data, output)
    }
}

pub fn send_message<T: Module + Any>(reciever: ModHandle, op: u32, data: &[u8], callback: Option<fn(&mut T, &[u8], u32)>) {
    let mut buffer = [0; 256];
    let (old_id, old, new) = {
    	let mut handler = HANDLER.lock();
    	let old = handler.current;
    	handler.current = reciever;
    	(old, handler.mods[old], handler.mods[reciever])
	};

    let status = new.lock().receive_message(op, data, &mut buffer);
    {
    	HANDLER.lock().current = old_id;
    }

    callback.map(|callback| {
        let m = old.lock();
        callback(downcast_mut(m.deref_mut()).expect("Callback expected wrong module type."), &mut buffer, status)
    });
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

pub struct MainMod;

impl Module for MainMod {
	fn receive_message(&mut self, _: u32, _: &[u8], _: &mut[u8]) -> u32 {
		0
	}
}