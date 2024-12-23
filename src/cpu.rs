use std::sync::{Arc, Mutex};

use addressing::{adressing_from_operand, register_from_operand, AddressingMode};
use commands::*;
use interruptions::InterruptionBus;

use crate::{mem::{MappedMemoryWord, Memory, SimpleMappedMemoryWord}, utils::*};

pub mod addressing;
pub mod interpreter;
pub mod interruptions;
pub mod debug;
pub mod commands;

pub const FIRST_COMMAND: Address = 0x0200;
pub const STACK_START: Address = 0x0200;

pub const FLAGS_IN_MEMORY: Address = 0xFFFE;

pub const REG_COUNT: usize = 8;

pub const MARK_POINTER_INDEX: Byte = 5; // Or MP
pub const STACK_POINTER_INDEX: Byte = 6; // Or SP
pub const PROGRAM_COUNTER_INDEX: Byte = 7; // Or PC

pub const CARRY_FLAG_INDEX: Byte = 0; // Or C
pub const OVERFLOW_FLAG_INDEX: Byte = 1; // Or V
pub const ZERO_FLAG_INDEX: Byte = 2; // Or Z
pub const NEGATIVE_FLAG_INDEX: Byte = 3; // Or N
pub const TRAP_FLAG_INDEX: Byte = 4; // Or T

pub const PRIORITY_LOW_BIT_INDEX: Byte = 5;
pub const PRIORITY_MIDDLE_BIT_INDEX: Byte = 6;
pub const PRIORITY_HIGH_BIT_INDEX: Byte = 7;

pub struct CPU {
    status: Arc<Mutex<SimpleMappedMemoryWord>>, // Or PSW (Processor Status Word)
    registers: [Word; REG_COUNT],
    commands: Arc<Commands>,
    running: Arc<Mutex<bool>>,
    waiting: bool,
    interruption_bus: Arc<Mutex<InterruptionBus>>,
}

// Constructors
impl CPU {
    pub fn new(commands: Arc<Commands>) -> Self {
        CPU {
            status: Arc::new(Mutex::new(SimpleMappedMemoryWord::new())),
            registers: [0; REG_COUNT],
            commands: commands,
            running: Arc::new(Mutex::new(false)),
            waiting: false,
            interruption_bus: Arc::new(Mutex::new(InterruptionBus::new())),
        }
    }
}

impl Default for CPU {
    fn default() -> Self {
        Self::new(Arc::new(Commands::default()))
    }
}

// Execution
impl CPU {
    pub fn running_flag(&self) -> Arc<Mutex<bool>> {
        self.running.clone()
    }

    pub fn interruption_bus(&self) -> Arc<Mutex<InterruptionBus>> {
        self.interruption_bus.clone()
    }

    pub fn run(&mut self, mem: Arc<Mutex<Memory>>) {
        self.map_status_word(mem.clone());

        *self.running.lock().unwrap() = true;
        self.set_word_reg(PROGRAM_COUNTER_INDEX, FIRST_COMMAND as Word);
        self.set_word_reg(STACK_POINTER_INDEX, STACK_START as Word);

        while *self.running.lock().unwrap() {
            trace!("tick");

            if !self.waiting {
                self.step(mem.clone());
                //self.trace_registers();
            }

            self.process_interruption_if_needed(mem.clone());
            //self.trace_registers();
        }

        self.unmap_status_word(mem.clone());
    }

    fn step(&mut self, mem: Arc<Mutex<Memory>>) {
        let mut memory = mem.lock().unwrap();

        let (address, command_word) = self.next_command(&mut memory);
    
        trace!("processing next instruction");
        trace!("address 0x{address:04X}");
        trace!("instruction 0x{command_word:04X}");

        let Command(command_opcode, command_name, command_interpreter) = 
            self.command(command_word);

        trace!("command 0x{command_opcode:04X} ({command_name})");  
        command_interpreter(self, &mut memory, command_word);

        if self.trap_flag() {
            self.do_bpt(&mut memory, 0x0000u16);
        }
    }

    fn next_command(&mut self, memory: &mut Memory) -> (Address, Word) {
        let address: Address = self.get_and_increment(PROGRAM_COUNTER_INDEX, Word::size_bytes().into()).into();

        let command: Word = memory.read_word(address);

        (address, command)
    }

