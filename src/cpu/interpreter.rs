use crate::{ mem::{self, Memory}, utils::{has_carry, LongWord, Number, Word }};

use super::{ adr_operand, assert_even_reg, branch_offset, commands::{ dst_operand, src_operand }, has_signed_overflow, long_word, low_reg_operand, make_word, reg_operand, word_has_carry, Address, Byte, CARRY_FLAG_INDEX, CPU, MARK_POINTER_INDEX, NEGATIVE_FLAG_INDEX, OVERFLOW_FLAG_INDEX, PROGRAM_COUNTER_INDEX, STACK_POINTER_INDEX, ZERO_FLAG_INDEX };

// Zero-oparand
impl CPU {
    pub fn do_nop(&mut self, _memory: &mut Memory, _command: Word) { /* NO-OP */ }

    pub fn do_halt(&mut self, _memory: &mut Memory, _command: Word) {
        *self.running.lock().unwrap() = false;
    }

    pub fn do_wait(&mut self, _memory: &mut Memory, _command: Word) {
        self.waiting = true;
    }

    pub fn do_panic(&mut self, _memory: &mut Memory, _command: Word) {
        panic!("CPU panic!")
    }

    pub fn do_rti(&mut self, memory: &mut Memory, _command: Word) {
        let new_pc = self.pop_stack(memory);

        self.set_word_reg(PROGRAM_COUNTER_INDEX, new_pc);

        let new_psw = self.pop_stack(memory);

        self.set_status_word(new_psw);
    }

    pub fn do_rtt(&mut self, memory: &mut Memory, command: Word) {
        self.do_rti(memory, command);
    }

    pub fn do_bpt(&mut self, memory: &mut Memory, _command: Word) {
        self.perform_trap(memory, 0x000C); // Trap from 14 (oct)
    }

    pub fn do_iot(&mut self, memory: &mut Memory, _command: Word) {
        self.perform_trap(memory, 0x0010); // Trap from 20 (oct)
    }
}

// Set priority & some Control Flow & Float commands
impl CPU {
    pub fn do_spl(&mut self, _memory: &mut Memory, command: Word) {
        self.update_priority(command.low());
    }

    pub fn do_rts(&mut self, memory: &mut Memory, command: Word) {
        let reg = low_reg_operand(command);

        let reg_value = self.get_word_from_reg(reg);

        self.set_word_reg(PROGRAM_COUNTER_INDEX, reg_value);

        let stack_value = self.pop_stack(memory);

        self.set_word_reg(reg, stack_value);
    }

    pub fn do_fadd(&mut self, memory: &mut Memory, command: Word) {
        let reg = low_reg_operand(command);

        let src_float = self.get_float_from_reg(memory, reg);

        self.increment_reg(reg, 2 * Word::size_bytes().word());

        let dst_float = self.get_float_from_reg(memory, reg);

        let result = src_float + dst_float;

        self.set_float_by_reg(memory, reg, result);
    }

    pub fn do_fsub(&mut self, memory: &mut Memory, command: Word) {
        let reg = low_reg_operand(command);

        let src_float = self.get_float_from_reg(memory, reg);

        self.increment_reg(reg, 2 * Word::size_bytes().word());

        let dst_float = self.get_float_from_reg(memory, reg);

        let result = dst_float - src_float;

        self.set_float_by_reg(memory, reg, result);
    }

    pub fn do_fmul(&mut self, memory: &mut Memory, command: Word) {
        let reg = low_reg_operand(command);

        let src_float = self.get_float_from_reg(memory, reg);

        self.increment_reg(reg, 2 * Word::size_bytes().word());

        let dst_float = self.get_float_from_reg(memory, reg);

        let result = dst_float * src_float;

        self.set_float_by_reg(memory, reg, result);
    }

    pub fn do_fdiv(&mut self, memory: &mut Memory, command: Word) {
        let reg = low_reg_operand(command);

        let src_float = self.get_float_from_reg(memory, reg);

        self.increment_reg(reg, 2 * Word::size_bytes().word());

        let dst_float = self.get_float_from_reg(memory, reg);

        let result = dst_float / src_float;

        self.set_float_by_reg(memory, reg, result);
    }
}

