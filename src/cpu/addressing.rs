use crate::mem::Memory;

use super::{ Address, Byte, CPU, PROGRAM_COUNTER_INDEX, WORD_SIZE_BYTES};

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