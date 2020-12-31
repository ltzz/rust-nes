use super::{cpu::Cpu, memory_map::MemoryMap, rom::Rom};

pub struct Nes {
    pub rom: Rom,
    pub memory_map: MemoryMap,
    pub cpu: Cpu
}

impl Nes {

    pub fn new(rom: Rom) -> Nes {
        let memory_map = MemoryMap::new();
        let cpu = Cpu::new();
        Nes{rom, memory_map, cpu}
    }

    pub fn reset(&mut self){
        let upper: u8 = self.memory_map.get_from_address(self, 0xFFFD) & 0xFF;
        let lower: u8 =  self.memory_map.get_from_address(self, 0xFFFC) & 0xFF;
        let addr: u16 = ((upper as u16) << 8) | lower as u16;
        self.cpu.program_counter = addr as u32;
        self.cpu.init();
    }

    pub fn execute(){
        // TODO:
    }
}