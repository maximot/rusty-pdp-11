use crate::{ mem::{self, Memory}, utils::{has_carry, LongWord, Number, Word }};

use super::{ adr_operand, assert_even_reg, branch_offset, commands::{ dst_operand, src_operand }, long_word, reg_operand, word_has_carry, Byte, CPU, PROGRAM_COUNTER_INDEX };

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

// One-and-a-half-operand
impl CPU {
    pub fn do_mul(&mut self, memory: &mut Memory, command: Word) {
        let dst = reg_operand(command);
        let dst_hi = dst | 0x01;

        let src = adr_operand(command);

        let dst_value = self.get_word_from_reg(dst);
        let src_value = self.get_word_by_operand(memory, src);

        let result = (dst_value as LongWord) * (src_value as LongWord);

        self.set_word_reg(dst_hi, result.high());
        self.set_word_reg(dst, result.low());

        self.update_status_flags(memory, result, has_carry(result));
        self.update_overflow_flag(memory, false);
    }

    pub fn do_div(&mut self, memory: &mut Memory, command: Word) {
        let dst = reg_operand(command);

        assert_even_reg(&dst);

        let dst_hi = dst | 0x01u8;

        let src = adr_operand(command);

        let dst_lo_value = self.get_word_from_reg(dst);
        let dst_hi_value = self.get_word_from_reg(dst_hi);

        let dst_value = long_word(dst_lo_value, dst_hi_value);
        let src_value = self.get_word_by_operand(memory, src) as LongWord;

        if src_value.is_zero() {
            self.update_carry_flag(memory, true);
            self.update_overflow_flag(memory, true);
            return;
        }

        let quotient = dst_value / src_value;
        let reminder = dst_value % src_value;

        self.set_word_reg(dst_hi, reminder.low());
        self.set_word_reg(dst, quotient.low());

        self.update_status_flags(memory, quotient.low(), has_carry(quotient));
        self.update_carry_flag(memory, false);
    }

    pub fn do_ash(&mut self, memory: &mut Memory, command: Word) {
        let dst = reg_operand(command);
        
        let src_value = self.get_word_by_operand(memory, adr_operand(command));
        let left_shift = (src_value & 0x0020u16) == 0x0000u16;
        let shift = src_value & 0x001Fu16;

        let dst_value = self.get_word_from_reg(dst);

        if shift == 0 {
            self.update_status_flags(memory, dst_value, false);
            return;
        }

        let partial_shift = shift - 1u16;

        let (intermediate_result, result) = if left_shift {
            let partially_shifted = dst_value << partial_shift;
            let shifted = partially_shifted << 1;

            (partially_shifted, shifted)
        } else {
            let partially_shifted = dst_value >> partial_shift;
            let shifted = partially_shifted >> 1;

            (partially_shifted, shifted)
        };

        self.set_word_reg(dst, result);

        let overflow = dst_value.is_negative() != result.is_negative();

        let carry = if left_shift {
            intermediate_result.is_negative()
        } else {
            (intermediate_result & 0x0001u16) > 0
        };

        self.update_status_flags(memory, result, carry);
        self.update_overflow_flag(memory, overflow);
    }

    pub fn do_ashc(&mut self, memory: &mut Memory, command: Word) {
        let dst = reg_operand(command);

        assert_even_reg(&dst);

        let dst_hi = dst | 0x01u8;
        
        let src_value = self.get_word_by_operand(memory, adr_operand(command));
        let left_shift = (src_value & 0x0020u16) == 0x0000u16;
        let shift = src_value & 0x001Fu16;

        let dst_lo_value = self.get_word_from_reg(dst);
        let dst_hi_value = self.get_word_from_reg(dst_hi);

        let dst_value = long_word(dst_lo_value, dst_hi_value);

        if shift == 0 {
            self.update_status_flags(memory, dst_value, false);
            return;
        }

        let partial_shift = shift - 1u16;

        let (intermediate_result, result) = if left_shift {
            let partially_shifted = dst_value << partial_shift;
            let shifted = partially_shifted << 1;

            (partially_shifted, shifted)
        } else {
            let partially_shifted = dst_value >> partial_shift;
            let shifted = partially_shifted >> 1;

            (partially_shifted, shifted)
        };

        self.set_word_reg(dst, result.low());
        self.set_word_reg(dst_hi, result.high());

        let overflow = dst_value.is_negative() != result.is_negative();

        let carry = if left_shift {
            intermediate_result.is_negative()
        } else {
            (intermediate_result & 0x00000001u32) > 0
        };

        self.update_status_flags(memory, result, carry);
        self.update_overflow_flag(memory, overflow);
    }

    pub fn do_xor(&mut self, memory: &mut Memory, command: Word) {
        let dst = adr_operand(command);

        let dst_value = self.get_word_by_operand(memory, dst);
        let src_value = self.get_word_from_reg(reg_operand(command));

        let result = dst_value ^ src_value;

        self.put_word_by_operand(memory, dst, result);
        self.update_status_flags_bitwise(memory, result);
    }
}

