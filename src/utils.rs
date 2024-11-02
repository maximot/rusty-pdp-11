
pub type Address = usize;
pub type Byte = u8;
pub type Word = u16;
pub type LongWord = u32;

pub const BYTE_SIZE_BITS: Word = 8;
pub const WORD_SIZE_BYTES: Word = 2;
pub const WORD_SIZE_BITS: Word = BYTE_SIZE_BITS * WORD_SIZE_BYTES;

#[inline(always)]
pub fn word(low: Byte, high: Byte) -> Word {
    (high as Word) << 8 | (low as Word)
}

#[inline(always)]
pub fn has_carry(word: LongWord) -> bool {
    (word & 0xFFFF0000) > 0
}

#[inline(always)]
pub fn word_has_carry(word: Word) -> bool {
    (word & 0xFF00) > 0
}

pub trait Number: Sized {
    fn set_n_bit(&self, n: Byte, value: bool) -> Self;
    fn get_n_bit(&self, n: Byte) -> bool;

    fn register(&self) -> Word;
    fn word(&self) -> Word;
    fn high(&self) -> Byte;
    fn low(&self) -> Byte;

    fn is_zero(&self) -> bool;
    fn is_negative(&self) -> bool;
    fn one_complement(&self) -> Self;
    fn two_complement(&self) -> Self;

    fn size_bytes() -> Byte;
    fn size_bits() -> Byte { Self::size_bytes() << 3 }
}

impl Number for Byte {
    #[inline(always)]
    fn set_n_bit(&self, n: Byte, value: bool) -> Self {
        assert!(n < Self::size_bits());

        match value {
            true => *self | (0x01u8 << n),
            false => *self & (0xFFu8 ^ (0x01u8 << n)),
        }
    }

    #[inline(always)]
    fn get_n_bit(&self, n: Byte) -> bool {
        assert!(n < Self::size_bits());

        (*self >> n & 0x01u8) > 0
    }

    #[inline(always)]
    fn word(&self) -> Word {
        *self as Word
    }

    #[inline(always)]
    fn register(&self) -> Word {
        *self as Word | if self.is_negative() { 0xFF00 } else { 0x0000 }
    }
    
    #[inline(always)]
    fn high(&self) -> Byte {
        self.register().high()
    }
    
    #[inline(always)]
    fn low(&self) -> Byte {
        *self
    }
    
    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == 0x00u8
    }
    
    #[inline(always)]
    fn is_negative(&self) -> bool {
        (*self & 0x80u8) > 0x00u8
    }
    
    #[inline(always)]
    fn one_complement(&self) -> Self {
        !(*self)
    }
    
    #[inline(always)]
    fn two_complement(&self) -> Self {
        self.one_complement() + 0x01u8
    }

    #[inline(always)]
    fn size_bytes() -> Byte { std::mem::size_of::<Byte>() as Byte }
}

impl Number for Word {
    #[inline(always)]
    fn set_n_bit(&self, n: Byte, value: bool) -> Self {
        assert!(n < Self::size_bits());

        match value {
            true => *self | (0x0001u16 << n),
            false => *self & (0xFFFFu16 ^ (0x0001u16 << n)),
        }
    }

    #[inline(always)]
    fn get_n_bit(&self, n: Byte) -> bool {
        assert!(n < Self::size_bits());

        (*self >> n & 0x0001u16) > 0
    }

    #[inline(always)]
    fn word(&self) -> Word {
        self.register()
    }

    #[inline(always)]
    fn register(&self) -> Word {
        *self
    }

    #[inline(always)]
    fn high(&self) -> Byte {
        (*self >> Byte::size_bits()) as Byte 
    }

    #[inline(always)]
    fn low(&self) -> Byte {
        *self as Byte
    }

    #[inline(always)]
    fn is_zero(&self) -> bool {
        *self == 0x0000u16
    }

    #[inline(always)]
    fn is_negative(&self) -> bool {
        (*self & 0x8000u16) > 0x0000u16
    }

    #[inline(always)]
    fn one_complement(&self) -> Self {
        !(*self)
    }

    #[inline(always)]
    fn two_complement(&self) -> Self {
        self.one_complement() + 0x0001u16
    }

    #[inline(always)]
    fn size_bytes() -> Byte { std::mem::size_of::<Word>() as Byte }
}
