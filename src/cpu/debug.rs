use super::{Word, CPU, REG_COUNT};

pub struct CPUStateDump {
    pub status: Word,
    pub registers: [Word; REG_COUNT],
    pub running: bool
}

impl CPU {
    pub fn dump_state(&self) -> CPUStateDump {
        CPUStateDump {
            status: self.status,
            registers: self.registers.clone(),
            running: self.running,
        }
    }

    pub (in super) fn trace_registers(&self) {
        trace!("#######################");
        for (i, register) in self.registers.iter().enumerate() {
            trace!("Reg{i} = 0x{register:04X}");
        }
        trace!("PSW = 0x{:04X}", self.status);
        trace!("#######################");
    }
}