use std::{collections::HashMap, sync::{Arc, Mutex}};

use crate::utils::{make_word, Address, Byte, Number, Word};

const MEM_SIZE: usize = 2 << 16;

pub trait MappedMemoryWord {
    fn read_word(&self) -> Word;

    fn write_word(&mut self, word: Word);
}

pub struct SimpleMappedMemoryWord {
    word: Word
}

impl SimpleMappedMemoryWord {
    pub fn new() -> Self {
        SimpleMappedMemoryWord {
            word: 0x0000u16
        }
    }
}

impl MappedMemoryWord for SimpleMappedMemoryWord {
    fn read_word(&self) -> Word {
        self.word
    }

    fn write_word(&mut self, word: Word) {
        self.word = word
    }
}

pub struct Memory {
    bytes: [Byte; MEM_SIZE],
    mapped: HashMap<Address, Arc<Mutex<dyn MappedMemoryWord>>>,
}

impl Memory {
    pub fn new() -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Memory {
            bytes: [0; MEM_SIZE],
            mapped: HashMap::new(),
        }))
    }

    pub fn read_byte(&self, address: Address) -> Byte {
        Self::validate_address(address);

        let mapped_address = address & 0xFFFE;
        if let Some(mapped) = self.get_mapped(address) {
            let word = mapped.lock().unwrap().read_word();

            if address == mapped_address {
                return word.low();
            } else {
                return word.high();
            }
        }
        
        return self.bytes[address];
    }

    pub fn write_byte(&mut self, address: Address, data: Byte) -> Address {
        Self::validate_address(address);
        
        self.bytes[address] = data;

        let mapped_address = address & 0xFFFE;
        if let Some(mapped) = self.get_mapped_mut(address) {
            let result = if address == mapped_address {
                data.register()
            } else {
                make_word(0x00u8, data)
            };

            mapped.lock().unwrap().write_word(result)
        }

        Self::next_byte_address(address)
    }

    pub fn read_word(&self, address: Address) -> Word {
        Self::validate_word_address(address);

        if let Some(mapped) = self.get_mapped(address) {
            return mapped.lock().unwrap().read_word();
        }
    
        let high = self.read_byte(address + 1);
        let low = self.read_byte(address);
        
        return make_word(low, high);
    }

    pub fn write_word(&mut self, address: Address, word: Word) -> Address {
        Self::validate_word_address(address);

        self.write_byte(address, word.low());
        self.write_byte(address + 1, word.high());

        if let Some(mapped) = self.get_mapped_mut(address) {
            mapped.lock().unwrap().write_word(word);
        }

        Self::next_word_address(address)
    }

    pub fn map_word(&mut self, address: Address, mapped_word: Arc<Mutex<dyn MappedMemoryWord>>) -> Address {
        Self::validate_word_address(address);

        self.mapped.insert(address, mapped_word);

        Self::next_word_address(address)
    }

    pub fn unmap_word(&mut self, address: Address) -> Address {
        Self::validate_word_address(address);

        let value = self.read_word(address);

        self.mapped.remove(&address);

        self.write_word(address, value);

        Self::next_word_address(address)
    }

    fn get_mapped_mut(&mut self, address: Address) -> Option<&mut Arc<Mutex<dyn MappedMemoryWord>>> {
        self.mapped.get_mut(&address)
    }

    fn get_mapped(&self, address: Address) -> Option<&Arc<Mutex<dyn MappedMemoryWord>>> {
        self.mapped.get(&address)
    }

    fn validate_address(address: Address) {
        assert!(address < MEM_SIZE - 1);
    }

    fn validate_word_address(address: Address) {
        Self::validate_address(address);
        assert!(address % 2 == 0);
    }

    fn next_word_address(address: Address) -> Address {
        address + 2
    }

    fn next_byte_address(address: Address) -> Address {
        address + 1
    }
}


