use crate::mem::Memory;

use super::{ Address, Byte, Number, Word, CPU, PROGRAM_COUNTER_INDEX, WORD_SIZE_BYTES };

// Addressing
impl CPU {
    pub (in super) fn get_register_deferred_address(&mut self, _memory: &Memory, reg_index: Byte, _increment_by: Byte) -> Address {
        self.get_word_from_reg(reg_index).into()
    }

    pub (in super) fn get_autoincrement_address(&mut self, _memory: &Memory, reg_index: Byte, increment_by: Byte) -> Address {
        self.get_and_increment(reg_index, increment_by.into()).into()
    }

    pub (in super) fn get_autoincrement_deferred_address(&mut self, memory: &Memory, reg_index: Byte, _increment_by: Byte) -> Address {
        memory.read_word(self.get_and_increment(reg_index, WORD_SIZE_BYTES).into()).into()
    }

    pub (in super) fn get_autodecrement_address(&mut self, _memory: &Memory, reg_index: Byte, increment_by: Byte) -> Address {
        self.decrement_and_get(reg_index, increment_by.into()).into()
    }

    pub (in super) fn get_autodecrement_deferred_address(&mut self, memory: &Memory, reg_index: Byte, _increment_by: Byte) -> Address {
        memory.read_word(self.decrement_and_get(reg_index, WORD_SIZE_BYTES).into()).into()
    }

    pub (in super) fn get_index_address(&mut self, memory: &Memory, reg_index: Byte, _increment_by: Byte) -> Address {
        let n = memory.read_word(self.get_and_increment(PROGRAM_COUNTER_INDEX, WORD_SIZE_BYTES).into());

        (n + self.get_word_from_reg(reg_index)).into()
    }

    pub (in super) fn get_index_deferred_address(&mut self, memory: &Memory, reg_index: Byte, increment_by: Byte) -> Address {
        memory.read_word(self.get_index_address(memory, reg_index, increment_by)).into()
    }

    pub (in super) fn get_immediate_address(&mut self, _memory: &Memory, reg_index: Byte, _increment_by: Byte) -> Address {
        self.get_and_increment(reg_index, WORD_SIZE_BYTES).into()
    }

    pub (in super) fn get_addressing_func(addressing: AddressingMode) -> impl Fn(&mut CPU, &Memory, Byte, Byte) -> Address {
        match addressing {
            AddressingMode::Register => panic!("Can't get addressing func for AddressingMode::Register"),
            AddressingMode::RegisterDeferred => CPU::get_register_deferred_address,
            AddressingMode::Autoicrement => CPU::get_autoincrement_address,
            AddressingMode::AutoicrementDeferred => CPU::get_autoincrement_deferred_address,
            AddressingMode::Autodecrement => CPU::get_autodecrement_address,
            AddressingMode::AutodecrementDeferred => CPU::get_autodecrement_deferred_address,
            AddressingMode::Index => CPU::get_index_address,
            AddressingMode::IndexDeferred => CPU::get_index_deferred_address,
            AddressingMode::Immediate => CPU::get_immediate_address,
            AddressingMode::Absolute => CPU::get_autoincrement_deferred_address,
            AddressingMode::Relative => CPU::get_index_address, 
            AddressingMode::RelativeDeferred => CPU::get_index_deferred_address,
        }
    }
}

// Put operand
impl CPU {
    pub (in super) fn put_operand_value_with_addressing<T, N: Number<T>>(
        &mut self, 
        memory: &mut Memory, 
        reg_index: Byte, 
        addressing: AddressingMode, 
        data: N, 
        write_memory: impl Fn(&mut Memory, Address, N) -> usize, 
        set_register: impl Fn(&mut CPU, Byte, N),
    ) {
        match addressing {
            AddressingMode::Register => self.put_addressing_register(reg_index, data, set_register),
            _ => self.put_operand_value(memory, write_memory, Self::get_addressing_func(addressing), reg_index, data),
        }
    }

