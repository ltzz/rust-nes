use super::{cpu::Cpu, memory_map::MemoryMap, ppu::Ppu, rom::Rom};

pub struct Nes {
    pub memory_map: MemoryMap,
    pub cpu: Cpu,
    pub frame_buffer: Vec<u8>
}

impl Nes {

    pub fn new(rom: Rom) -> Nes {
        let ppu = Ppu::new();
        let memory_map = MemoryMap::new(rom, ppu);
        let cpu = Cpu::new();
        let mut frame_buffer: Vec<u8> = vec!(0; 256*240*4);

        // for y in 0..240 {
        //     for x in 0..256 {
        //         if x + y < 128 {
        //             frame_buffer[(y * 256 + x) * 4 + 0] = 0 as u8;
        //             frame_buffer[(y * 256 + x) * 4 + 1] = 255 as u8;
        //             frame_buffer[(y * 256 + x) * 4 + 2] = 0 as u8;
        //         }
        //         else {
        //             frame_buffer[(y * 256 + x) * 4 + 0] = 255 as u8;
        //             frame_buffer[(y * 256 + x) * 4 + 1] = 255 as u8;
        //             frame_buffer[(y * 256 + x) * 4 + 2] = 255 as u8;
        //         }
        //         frame_buffer[(y * 256 + x) * 4 + 3] = 255 as u8;
        //     }
        // }

        Nes{memory_map, cpu, frame_buffer}
    }

    pub fn reset(&mut self){
        let upper: u8 = self.memory_map.get_from_address(0xFFFD) & 0xFF;
        let lower: u8 =  self.memory_map.get_from_address(0xFFFC) & 0xFF;
        let addr: u16 = ((upper as u16) << 8) | lower as u16;
        self.cpu.program_counter = addr as u32;
        self.cpu.init();
    }

    pub fn execute(&mut self){
        self.cpu.next_cycle(&mut self.memory_map);
        self.memory_map.ppu.next_cycle(&mut self.frame_buffer);
    }
}