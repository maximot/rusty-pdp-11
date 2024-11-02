use crate::utils::{word, Address, Byte, Number, Word};

const MEM_SIZE: usize = 2 << 16;

pub struct Memory {
    bytes: [Byte; MEM_SIZE]
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            bytes: [0; MEM_SIZE]
        }
    }

    pub fn read_byte(&self, address: Address) -> Byte {
        Self::validate_address(address);
        
        return self.bytes[address];
    }

    pub fn write_byte(&mut self, address: Address, data: Byte) -> Address {
        Self::validate_address(address);
        
        self.bytes[address] = data;

        address + 1
    }

    pub fn read_word(&self, address: Address) -> Word {
        Self::validate_word_address(address);
    
        let high = self.read_byte(address + 1);
        let low = self.read_byte(address);
        
        return word(low, high);
    }

    pub fn write_word(&mut self, address: Address, word: Word) -> Address {
        Self::validate_word_address(address);

        self.write_byte(address, word.low());
        self.write_byte(address + 1, word.high());

        address + 2
    }

    fn validate_address(address: Address) {
        assert!(address < MEM_SIZE - 1);
    }

    fn validate_word_address(address: Address) {
        Self::validate_address(address);
        assert!(address % 2 == 0);
    }
}