    fn put_operand_value<T, N: Number<T>>(
        &mut self, 
        memory: &mut Memory, 
        write_memory: impl Fn(&mut Memory, Address, N) -> usize, 
        get_address: impl Fn(&mut CPU, &Memory, Byte, Byte) -> Address,
        reg_index: Byte,
        value: N
    ) {
        write_memory(memory, get_address(self, memory, reg_index, N::size_bytes()), value);
    }

    fn put_addressing_register<T, N: Number<T>>(&mut self, reg_index: Byte, data: N, set_register: impl Fn(&mut CPU, Byte, N)) {
        set_register(self, reg_index, data);
    }
}

// Get operand
impl CPU {
    pub (in super) fn get_operand_value_with_addressing<T, N: Number<T>>(
        &mut self, 
        memory: &Memory, 
        reg_index: Byte, 
        addressing: AddressingMode, 
        read_memory: impl Fn(&Memory, Address) -> N, 
        get_register: impl Fn(&mut CPU, Byte) -> N
    ) -> N {
        match addressing {
            AddressingMode::Register => self.get_addressing_register(reg_index, get_register),
            _ => self.get_operand_value(memory, read_memory, Self::get_addressing_func(addressing), reg_index),
        }
    }

    fn get_operand_value<T, N: Number<T>>(
        &mut self, 
        memory: &Memory, 
        read_memory: impl Fn(&Memory, Address) -> N, 
        get_address: impl Fn(&mut CPU, &Memory, Byte, Byte) -> Address,
        reg_index: Byte
    ) -> N {
        read_memory(memory, get_address(self, memory, reg_index, N::size_bytes()))
    }

    fn get_addressing_register<T, N: Number<T>>(&mut self, reg_index: Byte, get_register: impl Fn(&mut CPU, Byte) -> N) -> N {
        get_register(self, reg_index)
    }
}

// Get operand address
impl CPU {
    pub (in super) fn get_operand_address_with_addressing(
        &mut self,
        memory: &Memory, 
        reg_index: Byte, 
        addressing: AddressingMode, 
    ) -> Address {
        Self::get_addressing_func(addressing)(self, memory, reg_index, Word::size_bytes())
    }
}

pub (in super) fn adressing_from_operand(operand: Byte) -> AddressingMode {
    let mode = operand >> 3 & 0x07;

    if register_from_operand(operand) == PROGRAM_COUNTER_INDEX {
        return (mode << 3 | 0x07).into();
    }

    mode.into()
}

pub (in super) fn register_from_operand(operand: Byte) -> Byte {
    operand & 0x07
}

#[repr(u8)]
pub (in super) enum AddressingMode {
    Register = 0x0,
    RegisterDeferred = 0x1,
    Autoicrement = 0x2,
    AutoicrementDeferred = 0x3,
    Autodecrement = 0x4,
    AutodecrementDeferred = 0x5,
    Index = 0x6,
    IndexDeferred = 0x7,

    // Only for PC register
    Immediate = 0x17,
    Absolute = 0x1F,
    Relative = 0x37,
    RelativeDeferred = 0x3F,
}

impl From<Byte> for AddressingMode {
    fn from(byte: Byte) -> AddressingMode {
        match byte {
            0x00 => AddressingMode::Register,
            0x01 => AddressingMode::RegisterDeferred,
            0x02 => AddressingMode::Autoicrement,
            0x03 => AddressingMode::AutoicrementDeferred,
            0x04 => AddressingMode::Autodecrement,
            0x05 => AddressingMode::AutodecrementDeferred,
            0x06 => AddressingMode::Index,
            0x07 => AddressingMode::IndexDeferred,
            0x17 => AddressingMode::Immediate,
            0x1F => AddressingMode::Absolute,
            0x37 => AddressingMode::Relative,
            0x3F => AddressingMode::RelativeDeferred,
            _ => panic!("Unknown AddressingMode!")
        }
    }
}