    fn process_interruption_if_needed(&mut self, mem: Arc<Mutex<Memory>>) {
        if let Some(interruption_address) = self.get_interruption_address_if_any() {
            trace!("processing an interrupt from address 0x{interruption_address:04X}");

            self.waiting = false;
            let mut memory = mem.lock().unwrap();
            self.perform_trap(&mut memory, interruption_address);
        }
    }

    fn get_interruption_address_if_any(&mut self) -> Option<Address> {
        self.interruption_bus.lock().unwrap().next_interruption_if_any(self.current_priority())
    }

    fn map_status_word(&mut self, mem: Arc<Mutex<Memory>>) {
        mem.lock().unwrap().map_word(FLAGS_IN_MEMORY, self.status.clone());
    }

    fn unmap_status_word(&mut self, mem: Arc<Mutex<Memory>>) {
        mem.lock().unwrap().unmap_word(FLAGS_IN_MEMORY);
    }
}

// Put operand
impl CPU {
    fn put_byte_by_operand(&mut self, memory: &mut Memory, operand: Byte, byte: Byte) {
        self.put_byte(memory, register_from_operand(operand), adressing_from_operand(operand), byte);
    }

    fn put_word_by_operand(&mut self, memory: &mut Memory, operand: Byte, word: Word) {
        self.put_word(memory, register_from_operand(operand), adressing_from_operand(operand), word);
    }

    fn put_byte(&mut self, memory: &mut Memory, reg_index: Byte, addressing: AddressingMode, byte: Byte) {
        self.put_operand_value_with_addressing(memory, reg_index, addressing, byte, Memory::write_byte, CPU::set_byte_reg)
    }

    fn put_word(&mut self, memory: &mut Memory, reg_index: Byte, addressing: AddressingMode, word: Word) {
        self.put_operand_value_with_addressing(memory, reg_index, addressing, word, Memory::write_word, CPU::set_word_reg)
    }
}

// Get operand
impl CPU {
    fn get_byte_by_operand(&mut self, memory: &Memory, operand: Byte) -> Byte {
        self.get_byte(memory, register_from_operand(operand), adressing_from_operand(operand))
    }

    fn get_word_by_operand(&mut self, memory: &Memory, operand: Byte) -> Word {
        self.get_word(memory, register_from_operand(operand), adressing_from_operand(operand))
    }

    fn get_byte(&mut self, memory: &Memory, reg_index: Byte, addressing: AddressingMode) -> Byte {
        self.get_operand_value_with_addressing(memory, reg_index, addressing, Memory::read_byte, Self::get_byte_from_reg)
    }

    fn get_word(&mut self, memory: &Memory, reg_index: Byte, addressing: AddressingMode) -> Word {
        self.get_operand_value_with_addressing(memory, reg_index, addressing, Memory::read_word, Self::get_word_from_reg)
    }
}

// Get operand address
impl CPU {
    fn get_operand_address(&mut self, memory: &Memory, operand: Byte) -> Address {
        self.get_operand_address_with_addressing(memory, register_from_operand(operand), adressing_from_operand(operand))
    }
}

// Registers
impl CPU {
    fn get_word_from_reg(&mut self, reg_index: Byte) -> Word {
        self.registers[usize::from(reg_index)]
    }

    fn get_byte_from_reg(&mut self, reg_index: Byte) -> Byte {
        self.get_word_from_reg(reg_index).low()
    }

    fn get_and_increment(&mut self, reg_index: Byte, by: Word) -> Word {
        let result = self.get_word_from_reg(reg_index);
        self.increment_reg(reg_index, by);

        result
    }

    fn decrement_and_get(&mut self, reg_index: Byte, by: Word) -> Word {
        self.decrement_reg(reg_index, by);

        self.get_word_from_reg(reg_index)
    }

    fn increment_reg(&mut self, reg_index: Byte, by: Word) {
        self.registers[reg_index as usize] += by;
    }

    fn decrement_reg(&mut self, reg_index: Byte, by: Word) {
        self.registers[reg_index as usize] -= by;
    }

    fn set_byte_reg(&mut self, reg_index: Byte, value: Byte) {
        self.registers[reg_index as usize] = value.register();
    }

    fn set_word_reg(&mut self, reg_index: Byte, value: Word) {
        self.registers[reg_index as usize] = value;
    }
}

// Float registers
impl CPU {
    fn get_float_from_reg(&mut self, memory: &Memory, reg_index: Byte) -> f32 {
        let address = self.get_word_from_reg(reg_index);

        let hi_word = memory.read_word((address + 2).into());
        let lo_word = memory.read_word(address.into());

        f32::from_bits(long_word(lo_word, hi_word))
    }

