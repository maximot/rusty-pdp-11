use std::{io::Write, sync::{Arc, Mutex}, thread, time::Duration};

use console::Term;

use crate::{cpu::{interruptions::InterruptionBus, CPU}, mem::{MappedMemoryWord, Memory, SimpleMappedMemoryWord}, utils::{blocking_queue::BlockingQueue, Address, Byte, Number, Word}};

pub const RECEIVER_STATUS_ADDRESS: Address = 0xFF70;
pub const RECEIVER_BUFFER_ADDRESS: Address = 0xFF72;
pub const TRANSMITTER_STATUS_ADDRESS: Address = 0xFF74;
pub const TRANSMITTER_BUFFER_ADDRESS: Address = 0xFF76;

pub const RDY_STATUS_BIT: Byte = 0x07;
pub const INT_STATUS_BIT: Byte = 0x06;

pub const INT_PRIORITY: Byte = 0x04;

pub const RECEIVER_INT: Address = 0x0030;
pub const TRANSMITTER_INT: Address = 0x0034;

struct TtyMappedMemoryWord {
    has_new_data: Mutex<bool>,
    word: SimpleMappedMemoryWord,
}

impl TtyMappedMemoryWord {
    pub fn new() -> Self {
        TtyMappedMemoryWord {
            has_new_data: Mutex::new(false),
            word: SimpleMappedMemoryWord::new(),
        }
    }

    fn has_new_data(&self) -> bool {
        *self.has_new_data.lock().unwrap()
    }
}

impl MappedMemoryWord for TtyMappedMemoryWord {
    fn read_word(&self) -> Word {
        let mut has_new_data = self.has_new_data.lock().unwrap();
        *has_new_data = false;

        self.word.read_word()
    }

    fn write_word(&mut self, word: Word) {
        self.word.write_word(word);

        let mut has_new_data = self.has_new_data.lock().unwrap();
        *has_new_data = true;
    }
}

pub struct Dl11Tty {
    receiver_queue: Arc<BlockingQueue<Byte>>,

    receiver_status: Arc<Mutex<TtyMappedMemoryWord>>,
    receiver_buffer: Arc<Mutex<TtyMappedMemoryWord>>,
    transmitter_status: Arc<Mutex<TtyMappedMemoryWord>>,
    transmitter_buffer: Arc<Mutex<TtyMappedMemoryWord>>,
}

impl Dl11Tty {
    pub fn new() -> Self {
        Dl11Tty {
            receiver_queue: Arc::new(BlockingQueue::new()),

            receiver_status: Arc::new(Mutex::new(TtyMappedMemoryWord::new())),
            receiver_buffer: Arc::new(Mutex::new(TtyMappedMemoryWord::new())),
            transmitter_status: Arc::new(Mutex::new(TtyMappedMemoryWord::new())),
            transmitter_buffer: Arc::new(Mutex::new(TtyMappedMemoryWord::new())),
        }
    }
}

impl Dl11Tty {
    pub fn run(&mut self, interruption_bus: Arc<Mutex<InterruptionBus>>, mem: Arc<Mutex<Memory>>, running_flag: Arc<Mutex<bool>>) {
        self.map_registers(mem.clone());
        trace!("tty start");

        self.set_printing(false);

        let thread_active_flag = Arc::new(Mutex::new(true));

        let thread_active_flag_clone = thread_active_flag.clone();
        let reciever_queue = self.receiver_queue.clone();
        let stdin_loop = thread::spawn(move || { stdin_loop(reciever_queue, thread_active_flag_clone); });
        
        while *running_flag.lock().unwrap() {
            trace!("tty tick");
            self.try_print(interruption_bus.clone());
            self.try_receive(interruption_bus.clone());
            thread::sleep(Duration::from_millis(32));
        }

        trace!("tty stop");
        *thread_active_flag.lock().unwrap() = false;
        let _ = stdin_loop.join();
        self.unmap_registers(mem.clone());
    }

