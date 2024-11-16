use std::{sync::{Arc, Mutex}, thread::{self, JoinHandle}, time::Duration};

use crate::{cpu::CPU, mem::Memory, tty::Dl11Tty};

pub struct Pdp11 {
    memory: Arc<Mutex<Memory>>,
    cpu: CPU,
    dl11tty: Arc<Mutex<Dl11Tty>>,
}

impl Pdp11 {
    pub fn new() -> Self {
        let memory = Memory::new();
        let cpu = CPU::default();
        let dl11tty = Arc::new(Mutex::new(Dl11Tty::new()));

        Pdp11 {
            memory: memory,
            cpu: cpu,
            dl11tty: dl11tty,
        }
    }

    pub fn run(&mut self) {
        let dl11tty_thread = self.run_tty();

        self.run_cpu();

        let _ = dl11tty_thread.join();
    }

    pub fn run_async(mut self) -> JoinHandle<()> {
        thread::spawn(move || {
            self.run();
        })
    }

    fn run_tty(&mut self) -> JoinHandle<()> {
        let cpu_running_flag = self.cpu.running_flag();
        let interruption_bus = self.cpu.interruption_bus();

        let dl11tty_memory_clone = self.memory.clone();
        
        let dl11tty = self.dl11tty.clone();
        
        let dl11tty_thread = thread::spawn(move || {
            thread::sleep(Duration::from_secs(1));
            dl11tty.lock().unwrap().run(interruption_bus, dl11tty_memory_clone, cpu_running_flag);
        });

        dl11tty_thread
    }

    fn run_cpu(&mut self) {
        self.cpu.run(self.memory.clone());
    }
}