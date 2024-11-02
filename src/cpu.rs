use std::rc::Rc;

use addressing::{adressing_from_operand, register_from_operand, AddressingMode};
use commands::*;

use crate::{mem::Memory, utils::*};

pub mod addressing;
pub mod interpreter;
pub mod debug;
pub mod commands;

pub const FIRST_COMMAND: Address = 0x200;

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
}

// Constructors
impl CPU {
    pub fn new(commands: Rc<Commands>) -> Self {
        CPU {
            status: 0x0000,
            registers: [0; REG_COUNT],
            commands: commands,
            running: false,
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
            let (address, command_word) = self.next_command(memory);
    
            trace!("tick");
            trace!("address 0x{address:04X}");
            trace!("instruction 0x{command_word:04X}");

            let command = self.command(command_word);
            let command_opcode = command.0;
            let command_name = command.1;

            trace!("command 0x{command_opcode:04X} ({command_name})");

            let command_interpreter = command.2;

            command_interpreter(self, memory, command_word);

            //self.trace_registers();
        }
    }

    fn next_command(&mut self, memory: &mut Memory) -> (Address, Word) {
        let address: Address = self.get_and_increment(PROGRAM_COUNTER_INDEX, WORD_SIZE_BYTES).into();

        let command: Word = memory.read_word(address);

        (address, command)
    }