// Set/clear flags
impl CPU {
    pub fn do_se(&mut self, _memory: &mut Memory, command: Word) {
        if command.get_n_bit(CARRY_FLAG_INDEX) {
            self.update_carry_flag(true);
        }

        if command.get_n_bit(OVERFLOW_FLAG_INDEX) {
            self.update_overflow_flag(true);
        }

        if command.get_n_bit(ZERO_FLAG_INDEX) {
            self.update_zero_flag(true);
        }

        if command.get_n_bit(NEGATIVE_FLAG_INDEX) {
            self.update_negative_flag(true);
        }
    }

    pub fn do_cl(&mut self, _memory: &mut Memory, command: Word) {
        if command.get_n_bit(CARRY_FLAG_INDEX) {
            self.update_carry_flag(false);
        }

        if command.get_n_bit(OVERFLOW_FLAG_INDEX) {
            self.update_overflow_flag(false);
        }

        if command.get_n_bit(ZERO_FLAG_INDEX) {
            self.update_zero_flag(false);
        }

        if command.get_n_bit(NEGATIVE_FLAG_INDEX) {
            self.update_negative_flag(false);
        }
    }
}

// One-operand
impl CPU {
    pub fn do_clr(&mut self, memory: &mut Memory, command: Word) {
        let zero = 0x0000u16;

        self.put_word_by_operand(memory, adr_operand(command), zero);

        self.update_status_flags(zero, false, false);
    }

    pub fn do_clrb(&mut self, memory: &mut Memory, command: Word) {
        let zero = 0x00u8;

        self.put_byte_by_operand(memory, adr_operand(command), zero);

        self.update_status_flags(zero, false, false);
    }

    pub fn do_inc(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let word = self.get_word_by_operand(memory, operand);

        let sum = word as LongWord + 0x00000001u32;

        let result = sum as Word;

        self.put_word_by_operand(memory, operand, result);

        self.update_status_flags(result, has_carry(sum), has_signed_overflow(word, result));
    }

    pub fn do_incb(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let byte = self.get_byte_by_operand(memory, operand);

        let sum = byte as Word + 0x0001u16;

        let result = sum as Byte;

        self.put_byte_by_operand(memory, operand, result);

        self.update_status_flags(result, word_has_carry(sum), has_signed_overflow(byte, result));
    }

    pub fn do_dec(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let word = self.get_word_by_operand(memory, operand);

        let sub = word as LongWord - 0x00000001u32;

        let result = sub as Word;

        self.put_word_by_operand(memory, operand, result);

        self.update_status_flags(result, has_carry(sub), has_signed_overflow(word, result));
    }

    pub fn do_decb(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let byte = self.get_byte_by_operand(memory, operand);

        let sub = byte as Word - 0x0001u16;

        let result = sub as Byte;

        self.put_byte_by_operand(memory, operand, result);

        self.update_status_flags(result, word_has_carry(sub), has_signed_overflow(byte, result));
    }

    pub fn do_adc(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let word = self.get_word_by_operand(memory, operand);

        let sum = word as LongWord + if self.carry_flag() { 0x00000001u32 } else { 0x00000000u32 };

        let result = sum as Word;

        self.put_word_by_operand(memory, operand, result);

        self.update_status_flags(result, has_carry(sum), has_signed_overflow(word, result));
    }

    pub fn do_adcb(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let byte = self.get_byte_by_operand(memory, operand);

        let sum = byte as Word + if self.carry_flag() { 0x0001u16 } else { 0x0000u16 };

        let result = sum as Byte;

        self.put_byte_by_operand(memory, operand, result);

        self.update_status_flags(result, word_has_carry(sum), has_signed_overflow(byte, result));
    }

