use super::{cpu::{Cpu, make_nmi_interrupt}, ppu::Ppu, rom::Rom};

pub struct MemoryMap {
    pub rom: Rom,
    pub wram: Vec<u8>,
    pub ppu: Ppu
}

impl MemoryMap {
    pub fn new(rom: Rom, ppu: Ppu) -> MemoryMap {
        let wram = vec!(0; 0x800);
        MemoryMap{rom, wram, ppu}
    }

    pub fn get_from_address(&mut self, address: u32) -> u8{
        if 0x0000 <= address && address < 0x2000 {
            //WRAM MIRROR * 3
            return self.wram[(address % 0x800) as usize];
        }
        else if address < 0x2008 {
            // ppu i/o
            if address == 0x2000 {
                return self.ppu.ppu_reg[0];
            }
            else if address == 0x2001 {
                return self.ppu.ppu_reg[1];
            }
            else if address == 0x2002 {
                return self.ppu.ppu_reg[2];
            }
            else if address == 0x2004 {
                return self.ppu.read_oam_data();
            }
            else if address == 0x2005 {
                return self.ppu.ppu_reg[5];
            }
            else if address == 0x2006 {
            }
            else if address == 0x2007 {
                return self.ppu.read_ppu_data();
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
            return self.rom.prg_rom[(address as usize - 0x8000) % self.rom.prg_rom.len()];
        }
        else if address <= 0xFFFF {
            // prg-ram high
            return self.rom.prg_rom[(address as usize - 0x8000) % self.rom.prg_rom.len()];
        }
        else{
            // ppu
        }
        return 0x00;
    }

    pub fn set_from_address(&mut self, address: u32, value: u8) {
        if 0x0000 <= address && address < 0x2000 {
            //WRAM MIRROR * 3
            self.wram[(address % 0x800) as usize] = value;
        } else if address < 0x2008 {
            // ppu i/o
            self.ppu.ppu_reg[(address - 0x2000) as usize] = value;
            if address == 0x2005 {
                self.ppu.write_ppu_scroll();
            }
            else if address == 0x2006 {
                self.ppu.write_ppu_addr();
            }
            else if address == 0x2007 {
                self.ppu.write_ppu_data();
            }
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
            self.ppu.sprite_dma(value, &self.wram);
        } else if address == 0x4016 {
            // joyPad.buttonResetFromIO();
        }
    }

    
    pub fn get_from_address16(&mut self, address: u32) -> u16{
        let lower = self.get_from_address(address + 0) & 0xFF;
        let upper = self.get_from_address(address + 1) & 0xFF;
        let value: u16 = ((upper as u16) << 8) as u16 | lower as u16;
        return value;
    }
    
    pub fn get_from_address16_by_address8(&mut self, address: u8) -> u16{
        let lower = self.get_from_address(((address + 0) & 0xFF) as u32) & 0xFF;
        let upper = self.get_from_address(((address.wrapping_add(1)) & 0xFF) as u32) & 0xFF;
        let value: u16 = ((upper as u16) << 8) as u16 | lower as u16;
        return value;
    }

    pub fn get_from_address_in_page(&mut self, address: u32) -> u16{
        let page = address >> 8;
        let lower = self.get_from_address((page << 8) | (address + 0) & 0xFF) & 0xFF;
        let upper = self.get_from_address((page << 8) | (address + 1) & 0xFF) & 0xFF;
        let value: u16 = ((upper as u16) << 8) as u16 | lower as u16;
        return value;
    }

    pub fn ppu_next_cycle(&mut self, frame_buffer: &mut [u8], cpu: &mut Cpu){
        self.ppu.next_cycle(frame_buffer, &self.rom);
        if self.ppu.current_line == 241 {
            // VBLANKフラグ=1
            self.ppu.ppu_reg[0] |= 0x80;
            // nmi割り込み
            make_nmi_interrupt(cpu, self)
        }
        if self.ppu.current_line == 261 {
            // VBLANKフラグ=0
            self.ppu.ppu_reg[0] &= 0x7F;
        }
        if self.ppu.current_line == 341 {
            self.ppu.current_line = 0;
        }
    }
}