// Two-operand
impl CPU {
    pub fn do_mov(&mut self, memory: &mut Memory, command: Word) {
        let word_to_move = self.get_word_by_operand(memory, src_operand(command));

        self.put_word_by_operand(memory, dst_operand(command), word_to_move);

        self.update_status_flags_bitwise(memory, word_to_move);
    }

    pub fn do_movb(&mut self, memory: &mut Memory, command: Word) {
        let byte_to_move = self.get_byte_by_operand(memory, src_operand(command));

        self.put_byte_by_operand(memory, dst_operand(command), byte_to_move);

        self.update_status_flags_bitwise(memory, byte_to_move);
    }

    pub fn do_add(&mut self, memory: &mut Memory, command: Word) {
        let dst = dst_operand(command);

        let dst_value = self.get_word_by_operand(memory, dst);
        let src_value = self.get_word_by_operand(memory, src_operand(command));

        let sum = dst_value as LongWord + src_value as LongWord;

        let result = sum as Word;

        self.put_word_by_operand(memory, dst, result);

        self.update_status_flags(memory, result, has_carry(sum));
    }

    pub fn do_sub(&mut self, memory: &mut Memory, command: Word) {
        let dst = dst_operand(command);

        let dst_value = self.get_word_by_operand(memory, dst);
        let src_value = self.get_word_by_operand(memory, src_operand(command));

        let sub = dst_value as LongWord - src_value as LongWord;

        let result = sub as Word;

        self.put_word_by_operand(memory, dst, result);

        self.update_status_flags(memory, result, !has_carry(sub));
    }

    pub fn do_cmp(&mut self, memory: &mut Memory, command: Word) {
        let dst_value = self.get_word_by_operand(memory, dst_operand(command));
        let src_value = self.get_word_by_operand(memory, src_operand(command));

        let sub = src_value as LongWord - dst_value as LongWord;

        let result = sub as Word;

        self.update_status_flags(memory, result, !has_carry(sub));
    }

    pub fn do_cmpb(&mut self, memory: &mut Memory, command: Word) {
        let dst_value = self.get_byte_by_operand(memory, dst_operand(command));
        let src_value = self.get_byte_by_operand(memory, src_operand(command));

        let sub = src_value as Word - dst_value as Word;

        let result = sub as Byte;

        self.update_status_flags(memory, result, !word_has_carry(sub));
    }

    pub fn do_bis(&mut self, memory: &mut Memory, command: Word) {
        let dst = dst_operand(command);

        let dst_value = self.get_word_by_operand(memory, dst);
        let src_value = self.get_word_by_operand(memory, src_operand(command));

        let result = dst_value | src_value;

        self.put_word_by_operand(memory, dst, result);

        self.update_status_flags_bitwise(memory, result);
    }

    pub fn do_bisb(&mut self, memory: &mut Memory, command: Word) {
        let dst = dst_operand(command);

        let dst_value = self.get_byte_by_operand(memory, dst);
        let src_value = self.get_byte_by_operand(memory, src_operand(command));

        let result = dst_value | src_value;

        self.put_byte_by_operand(memory, dst, result);

        self.update_status_flags_bitwise(memory, result);
    }

    pub fn do_bic(&mut self, memory: &mut Memory, command: Word) {
        let dst = dst_operand(command);

        let dst_value = self.get_word_by_operand(memory, dst);
        let src_value = self.get_word_by_operand(memory, src_operand(command));

        let result = dst_value & src_value.one_complement();

        self.put_word_by_operand(memory, dst, result);

        self.update_status_flags_bitwise(memory, result);
    }

    pub fn do_bicb(&mut self, memory: &mut Memory, command: Word) {
        let dst = dst_operand(command);

        let dst_value = self.get_byte_by_operand(memory, dst);
        let src_value = self.get_byte_by_operand(memory, src_operand(command));

        let result = dst_value & src_value.one_complement();

        self.put_byte_by_operand(memory, dst, result);

        self.update_status_flags_bitwise(memory, result);
    }

    pub fn do_bit(&mut self, memory: &mut Memory, command: Word) {
        let dst_value = self.get_word_by_operand(memory, dst_operand(command));
        let src_value = self.get_word_by_operand(memory, src_operand(command));

        let result = src_value & dst_value;

        self.update_status_flags_bitwise(memory, result);
    }

    pub fn do_bitb(&mut self, memory: &mut Memory, command: Word) {
        let dst_value = self.get_byte_by_operand(memory, dst_operand(command));
        let src_value = self.get_byte_by_operand(memory, src_operand(command));

        let result = src_value & dst_value;

        self.update_status_flags_bitwise(memory, result);
    }
}

// Branch commands
impl CPU {
    pub fn do_br(&mut self, _memory: &mut Memory, command: Word) {
        let offset = branch_offset(command);

        let pc = self.get_word_from_reg(PROGRAM_COUNTER_INDEX);

        let result = pc + offset;

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