    fn command(&self, command_word: Word) -> &Command {
        if let Some(command) = self.commands.o_0_commands.get(&(command_word & O_0_MASK)) {
            return command;
        }

        if let Some(command) = self.commands.o_1_commands.get(&(command_word & O_1_MASK)) {
            return command;
        }

        if let Some(command) = self.commands.o_1_5_commands.get(&(command_word & O_1_5_MASK)) {
            return command;
        }

        if let Some(command) = self.commands.o_2_commands.get(&(command_word & O_2_MASK)) {
            return command;
        }

        if let Some(command) = self.commands.b_commands.get(&(command_word & B_MASK)) {
            return command;
        }

        &UNKNOWN_COMMAND
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

    fn put_operand_value_with_addressing<N: Number>(
        &mut self, 
        memory: &mut Memory, 
        reg_index: Byte, 
        addressing: AddressingMode, 
        data: N, 
        write_memory: impl Fn(&mut Memory, Address, N) -> usize, 
        set_register: impl Fn(&mut CPU, Byte, N),
    ) {
        match addressing {
            AddressingMode::Register => self.set_addressing_register(reg_index, data, set_register),
            AddressingMode::RegisterDeferred => self.set_operand_value(memory, write_memory, CPU::get_register_deferred_address, reg_index, data),
            AddressingMode::Autoicrement => self.set_operand_value(memory, write_memory, CPU::get_autoincrement_address, reg_index, data),
            AddressingMode::AutoicrementDeferred => self.set_operand_value(memory, write_memory, CPU::get_autoincrement_deferred_address, reg_index, data),
            AddressingMode::Autodecrement => self.set_operand_value(memory, write_memory, CPU::get_autodecrement_address, reg_index, data),
            AddressingMode::AutodecrementDeferred => self.set_operand_value(memory, write_memory, CPU::get_autodecrement_deferred_address, reg_index, data),
            AddressingMode::Index => self.set_operand_value(memory, write_memory, CPU::get_index_address, reg_index, data),
            AddressingMode::IndexDeferred => self.set_operand_value(memory, write_memory, CPU::get_index_deferred_address, reg_index, data),
            AddressingMode::Immediate => self.set_operand_value(memory, write_memory, CPU::get_immediate_address, reg_index, data),
            AddressingMode::Absolute => self.set_operand_value(memory, write_memory, CPU::get_autoincrement_deferred_address, reg_index, data),
            AddressingMode::Relative => self.set_operand_value(memory, write_memory, CPU::get_index_address, reg_index, data),
            AddressingMode::RelativeDeferred => self.set_operand_value(memory, write_memory, CPU::get_index_deferred_address, reg_index, data),
        }
    }

    fn set_operand_value<N: Number>(
        &mut self, 
        memory: &mut Memory, 
        write_memory: impl Fn(&mut Memory, Address, N) -> usize, 
        get_address: impl Fn(&mut CPU, &Memory, Byte, Byte) -> Address,
        reg_index: Byte,
        value: N
    ) {
        write_memory(memory, get_address(self, memory, reg_index, N::size_bytes()), value);
    }

    fn set_addressing_register<N: Number>(&mut self, reg_index: Byte, data: N, set_register: impl Fn(&mut CPU, Byte, N)) {
        set_register(self, reg_index, data);
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

    fn get_operand_value_with_addressing<N: Number>(
        &mut self, 
        memory: &Memory, 
        reg_index: Byte, 
        addressing: AddressingMode, 
        read_memory: impl Fn(&Memory, Address) -> N, 
        get_register: impl Fn(&mut CPU, Byte) -> N
    ) -> N {
        match addressing {
            AddressingMode::Register => self.get_addressing_register(reg_index, get_register),
            AddressingMode::RegisterDeferred => self.get_operand_value(memory, read_memory, CPU::get_register_deferred_address, reg_index),
            AddressingMode::Autoicrement => self.get_operand_value(memory, read_memory, CPU::get_autoincrement_address, reg_index),
            AddressingMode::AutoicrementDeferred => self.get_operand_value(memory, read_memory, CPU::get_autoincrement_deferred_address, reg_index),
            AddressingMode::Autodecrement => self.get_operand_value(memory, read_memory, CPU::get_autodecrement_address, reg_index),
            AddressingMode::AutodecrementDeferred => self.get_operand_value(memory, read_memory, CPU::get_autodecrement_deferred_address, reg_index),
            AddressingMode::Index => self.get_operand_value(memory, read_memory, CPU::get_index_address, reg_index),
            AddressingMode::IndexDeferred => self.get_operand_value(memory, read_memory, CPU::get_index_deferred_address, reg_index),
            AddressingMode::Immediate => self.get_operand_value(memory, read_memory, CPU::get_immediate_address, reg_index),
            AddressingMode::Absolute => self.get_operand_value(memory, read_memory, CPU::get_autoincrement_deferred_address, reg_index),
            AddressingMode::Relative => self.get_operand_value(memory, read_memory, CPU::get_index_address, reg_index),
            AddressingMode::RelativeDeferred => self.get_operand_value(memory, read_memory, CPU::get_index_deferred_address, reg_index),
        }
    }

    fn get_operand_value<N: Number>(
        &mut self, 
        memory: &Memory, 
        read_memory: impl Fn(&Memory, Address) -> N, 
        get_address: impl Fn(&mut CPU, &Memory, Byte, Byte) -> Address,
        reg_index: Byte
    ) -> N {
        read_memory(memory, get_address(self, memory, reg_index, N::size_bytes()))
    }

    fn get_addressing_register<N: Number>(&mut self, reg_index: Byte, get_register: impl Fn(&mut CPU, Byte) -> N) -> N {
        get_register(self, reg_index)
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
    fn update_status_flags_bitwise<N: Number>(&mut self, result: N) {
        self.update_overflow_flag(false);
        self.update_zero_flag(result.is_zero());
        self.update_negative_flag(result.is_negative());
    }

    fn update_status_flags<N: Number>(&mut self, result: N, carry_bit: bool) {
        self.update_carry_flag(carry_bit);
        self.update_overflow_flag(carry_bit);
        self.update_zero_flag(result.is_zero());
        self.update_negative_flag(result.is_negative());
    }

    fn update_carry_flag(&mut self, carry_bit: bool) {
        self.status = self.status.set_n_bit(CARRY_FLAG_INDEX, carry_bit)
    }

    fn update_overflow_flag(&mut self, overflow_bit: bool) {
        self.status = self.status.set_n_bit(OVERFLOW_FLAG_INDEX, overflow_bit)
    }

    fn update_zero_flag(&mut self, zero_bit: bool) {
        self.status = self.status.set_n_bit(ZERO_FLAG_INDEX, zero_bit)
    }

    fn update_negative_flag(&mut self, negative_bit: bool) {
        self.status = self.status.set_n_bit(NEGATIVE_FLAG_INDEX, negative_bit)
    }
}

// Asserts
fn assert_not_pc(reg_index: &Byte) {
    assert!(*reg_index != PROGRAM_COUNTER_INDEX);
}

fn assert_pc(reg_index: &Byte) {
    assert!(*reg_index == PROGRAM_COUNTER_INDEX);
}
