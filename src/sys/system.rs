use super::rom::Rom;

pub struct Nes {
    pub rom: Rom
    // pub rom: Rom,
    // pub memory_map: MemoryMap,
    // pub cpu: Cpu
}

impl Nes {

    pub fn new(rom: Rom) -> Nes {
        Nes{rom}
    }

    pub fn reset(){
        // let upper: u8 = memory_map.getRAMValue(0xFFFD) & 0xFF;
        // let lower: u8 =  memory_map.getRAMValue(0xFFFC) & 0xFF;
        // let addr: u16 = (upper << 8) | lower;
        // cpu.program_counter = addr;
        // cpu.init();
    }

    pub fn execute(){
        // TODO:
    }
}