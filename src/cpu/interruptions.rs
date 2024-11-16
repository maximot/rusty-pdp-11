
use super::{blocking_queue::BlockingQueue, Address, Byte};

pub struct InterruptionBus {
    interruption_br4: BlockingQueue<Address>,
    interruption_br5: BlockingQueue<Address>,
    interruption_br6: BlockingQueue<Address>,
    interruption_br7: BlockingQueue<Address>,
}

impl InterruptionBus {
    pub fn new() -> Self {
        InterruptionBus {
            interruption_br4: BlockingQueue::new(),
            interruption_br5: BlockingQueue::new(),
            interruption_br6: BlockingQueue::new(),
            interruption_br7: BlockingQueue::new(),
        }
    }

    pub fn interrupt(&mut self, vector_address: Address, priority: Byte) {
        assert!(priority <= 0x07);
        assert!(priority > 0x03);

        match priority {
            0x04 => self.interruption_br4.push(vector_address),
            0x05 => self.interruption_br5.push(vector_address),
            0x06 => self.interruption_br6.push(vector_address),
            0x07 => self.interruption_br7.push(vector_address),
            _ => panic!(),
        }
    }

    pub fn next_interruption_if_any(&self, priority: Byte) -> Option<Address> {
        assert!(priority <= 0x07);

        if priority == 0x07 { return None; };

        if let Some(l7_interruption) = self.interruption_br7.pop() {
            return Some(l7_interruption);
        };

        if priority == 0x06 { return None; };

        if let Some(l6_interruption) = self.interruption_br6.pop() {
            return Some(l6_interruption);
        };

        if priority == 0x05 { return None; };

        if let Some(l5_interruption) = self.interruption_br5.pop() {
            return Some(l5_interruption);
        };

        if priority == 0x04 { return None; };

        if let Some(l4_interruption) = self.interruption_br4.pop() {
            return Some(l4_interruption);
        };

        return None;
    }
}