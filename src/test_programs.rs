use std::sync::{Arc, Mutex};

use crate::{cpu::{debug::CPUStateDump, CPU, FIRST_COMMAND, REG_COUNT}, mem::Memory, utils::{Byte, Word}};


pub fn test_cpu(cpu: &mut CPU) {
    test_mov_add(cpu, 3, 3);
    test_mov_sub(cpu, 3, 3);
}

pub fn test_mov_add(cpu: &mut CPU, a: Word, b: Word) {
    run_test("ADD command", cpu, 
        |cpu| {
            run_and_dump(cpu, make_add_test(a, b))
        },
        |dump| {
            assert!(dump.registers[0] == a + b);
        }
    );
}

pub fn test_mov_sub(cpu: &mut CPU, a: Word, b: Word) {
    run_test("SUB command", cpu, 
        |cpu| {
            run_and_dump(cpu, make_sub_test(a, b))
        },
        |dump| {
            assert!(dump.registers[0] == a - b);
        }
    );
}

fn run_and_dump(cpu: &mut CPU, memory: Arc<Mutex<Memory>>) -> CPUStateDump {
    cpu.run(memory);
    cpu.dump_state()
}

fn run_test(name: &'static str, cpu: &mut CPU, run: impl Fn(&mut CPU) -> CPUStateDump, validate: impl Fn(&CPUStateDump) -> ()) {
    trace!("Test: {name}");

    let dump = run(cpu);
    validate(&dump);

    trace!("Passed!");
}

fn make_sub_test(a: Word, b: Word) -> Arc<Mutex<Memory>> {
    make_two_operands_test(0x0E000, b, a)
}

fn make_add_test(a: Word, b: Word) -> Arc<Mutex<Memory>> {
    make_two_operands_test(0x6000, b, a)
}

fn make_two_operands_test(opcode: Word, src: Word, dst: Word) -> Arc<Mutex<Memory>> {
    let mem = Memory::new();

    let mem_binding = mem.clone();
    let mut memory = mem_binding.lock().unwrap();

    let mut address = FIRST_COMMAND;

    let src_reg: Byte = 1;
    let dst_reg: Byte = 0;

    address = memory.write_word(address, mov_const(dst_reg));
    address = memory.write_word(address, dst);
    address = memory.write_word(address, mov_const(src_reg));
    address = memory.write_word(address, src);
    address = memory.write_word(address, make_two_cmd(opcode, src_reg, dst_reg));

    mem
}

fn mov_const(reg: Byte) -> Word {
    assert!(reg < (REG_COUNT as Byte));

    0x15C0 | (reg as Word)
}

fn make_two_cmd(opcode: Word, src: Byte, dst: Byte) -> Word {
    opcode | ((src as Word) << 6) | dst as Word
}
