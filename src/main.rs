extern crate pretty_env_logger;
#[macro_use] extern crate log;

mod utils;
mod mem;
mod cpu;
mod tty;
mod assembly;

mod test_programs;
use assembly::Pdp11;
use cpu::CPU;
use test_programs::test_cpu;

fn main() {
    pretty_env_logger::init();
    run_cpu_tests();

    run_assembled_pdp_11();
}

fn run_assembled_pdp_11() {
    // TODO: LOAD PROGRAMM
    let assembly = Pdp11::new();

    let _ = assembly.run_async().join();
}

fn run_cpu_tests() {
    let mut cpu = CPU::default();

    test_cpu(&mut cpu);
}
