use std::rc::Rc;

use addressing::{adressing_from_operand, register_from_operand, AddressingMode};
use commands::*;

use crate::{mem::Memory, utils::*};

pub mod addressing;
pub mod interpreter;
pub mod debug;
pub mod commands;

pub const FIRST_COMMAND: Address = 0x0200;

pub const FLAGS_IN_MEMORY: Address = 0xFFFE;

pub const REG_COUNT: usize = 8;

pub const STACK_POINTER_INDEX: Byte = 6; // Or SP
pub const PROGRAM_COUNTER_INDEX: Byte = 7; // Or PC

pub const CARRY_FLAG_INDEX: Byte = 0; // Or C
pub const OVERFLOW_FLAG_INDEX: Byte = 1; // Or V
pub const ZERO_FLAG_INDEX: Byte = 2; // Or Z
pub const NEGATIVE_FLAG_INDEX: Byte = 3; // Or N

// TODO: PROCESS COMMAND
// TODO: INTERUPTIONS?
pub struct CPU {
    status: Word, // Or PSW (Processor Status Word)
    registers: [Word; REG_COUNT],
    commands: Rc<Commands>,
    running: bool,
    waiting: bool,
}

// Constructors
impl CPU {
    pub fn new(commands: Rc<Commands>) -> Self {
        CPU {
            status: 0x0000,
            registers: [0; REG_COUNT],
            commands: commands,
            running: false,
            waiting: false,
        }
    }
}

impl Default for CPU {
    fn default() -> Self {
        Self::new(Rc::new(Commands::default()))
    }
}

// Execution
impl CPU {
    pub fn run(&mut self, memory: &mut Memory) {
        self.running = true;
        self.set_word_reg(PROGRAM_COUNTER_INDEX, FIRST_COMMAND as Word);

        while self.running {
            if !self.waiting {
                self.step(memory);
            }

            // TODO: INTERRUPTION + set waiting false

            //self.trace_registers();
        }
    }

    fn step(&mut self, memory: &mut Memory) {
        let (address, command_word) = self.next_command(memory);
    
        trace!("tick");
        trace!("address 0x{address:04X}");
        trace!("instruction 0x{command_word:04X}");

        let Command(command_opcode, command_name, command_interpreter) = 
            self.command(command_word);

        trace!("command 0x{command_opcode:04X} ({command_name})");  
        command_interpreter(self, memory, command_word);
    }

    fn next_command(&mut self, memory: &mut Memory) -> (Address, Word) {
        let address: Address = self.get_and_increment(PROGRAM_COUNTER_INDEX, WORD_SIZE_BYTES).into();

        let command: Word = memory.read_word(address);

        (address, command)
    }
}

// Put
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

// Get
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

// Flags
impl CPU {
    fn update_status_flags_bitwise<T, N: Number<T>>(&mut self, memory: &mut Memory, result: N) {
        self.update_status_flags(memory, result, self.carry_flag(), false);
    }

    fn update_status_flags<T, N: Number<T>>(&mut self, memory: &mut Memory, result: N, carry_bit: bool, overflow_bit: bool) {
        self.update_carry_flag(memory, carry_bit);
        self.update_overflow_flag(memory, overflow_bit);
        self.update_zero_flag(memory, result.is_zero());
        self.update_negative_flag(memory, result.is_negative());
    }

    fn update_carry_flag(&mut self, memory: &mut Memory, carry_bit: bool) {
        self.status = self.status.set_n_bit(CARRY_FLAG_INDEX, carry_bit);
        self.update_flags_in_memory(memory);
    }

    fn update_overflow_flag(&mut self, memory: &mut Memory, overflow_bit: bool) {
        self.status = self.status.set_n_bit(OVERFLOW_FLAG_INDEX, overflow_bit);
        self.update_flags_in_memory(memory);
    }

    fn update_zero_flag(&mut self, memory: &mut Memory, zero_bit: bool) {
        self.status = self.status.set_n_bit(ZERO_FLAG_INDEX, zero_bit);
        self.update_flags_in_memory(memory);
    }

    fn update_negative_flag(&mut self, memory: &mut Memory, negative_bit: bool) {
        self.status = self.status.set_n_bit(NEGATIVE_FLAG_INDEX, negative_bit);
        self.update_flags_in_memory(memory);
    }

    fn carry_flag(&self) -> bool {
        self.status.get_n_bit(CARRY_FLAG_INDEX)
    }

    fn overflow_flag(&self) -> bool {
        self.status.get_n_bit(OVERFLOW_FLAG_INDEX)
    }

    fn zero_flag(&self) -> bool {
        self.status.get_n_bit(ZERO_FLAG_INDEX)
    }

    fn negative_flag(&self) -> bool {
        self.status.get_n_bit(NEGATIVE_FLAG_INDEX)
    }

    fn update_flags_in_memory(&self, memory: &mut Memory) {
        memory.write_word(FLAGS_IN_MEMORY, self.status);
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
