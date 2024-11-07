use std::collections::HashMap;

use crate::{ mem::Memory, utils::{Byte, Number, Word} };

use super::CPU;

// https://www.teach.cs.toronto.edu/~ajr/258/pdp11.pdf
// https://en.wikipedia.org/wiki/PDP-11_architecture

/**
 * Zero-operand command opcode mask
 * 1111111111111111
 * FEDCBA9876543210
 */
pub const O_0_MASK: Word = 0xFFFF;

/**
 * One-operand command opcode mask
 * 1111111111000000
 * FEDCBA9876543210
 */
pub const O_1_MASK: Word = 0xFFC0;

/**
 * One-and-a-half-operand command opcode mask
 * 1111111000000000
 * FEDCBA9876543210
 */
pub const O_1_5_MASK: Word = 0xFE00;

/**
 * Two-operand command opcode mask
 * 1111000000000000
 * FEDCBA9876543210
 */
pub const O_2_MASK: Word = 0xF000;

/**
 * Branch command opcode mask
 * 1111111100000000
 * FEDCBA9876543210
 */
pub const B_MASK: Word = 0xFF00;   

/**
 * Operand mask
 * 0000000000111111
 * FEDCBA9876543210
 */
pub const O_MASK: Word = 0x003F;

/**
 * Register mask
 * 0000000000000111
 * FEDCBA9876543210
 */
pub const REG_MASK: Word = 0x0007;

// For Two-operand instructions

pub fn dst_operand(command: Word) -> Byte {
    (command & O_MASK).low()
}

pub fn src_operand(command: Word) -> Byte {
    ((command >> 6) & O_MASK).low()
}

// For branch instructions

pub fn branch_offset(command: Word) -> Word {
    command.low().register() << 1
}

// For One-operand and One-and-a-half-operand instructions

pub fn reg_operand(command: Word) -> Byte {
    ((command >> 6) & REG_MASK).low()
}

pub fn adr_operand(command: Word) -> Byte {
    dst_operand(command)
}

pub struct Command(pub Word, pub &'static str, pub fn(&mut CPU, &mut Memory, Word));

pub struct Commands {
    pub o_0_commands: HashMap<Word, Command>,
    pub o_1_commands: HashMap<Word, Command>,
    pub o_1_5_commands: HashMap<Word, Command>,
    pub o_2_commands: HashMap<Word, Command>,
    pub b_commands: HashMap<Word, Command>,
}

impl Default for Commands {
    fn default() -> Self {
        Self { 
            // TODO: impl
            o_0_commands: HashMap::from([
                command(0x0000, "HALT", CPU::do_halt),
                command(0x0001, "WAIT", CPU::do_wait),
                command(0x0005, "RESET", CPU::do_nop), // TODO
                command(0x00A0, "NOP", CPU::do_nop),
            ]), 
            // TODO: Opcode + impl
            o_1_commands: HashMap::from([
                command(0x0040, "JMP", CPU::do_jmp),
                command(0x0A00, "CLR", CPU::do_clr),
                command(0x8A00, "CLRB", CPU::do_clrb),
                command(0x0A80, "INC", CPU::do_inc),
                command(0x8A80, "INCB", CPU::do_incb),
                command(0x0AC0, "DEC", CPU::do_dec),
                command(0x8AC0, "DECB", CPU::do_decb),
                command(0x0B40, "ADC", CPU::do_adc),
                command(0x8B40, "ADCB", CPU::do_adcb),
                command(0x0B80, "SBC", CPU::do_sdc),
                command(0x8B80, "SBCB", CPU::do_sdcb),
                command(0x0BC0, "TST", CPU::do_tst),
                command(0x8BC0, "TSTB", CPU::do_tstb),
                command(0x0B00, "NEG", CPU::do_neg),
                command(0x8B00, "NEGB", CPU::do_negb),
                command(0x0A40, "COM", CPU::do_com),
                command(0x8A40, "COMB", CPU::do_comb),
                command(0x0C00, "ROR", CPU::do_ror),
                command(0x8C00, "RORB", CPU::do_rorb),
                command(0x0C40, "ROL", CPU::do_rol),
                command(0x8C40, "ROLB", CPU::do_rolb),
                command(0x0C80, "ASR", CPU::do_asr),
                command(0x8C80, "ASRB", CPU::do_asrb),
                command(0x0CC0, "ASL", CPU::do_asl),
                command(0x8CC0, "ASLB", CPU::do_aslb),
                command(0x00C0, "SWAB", CPU::do_swab),
                command(0x0DC0, "SXT", CPU::do_sxt),
            ]), 
            // DONE: 
            o_1_5_commands: HashMap::from([
                command(0x7000, "MUL", CPU::do_mul),
                command(0x7200, "DIV", CPU::do_div),
                command(0x7400, "ASH", CPU::do_ash),
                command(0x7600, "ASHC", CPU::do_ashc),
                command(0x7800, "XOR", CPU::do_xor),
                command(0x7E00, "SOB", CPU::do_sob),
            ]),
            // DONE: 
            o_2_commands: HashMap::from([
                command(0x1000, "MOV", CPU::do_mov),
                command(0x9000, "MOVB", CPU::do_movb),
                command(0x2000, "CMP", CPU::do_cmp),
                command(0xA000, "CMPB", CPU::do_cmpb),
                command(0x3000, "BIT", CPU::do_bit),
                command(0xB000, "BITB", CPU::do_bitb),
                command(0x4000, "BIC", CPU::do_bic),
                command(0xC000, "BICB", CPU::do_bicb),
                command(0x5000, "BIS", CPU::do_bis),
                command(0xD000, "BISB", CPU::do_bisb),
                command(0x6000, "ADD", CPU::do_add),
                command(0xE000, "SUB", CPU::do_sub),
            ]), 
            // DONE:
            b_commands: HashMap::from([
                command(0x0100, "BR", CPU::do_br),
                command(0x0200, "BNE", CPU::do_bne),
                command(0x0300, "BEQ", CPU::do_beq),
                command(0x0400, "BGE", CPU::do_bge),
                command(0x0500, "BLT", CPU::do_blt),
                command(0x0600, "BGT", CPU::do_bgt),
                command(0x0700, "BLE", CPU::do_ble),
                command(0x8000, "BPL", CPU::do_bpl),
                command(0x8100, "BMI", CPU::do_bmi),
                command(0x8200, "BHI", CPU::do_bhi),
                command(0x8300, "BLOS", CPU::do_blos),
                command(0x8400, "BVC", CPU::do_bvc),
                command(0x8500, "BVS", CPU::do_bvs),
                command(0x8600, "BHIS/BCC", CPU::do_bcc),
                command(0x8700, "BCS/BLO", CPU::do_bcs),
            ]),
        }
    }
}

pub const UNKNOWN_COMMAND: Command = Command(0xFFFF, "UNKNOWN", CPU::do_panic);

fn command(opcode: Word, name: &'static str, interpretation: fn(&mut CPU, &mut Memory, Word)) -> (Word, Command) {
    (opcode, Command(opcode, name, interpretation))
}

impl CPU {
    pub (in super) fn command(&self, command_word: Word) -> &Command {
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