    fn map_registers(&mut self, mem: Arc<Mutex<Memory>>) {
        let mut memory = mem.lock().unwrap();

        memory.map_word(RECEIVER_STATUS_ADDRESS, self.receiver_status.clone());
        memory.map_word(RECEIVER_BUFFER_ADDRESS, self.receiver_buffer.clone());
        memory.map_word(TRANSMITTER_STATUS_ADDRESS, self.transmitter_status.clone());
        memory.map_word(TRANSMITTER_BUFFER_ADDRESS, self.transmitter_buffer.clone());
    }

    fn unmap_registers(&mut self, mem: Arc<Mutex<Memory>>) {
        let mut memory = mem.lock().unwrap();

        memory.unmap_word(RECEIVER_STATUS_ADDRESS);
        memory.unmap_word(RECEIVER_BUFFER_ADDRESS);
        memory.unmap_word(TRANSMITTER_STATUS_ADDRESS);
        memory.unmap_word(TRANSMITTER_BUFFER_ADDRESS);
    }
}


// Print impl
impl Dl11Tty {
    fn set_printing(&mut self, printing: bool) {
        let mut status = self.transmitter_status.lock().unwrap();

        let current = status.read_word();

        let new = current.set_n_bit(RDY_STATUS_BIT, !printing);

        status.write_word(new);
    }

    fn print_from_buffer(&mut self) {
        let buffer = self.transmitter_buffer.lock().unwrap();

        let char = [buffer.read_byte(false)];

        let mut stdout = Term::stdout();

        let _ = stdout.write(&char);
        let _ = stdout.flush();
    }

    fn is_empty_transmitter(&self) -> bool {
        !self.transmitter_buffer.lock().unwrap().has_new_data()
    }

    fn notify_ready_to_print(&self, interruption_bus: Arc<Mutex<InterruptionBus>>) {
        if self.transmitter_status.lock().unwrap().read_word().get_n_bit(INT_STATUS_BIT) {
            interruption_bus.lock().unwrap().interrupt(TRANSMITTER_INT, INT_PRIORITY);
        }
    }
}

// Print
impl Dl11Tty {
    fn try_print(&mut self, interruption_bus: Arc<Mutex<InterruptionBus>>) {
        if self.is_empty_transmitter() {
            return;
        }

        self.set_printing(true);

        self.print_from_buffer();

        self.set_printing(false);
        self.notify_ready_to_print(interruption_bus);
    }
}

// Receive impl
impl Dl11Tty {
    fn has_received_data(&self) -> bool {
        self.receiver_buffer.lock().unwrap().has_new_data()
    }

    fn data_from_receiver(&self) -> Option<Byte> {
        self.receiver_queue.pop()
    }

    fn set_recived(&mut self, received: bool) {
        let mut status = self.receiver_status.lock().unwrap();

        let current = status.read_word();

        let new = current.set_n_bit(RDY_STATUS_BIT, received);

        status.write_word(new);
    }

    fn should_notify_received(&self) -> bool {
        let status = self.receiver_status.lock().unwrap().read_word();

        status.get_n_bit(INT_STATUS_BIT) && !status.get_n_bit(RDY_STATUS_BIT)
    }

    fn notify_received(&self, interruption_bus: Arc<Mutex<InterruptionBus>>) {
        interruption_bus.lock().unwrap().interrupt(RECEIVER_INT, INT_PRIORITY);
    }
}

// Receive
impl Dl11Tty {
    fn try_receive(&mut self, interruption_bus: Arc<Mutex<InterruptionBus>>) {
        if self.has_received_data() {
            let should_notify = self.should_notify_received();
            
            self.set_recived(true);
            if should_notify {
                self.notify_received(interruption_bus);
                
            }
            return;
        }
        self.set_recived(false);

        if let Some(char) = self.data_from_receiver() {
            self.receiver_buffer.lock().unwrap().write_byte(char, false);
        }
    }
}

// Wait for user input
fn blocking_get_next_char() -> Option<Byte> {

    let char = Term::stdout().read_char().ok()? as Byte;

    Some(char)
}

fn stdin_loop(reciever_queue: Arc<BlockingQueue<Byte>>, active_flag: Arc<Mutex<bool>>) {
    trace!("stdin start");
    while *active_flag.lock().unwrap() {
        trace!("stdin tick");
        if let Some(next_char) = blocking_get_next_char() {
            reciever_queue.push(next_char);
        };
    }
    trace!("stdin stop");
}