extern crate pretty_env_logger;
#[macro_use] extern crate log;

mod utils;
mod mem;
mod cpu;

mod test_programs;
use cpu::CPU;
use test_programs::test_cpu;

// TODO: IMPLEMENT CPU
// TODO: LOAD PROGRAMM
fn main() {
    pretty_env_logger::init();

    let mut cpu = CPU::default();

    test_cpu(&mut cpu);
}