    fn set_float_by_reg(&mut self, memory: &mut Memory, reg_index: Byte, value: f32) {
        let address = self.get_word_from_reg(reg_index);

        let long_word_value = value.to_bits();

        memory.write_word((address + 2).into(), long_word_value.high());
        memory.write_word(address.into(), long_word_value.low());
    }
}

// Stack 
impl CPU {
    fn push_stack(&mut self, memory: &mut Memory, word: Word) {
        self.put_word(memory, STACK_POINTER_INDEX, AddressingMode::Autodecrement, word);
    }

    fn pop_stack(&mut self, memory: &Memory) -> Word {
        self.get_word(memory, STACK_POINTER_INDEX, AddressingMode::Autoicrement)
    }
}

// Flags
impl CPU {
    fn update_status_flags_bitwise<T, N: Number<T>>(&mut self, result: N) {
        self.update_status_flags(result, self.carry_flag(), false);
    }

    fn update_status_flags<T, N: Number<T>>(&mut self, result: N, carry_bit: bool, overflow_bit: bool) {
        self.update_carry_flag(carry_bit);
        self.update_overflow_flag(overflow_bit);
        self.update_zero_flag(result.is_zero());
        self.update_negative_flag(result.is_negative());
    }

    fn update_carry_flag(&mut self, carry_bit: bool) {
        self.set_flag(CARRY_FLAG_INDEX, carry_bit);
    }

    fn update_overflow_flag(&mut self, overflow_bit: bool) {
        self.set_flag(OVERFLOW_FLAG_INDEX, overflow_bit);
    }

    fn update_zero_flag(&mut self, zero_bit: bool) {
        self.set_flag(ZERO_FLAG_INDEX, zero_bit);
    }

    fn update_negative_flag(&mut self, negative_bit: bool) {
        self.set_flag(NEGATIVE_FLAG_INDEX, negative_bit);
    }

    fn update_trap_flag(&mut self, trap_status: bool) {
        self.set_flag(TRAP_FLAG_INDEX, trap_status);
    }

    fn update_priority(&mut self, priority: Byte) {
        self.set_flag(PRIORITY_LOW_BIT_INDEX, priority.get_n_bit(0));
        self.set_flag(PRIORITY_MIDDLE_BIT_INDEX, priority.get_n_bit(1));
        self.set_flag(PRIORITY_HIGH_BIT_INDEX, priority.get_n_bit(2));
    }

    fn carry_flag(&self) -> bool {
        self.get_flag(CARRY_FLAG_INDEX)
    }

    fn overflow_flag(&self) -> bool {
        self.get_flag(OVERFLOW_FLAG_INDEX)
    }

    fn zero_flag(&self) -> bool {
        self.get_flag(ZERO_FLAG_INDEX)
    }

    fn negative_flag(&self) -> bool {
        self.get_flag(NEGATIVE_FLAG_INDEX)
    }

    fn trap_flag(&self) -> bool {
        self.get_flag(TRAP_FLAG_INDEX)
    }

    fn current_priority(&self) -> Byte {
        let low = self.get_flag(PRIORITY_LOW_BIT_INDEX);
        let middle = self.get_flag(PRIORITY_MIDDLE_BIT_INDEX);
        let high = self.get_flag(PRIORITY_HIGH_BIT_INDEX);

        0x00u8
            .set_n_bit(0, low)
            .set_n_bit(1, middle)
            .set_n_bit(2, high)
    }

    fn status_word(&self) -> Word {
        self.status.lock().unwrap().read_word()
    }

    fn set_status_word(&mut self, new_psw: Word) {
        self.status.lock().unwrap().write_word(new_psw);
    } 

    fn get_flag(&self, n: Byte) -> bool {
        self.status_word().get_n_bit(n)
    }

    fn set_flag(&mut self, n: Byte, value: bool) {
        let mut status_word = self.status.lock().unwrap();
        let status_flags = status_word.read_word();

        status_word.write_word(status_flags.set_n_bit(n, value));
    }
}

// Asserts
fn assert_not_pc(reg_index: &Byte) {
    assert!(*reg_index != PROGRAM_COUNTER_INDEX);
}

fn assert_pc(reg_index: &Byte) {
    assert!(*reg_index == PROGRAM_COUNTER_INDEX);
}

fn assert_even_reg(reg_index: &Byte) {
    assert!((*reg_index & 0x01) == 0x00);
}
