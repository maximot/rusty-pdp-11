use crate::{ mem::Memory, utils::{has_carry, LongWord, Number, Word }};

use super::{ branch_offset, commands::{ dst_operand, src_operand }, word_has_carry, Byte, CPU, PROGRAM_COUNTER_INDEX };

// Zero-oparand
impl CPU {
    pub fn do_nop(&mut self, _memory: &mut Memory, _command: Word) { /* NO-OP */ }

    pub fn do_halt(&mut self, _memory: &mut Memory, _command: Word) {
        self.running = false;
    }

    pub fn do_panic(&mut self, _memory: &mut Memory, _command: Word) {
        panic!("CPU panic!")
    }
}

// Two-operand
impl CPU {
    pub fn do_mov(&mut self, memory: &mut Memory, command: Word) {
        let word_to_move = self.get_word_by_operand(memory, src_operand(command));

        self.put_word_by_operand(memory, dst_operand(command), word_to_move);

        self.update_status_flags_bitwise(word_to_move);
    }

    pub fn do_movb(&mut self, memory: &mut Memory, command: Word) {
        let byte_to_move = self.get_byte_by_operand(memory, src_operand(command));

        self.put_byte_by_operand(memory, dst_operand(command), byte_to_move);

        self.update_status_flags_bitwise(byte_to_move);
    }

    pub fn do_add(&mut self, memory: &mut Memory, command: Word) {
        let dst = dst_operand(command);

        let dst_value = self.get_word_by_operand(memory, dst);
        let src_value = self.get_word_by_operand(memory, src_operand(command));

        let sum = dst_value as LongWord + src_value as LongWord;

        let result = sum as Word;

        self.put_word_by_operand(memory, dst, result);

        self.update_status_flags(result, has_carry(sum));
    }

    pub fn do_sub(&mut self, memory: &mut Memory, command: Word) {
        let dst = dst_operand(command);

        let dst_value = self.get_word_by_operand(memory, dst);
        let src_value = self.get_word_by_operand(memory, src_operand(command));

        let sub = dst_value as LongWord - src_value as LongWord;

        let result = sub as Word;

        self.put_word_by_operand(memory, dst, result);

        self.update_status_flags(result, !has_carry(sub));
    }

    pub fn do_cmp(&mut self, memory: &mut Memory, command: Word) {
        let dst_value = self.get_word_by_operand(memory, dst_operand(command));
        let src_value = self.get_word_by_operand(memory, src_operand(command));

        let sub = src_value as LongWord - dst_value as LongWord;

        let result = sub as Word;

        self.update_status_flags(result, !has_carry(sub));
    }

    pub fn do_cmpb(&mut self, memory: &mut Memory, command: Word) {
        let dst_value = self.get_byte_by_operand(memory, dst_operand(command));
        let src_value = self.get_byte_by_operand(memory, src_operand(command));

        let sub = src_value as Word - dst_value as Word;

        let result = sub as Byte;

        self.update_status_flags(result, !word_has_carry(sub));
    }

    pub fn do_bis(&mut self, memory: &mut Memory, command: Word) {
        let dst = dst_operand(command);

        let dst_value = self.get_word_by_operand(memory, dst);
        let src_value = self.get_word_by_operand(memory, src_operand(command));

        let result = dst_value | src_value;

        self.put_word_by_operand(memory, dst, result);

        self.update_status_flags_bitwise(result);
    }

    pub fn do_bisb(&mut self, memory: &mut Memory, command: Word) {
        let dst = dst_operand(command);

        let dst_value = self.get_byte_by_operand(memory, dst);
        let src_value = self.get_byte_by_operand(memory, src_operand(command));

        let result = dst_value | src_value;

        self.put_byte_by_operand(memory, dst, result);

        self.update_status_flags_bitwise(result);
    }

    pub fn do_bic(&mut self, memory: &mut Memory, command: Word) {
        let dst = dst_operand(command);

        let dst_value = self.get_word_by_operand(memory, dst);
        let src_value = self.get_word_by_operand(memory, src_operand(command));

        let result = dst_value & src_value.one_complement();

        self.put_word_by_operand(memory, dst, result);

        self.update_status_flags_bitwise(result);
    }

