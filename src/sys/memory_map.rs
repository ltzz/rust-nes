use super::system::Nes;

pub struct MemoryMap {
    pub wram: Vec<u8>
}

impl MemoryMap {
    pub fn new() -> MemoryMap {
        let wram = [0; 0x800].to_vec();
        MemoryMap{wram}
    }

    pub fn get_from_address(&self, nes: &Nes, address: u32) -> u8{
        if 0x0000 <= address && address < 0x2000 {
            //WRAM MIRROR * 3
            return self.wram[address as usize % 0x800];
        }
        else if address < 0x2008 {
            // ppu i/o
            if address == 0x2002 {
                // return ppu.ppuReg[2];
            }
            else if address == 0x2006 {
            }
            else if address == 0x2007 {
                // ppu.readPPUData();
            }
        }
        else if address < 0x4000 {
            // ppu i/o mirror * 1023
        }
        else if address < 0x4020 {
            // apu i/o, pad
            if address == 0x4016 {
                let value:u8 = 0;
                // value |= joyPad.buttonReadFromIO() ? 0x01 : 0x00;
                return value;
            }
        }
        else if address < 0x6000 {
            // exrom
        }
        else if address < 0x8000 {
            // exram
        }
        else if address < 0xC000 {
            // prg-rom low
            return nes.rom.prg_rom[(address as usize - 0x8000) % nes.rom.prg_rom.len()];
        }
        else if address <= 0xFFFF {
            // prg-ram high
            return nes.rom.prg_rom[(address as usize - 0x8000) % nes.rom.prg_rom.len()];
        }
        else{
            // ppu
        }
        return 0x00;
    }

    pub fn set_from_address(&mut self, address: u32, value: u8) {
        if 0x0000 <= address && address < 0x2000 {
            //WRAM MIRROR * 3
            let wram_element = &mut self.wram[(address % 0x800) as usize];
            *wram_element = value;
        } else if address < 0x2008 {
            // // ppu i/o
            // ppu.ppuReg[address - 0x2000] = value;
            // if (address == 0x2006) {
            //     ppu.writePpuAddr();
            // }
            // else if(address == 0x2007) {
            //     ppu.writePPUData();
            // }
            // else if( address == 0x2003 ){
            //     final int a = 1;
            // }
            // else if( address == 0x2004 ){
            //     final int a = 1;
            // }
            // else if( address == 0x2005 ){
            //     final int a = 1;
            // }
        } else if address == 0x4014 {
            // ppu.spriteDMA(value, wram);
        } else if address == 0x4016 {
            // joyPad.buttonResetFromIO();
        }
    }

    
    pub fn get_from_address16(&self, sys: &Nes, address: u32) -> u16{
        let lower = self.get_from_address(sys, (address + 0) & 0xFFFF) & 0xFF;
        let upper = self.get_from_address(sys, (address + 1) & 0xFFFF) & 0xFF;
        let value: u16 = (upper << 8) as u16 | lower as u16;
        return value;
    }
    
    pub fn get_from_address16_by_address8(&self, sys: &Nes, address: u8) -> u16{
        let lower = self.get_from_address(sys, ((address + 0) & 0xFF) as u32) & 0xFF;
        let upper = self.get_from_address(sys, ((address + 1) & 0xFF) as u32) & 0xFF;
        let value: u16 = (upper << 8) as u16 | lower as u16;
        return value;
    }

    pub fn get_from_address_in_page(&self, sys: &Nes, address: u32) -> u16{
        let page = address >> 8;
        let lower = self.get_from_address(sys, (page << 8) | (address + 0) & 0xFF) & 0xFF;
        let upper = self.get_from_address(sys, (page << 8) | (address + 1) & 0xFF) & 0xFF;
        let value: u16 = (upper << 8) as u16 | lower as u16;
        return value;
    }
}