    pub fn do_sdc(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let word = self.get_word_by_operand(memory, operand);

        let sub = word as LongWord - if self.carry_flag() { 0x00000001u32 } else { 0x00000000u32 };

        let result = sub as Word;

        self.put_word_by_operand(memory, operand, result);

        self.update_status_flags(result, has_carry(sub), has_signed_overflow(word, result));
    }

    pub fn do_sdcb(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let byte = self.get_byte_by_operand(memory, operand);

        let sub = byte as Word - if self.carry_flag() { 0x0001u16 } else { 0x0000u16 };

        let result = sub as Byte;

        self.put_byte_by_operand(memory, operand, result);

        self.update_status_flags(result, word_has_carry(sub), has_signed_overflow(byte, result));
    }

    pub fn do_tst(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let word = self.get_word_by_operand(memory, operand);

        self.update_status_flags(word, false, false);
    }

    pub fn do_tstb(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let byte = self.get_byte_by_operand(memory, operand);

        self.update_status_flags(byte, false, false);
    }

    pub fn do_neg(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let word = self.get_word_by_operand(memory, operand);

        let result = word.two_complement();

        self.put_word_by_operand(memory, operand, result);

        self.update_status_flags(result, !result.is_zero(), !has_signed_overflow(word, result));
    }

    pub fn do_negb(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let byte = self.get_byte_by_operand(memory, operand);

        let result = byte.two_complement();

        self.put_byte_by_operand(memory, operand, result);

        self.update_status_flags(result, !result.is_zero(), !has_signed_overflow(byte, result));
    }

    pub fn do_com(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let word = self.get_word_by_operand(memory, operand);

        let result = word.one_complement();

        self.put_word_by_operand(memory, operand, result);

        self.update_status_flags(result, true, false);
    }

    pub fn do_comb(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let byte = self.get_byte_by_operand(memory, operand);

        let result = byte.one_complement();

        self.put_byte_by_operand(memory, operand, result);

        self.update_status_flags(result, true, false);
    }

    pub fn do_ror(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let carry = self.carry_flag();
        let word = self.get_word_by_operand(memory, operand);

        let new_carry = word & 0x0001u16 > 0;

        let result = (word >> 1).set_n_bit(Word::size_bits() - 1, carry);

        self.put_word_by_operand(memory, operand, result);

        self.update_status_flags(result, new_carry, new_carry ^ result.is_negative());
    }

    pub fn do_rorb(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let carry = self.carry_flag();
        let byte = self.get_byte_by_operand(memory, operand);

        let new_carry = byte & 0x01u8 > 0;

        let result = (byte >> 1).set_n_bit(Byte::size_bits() - 1, carry);

        self.put_byte_by_operand(memory, operand, result);

        self.update_status_flags(result, new_carry, new_carry ^ result.is_negative());
    }

    pub fn do_rol(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let carry = self.carry_flag();
        let word = self.get_word_by_operand(memory, operand);

        let new_carry = word.is_negative();

        let result = (word << 1).set_n_bit(0, carry);

        self.put_word_by_operand(memory, operand, result);

        self.update_status_flags(result, new_carry, new_carry ^ result.is_negative());
    }

    pub fn do_rolb(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let carry = self.carry_flag();
        let byte = self.get_byte_by_operand(memory, operand);

        let new_carry = byte.is_negative();

        let result = (byte << 1).set_n_bit(0, carry);

        self.put_byte_by_operand(memory, operand, result);

        self.update_status_flags(result, new_carry, new_carry ^ result.is_negative());
    }

    pub fn do_asr(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);
        
        let word = self.get_word_by_operand(memory, operand);
        let negative = word.is_negative();

        let new_carry = word & 0x0001u16 > 0;

        let result = (word >> 1).set_n_bit(Word::size_bits() - 1, negative);

        self.put_word_by_operand(memory, operand, result);