    pub fn do_bicb(&mut self, memory: &mut Memory, command: Word) {
        let dst = dst_operand(command);

        let dst_value = self.get_byte_by_operand(memory, dst);
        let src_value = self.get_byte_by_operand(memory, src_operand(command));

        let result = dst_value & src_value.one_complement();

        self.put_byte_by_operand(memory, dst, result);

        self.update_status_flags_bitwise(result);
    }

    pub fn do_bit(&mut self, memory: &mut Memory, command: Word) {
        let dst_value = self.get_word_by_operand(memory, dst_operand(command));
        let src_value = self.get_word_by_operand(memory, src_operand(command));

        let result = src_value & dst_value;

        self.update_status_flags_bitwise(result);
    }

    pub fn do_bitb(&mut self, memory: &mut Memory, command: Word) {
        let dst_value = self.get_byte_by_operand(memory, dst_operand(command));
        let src_value = self.get_byte_by_operand(memory, src_operand(command));

        let result = src_value & dst_value;

        self.update_status_flags_bitwise(result);
    }
}

// Branch commands
impl CPU {
    pub fn do_br(&mut self, _memory: &mut Memory, command: Word) {
        let offset = branch_offset(command) as Word;

        let pc = self.get_word_from_reg(PROGRAM_COUNTER_INDEX);

        let result = pc + (offset << 1);

        self.set_word_reg(PROGRAM_COUNTER_INDEX, result);
    }

    pub fn do_bne(&mut self, memory: &mut Memory, command: Word) {
        if !self.zero_flag() {
            self.do_br(memory, command);
        }
    }

    pub fn do_beq(&mut self, memory: &mut Memory, command: Word) {
        if self.zero_flag() {
            self.do_br(memory, command);
        }
    }

    pub fn do_bpl(&mut self, memory: &mut Memory, command: Word) {
        if !self.negative_flag() {
            self.do_br(memory, command);
        }
    }

    pub fn do_bmi(&mut self, memory: &mut Memory, command: Word) {
        if self.negative_flag() {
            self.do_br(memory, command);
        }
    }

    pub fn do_bvc(&mut self, memory: &mut Memory, command: Word) {
        if !self.overflow_flag() {
            self.do_br(memory, command);
        }
    }

    pub fn do_bvs(&mut self, memory: &mut Memory, command: Word) {
        if self.overflow_flag() {
            self.do_br(memory, command);
        }
    }

    pub fn do_bcc(&mut self, memory: &mut Memory, command: Word) {
        if !self.carry_flag() {
            self.do_br(memory, command);
        }
    }

    pub fn do_bcs(&mut self, memory: &mut Memory, command: Word) {
        if !self.carry_flag() {
            self.do_br(memory, command);
        }
    }

    pub fn do_bge(&mut self, memory: &mut Memory, command: Word) {
        let condition = !(self.negative_flag() ^ self.overflow_flag());

        if condition {
            self.do_br(memory, command);
        }
    }

    pub fn do_blt(&mut self, memory: &mut Memory, command: Word) {
        let condition = self.negative_flag() ^ self.overflow_flag();

        if condition {
            self.do_br(memory, command);
        }
    }

    pub fn do_bgt(&mut self, memory: &mut Memory, command: Word) {
        let condition = !(self.zero_flag() || (self.negative_flag() ^ self.overflow_flag()));

        if condition {
            self.do_br(memory, command);
        }
    }

    pub fn do_ble(&mut self, memory: &mut Memory, command: Word) {
        let condition = self.zero_flag() || (self.negative_flag() ^ self.overflow_flag());

        if condition {
            self.do_br(memory, command);
        }
    }

    pub fn do_bhi(&mut self, memory: &mut Memory, command: Word) {
        let condition = !(self.zero_flag() || self.carry_flag());

        if condition {
            self.do_br(memory, command);
        }
    }

    pub fn do_blos(&mut self, memory: &mut Memory, command: Word) {
        let condition = self.zero_flag() || self.carry_flag();

        if condition {
            self.do_br(memory, command);
        }
    }
}