        self.update_status_flags(result, new_carry, new_carry ^ result.is_negative());
    }

    pub fn do_asrb(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);
        
        let byte = self.get_byte_by_operand(memory, operand);
        let negative = byte.is_negative();

        let new_carry = byte & 0x01u8 > 0;

        let result = (byte >> 1).set_n_bit(Byte::size_bits() - 1, negative);

        self.put_byte_by_operand(memory, operand, result);

        self.update_status_flags(result, new_carry, new_carry ^ result.is_negative());
    }

    pub fn do_asl(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);
        
        let word = self.get_word_by_operand(memory, operand);

        let new_carry =  word.is_negative();

        let result = word << 1;

        self.put_word_by_operand(memory, operand, result);

        self.update_status_flags(result, new_carry, new_carry ^ result.is_negative());
    }

    pub fn do_aslb(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);
        
        let byte = self.get_byte_by_operand(memory, operand);

        let new_carry = byte.is_negative();

        let result = byte << 1;

        self.put_byte_by_operand(memory, operand, result);

        self.update_status_flags(result, new_carry, new_carry ^ result.is_negative());
    }

    pub fn do_swab(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);
        
        let word = self.get_word_by_operand(memory, operand);

        let result = make_word(word.high(), word.low());

        self.put_word_by_operand(memory, operand, result);

        self.update_status_flags(result.low(), false, false);
    }

    pub fn do_sxt(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);
        
        let n_flag = self.negative_flag();

        let result = if n_flag { 0xFFFFu16 } else { 0x0000u16 };

        self.put_word_by_operand(memory, operand, result);

        self.update_zero_flag(!n_flag);
    }

    pub fn do_jmp(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let address = self.get_operand_address(memory, operand);

        self.set_word_reg(PROGRAM_COUNTER_INDEX, address as u16);
    }

    pub fn do_mark(&mut self, memory: &mut Memory, command: Word) {
        let operand = adr_operand(command);

        let offset = operand << 1;

        let stack_pointer_value = self.get_word_from_reg(STACK_POINTER_INDEX);

        let result_stack_pointer = stack_pointer_value + (offset as Word);

        self.set_word_reg(STACK_POINTER_INDEX, result_stack_pointer);
        
        let mp_value = self.get_word_from_reg(MARK_POINTER_INDEX);

        self.set_word_reg(PROGRAM_COUNTER_INDEX, mp_value);

        let stack_top_value = self.pop_stack(memory);

        self.set_word_reg(MARK_POINTER_INDEX, stack_top_value);
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

        self.update_status_flags(result, has_carry(result), false);
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
            self.update_carry_flag(true);
            self.update_overflow_flag(true);
            return;
        }

        let quotient = dst_value / src_value;
        let reminder = dst_value % src_value;

        self.set_word_reg(dst_hi, reminder.low());
        self.set_word_reg(dst, quotient.low());

        self.update_status_flags(quotient.low(), has_carry(quotient), false);
    }

    pub fn do_ash(&mut self, memory: &mut Memory, command: Word) {
        let dst = reg_operand(command);
        
        let src_value = self.get_word_by_operand(memory, adr_operand(command));
        let left_shift = (src_value & 0x0020u16) == 0x0000u16;
        let shift = if left_shift { src_value } else { src_value.two_complement() } & 0x001Fu16;

        let dst_value = self.get_word_from_reg(dst);

        if shift == 0 {
            self.update_status_flags(dst_value, false, false);
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

        let carry = if left_shift {
            intermediate_result.is_negative()
        } else {
            (intermediate_result & 0x0001u16) > 0
        };

        self.update_status_flags(result, carry, has_signed_overflow(dst_value, result));
    }

    pub fn do_ashc(&mut self, memory: &mut Memory, command: Word) {
        let dst = reg_operand(command);

        assert_even_reg(&dst);

        let dst_hi = dst | 0x01u8;

        let src_value = self.get_word_by_operand(memory, adr_operand(command));
        let left_shift = (src_value & 0x0020u16) == 0x0000u16;
        let shift = if left_shift { src_value } else { src_value.two_complement() } & 0x001Fu16;

        let dst_lo_value = self.get_word_from_reg(dst);
        let dst_hi_value = self.get_word_from_reg(dst_hi);

        let dst_value = long_word(dst_lo_value, dst_hi_value);

        if shift == 0 {
            self.update_status_flags(dst_value, false, false);
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

        let carry = if left_shift {
            intermediate_result.is_negative()
        } else {
            (intermediate_result & 0x00000001u32) > 0
        };

        self.update_status_flags(result, carry, has_signed_overflow(dst_value, result));
    }

    pub fn do_xor(&mut self, memory: &mut Memory, command: Word) {
        let dst = adr_operand(command);

        let dst_value = self.get_word_by_operand(memory, dst);
        let src_value = self.get_word_from_reg(reg_operand(command));

        let result = dst_value ^ src_value;

        self.put_word_by_operand(memory, dst, result);
        self.update_status_flags_bitwise(result);
    }

    pub fn do_sob(&mut self, _memory: &mut Memory, command: Word) {
        let offset = adr_operand(command);
        let reg_index = reg_operand(command);

        let result = self.decrement_and_get(reg_index, 0x0001u16);

        if !result.is_zero() {
            let pc_value = self.get_word_from_reg(PROGRAM_COUNTER_INDEX);

            let pc_result = pc_value - (offset << 1) as Word;

            self.set_word_reg(PROGRAM_COUNTER_INDEX, pc_result);
        }
    }

    pub fn do_jsr(&mut self, memory: &mut Memory, command: Word) {
        let reg = reg_operand(command);

        let operand = adr_operand(command);

        let reg_value = self.get_word_from_reg(reg);
        let address = self.get_operand_address(memory, operand);

        self.push_stack(memory, reg_value);

        let pc_value = self.get_word_from_reg(PROGRAM_COUNTER_INDEX);
        self.set_word_reg(reg, pc_value);
        self.set_word_reg(PROGRAM_COUNTER_INDEX, address as Word);
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

        self.update_status_flags(result, has_carry(sum), has_signed_overflow(dst_value, result));
    }

    pub fn do_sub(&mut self, memory: &mut Memory, command: Word) {
        let dst = dst_operand(command);

        let dst_value = self.get_word_by_operand(memory, dst);
        let src_value = self.get_word_by_operand(memory, src_operand(command));

        let sub = dst_value as LongWord - src_value as LongWord;

        let result = sub as Word;

        self.put_word_by_operand(memory, dst, result);

        self.update_status_flags(result, !has_carry(sub), has_signed_overflow(dst_value, result));
    }

    pub fn do_cmp(&mut self, memory: &mut Memory, command: Word) {
        let dst_value = self.get_word_by_operand(memory, dst_operand(command));
        let src_value = self.get_word_by_operand(memory, src_operand(command));

        let sub = src_value as LongWord - dst_value as LongWord;

        let result = sub as Word;

        self.update_status_flags(result, !has_carry(sub), has_signed_overflow(src_value, result));
    }

    pub fn do_cmpb(&mut self, memory: &mut Memory, command: Word) {
        let dst_value = self.get_byte_by_operand(memory, dst_operand(command));
        let src_value = self.get_byte_by_operand(memory, src_operand(command));

        let sub = src_value as Word - dst_value as Word;

        let result = sub as Byte;

        self.update_status_flags(result, !word_has_carry(sub), has_signed_overflow(src_value, result));
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

    pub fn do_trap(&mut self, memory: &mut Memory, _command: Word) {
        self.perform_trap(memory, 0x0018);
    }

    pub fn do_emt(&mut self, memory: &mut Memory, _command: Word) {
        self.perform_trap(memory, 0x001C);
    }
}

// Perform trap
impl CPU {
    pub (in super) fn perform_trap(&mut self, memory: &mut Memory, trap_address: Address) {
        let pc_value = self.get_word_from_reg(PROGRAM_COUNTER_INDEX);
        let psw_value = self.status_word();

        self.push_stack(memory, psw_value);
        self.push_stack(memory, pc_value);

        let new_pc = memory.read_word(trap_address);
        let new_psw = memory.read_word(trap_address + 2);

        self.set_word_reg(PROGRAM_COUNTER_INDEX, new_pc);
        self.set_status_word(new_psw);
    }
}
