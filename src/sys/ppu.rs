use super::system::Nes;

pub struct Ppu {
    pub ppu_ram: [u8; 0x4000], // TODO: 今は容量適当
    pub ppu_oam: [u8; 0x100],
    pub ppu_reg: [u8; 8],
    // pub ppu_chr_rom: [u8],
    pub ppu_addr_count: u8, //TODO: 直せたら直す
    pub ppu_addr: u16,
    // pub frame_buffer: [i32],
    // pub bg_color_tables: [[u8]],
    // pub sp_color_tables: [[u8]],
    pub attribute_table_cache: [u8; 16*16] // 各16x16pixelの画面領域で使うパレット
    // TODO: PPURAMWrite作る ミラー領域とかの考慮のため
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu{
            ppu_ram: [0; 0x4000],
            ppu_oam: [0; 0x100],
            ppu_reg: [0; 8],
            ppu_addr_count: 0,
            ppu_addr: 0,
            attribute_table_cache: [0; 16*16]
        }
    }

    pub fn write_ppu_addr(&mut self){
        if self.ppu_addr_count == 0 {
            self.ppu_addr &= 0xFF;
            self.ppu_addr |= ((self.ppu_reg[6] & 0xFF) as u16) << 8;
        } else if self.ppu_addr_count == 1 {
            self.ppu_addr = self.ppu_addr | (self.ppu_reg[6] & 0xFF) as u16;
        }
        self.ppu_addr_count = (self.ppu_addr_count + 1) % 2;
    }

    pub fn write_ppu_data(&mut self){
        if self.ppu_addr == 0x3F00 || self.ppu_addr == 0x3F10 {
            self.ppu_ram[0x3F00] = self.ppu_reg[7];
            self.ppu_ram[0x3F10] = self.ppu_reg[7]; // mirror
        }
        else if self.ppu_addr == 0x3F04 || self.ppu_addr == 0x3F14 {
            self.ppu_ram[0x3F04] = self.ppu_reg[7];
            self.ppu_ram[0x3F14] = self.ppu_reg[7]; // mirror
        }
        else if self.ppu_addr == 0x3F08 || self.ppu_addr == 0x3F18 {
            self.ppu_ram[0x3F08] = self.ppu_reg[7];
            self.ppu_ram[0x3F18] = self.ppu_reg[7]; // mirror
        }
        else if self.ppu_addr == 0x3F0C || self.ppu_addr == 0x3F1C {
            self.ppu_ram[0x3F0C] = self.ppu_reg[7];
            self.ppu_ram[0x3F1C] = self.ppu_reg[7]; // mirror
        }
        else {
            self.ppu_ram[self.ppu_addr as usize] = self.ppu_reg[7];
        }
        let mut address_inc = 1;
        if (self.ppu_reg[0] & 0x04) > 0 { // $2000の値によって32byteインクリメント
            address_inc = 32;
        }
        self.ppu_addr += address_inc;
    }

    pub fn refreshAttributeTable(&mut self){
        let main_screen = self.ppu_reg[0] & 0x03;
        let start_addr = 0x2000 + (main_screen as u16 * 0x400);
        let end_addr = start_addr + 32 * 30;
        let attribute_table_start_addr = start_addr + 0x3C0;
        for attribute_table_addr in attribute_table_start_addr..end_addr{
            let value = self.ppu_ram[attribute_table_addr as usize] & 0xFF;
            let topLeft = value & 0x03;
            let topRight = (value & 0x0C) >> 2;
            let bottomLeft = (value & 0x30) >> 4;
            let bottomRight = (value & 0xC0) >> 6;
            let attributeTableY = attribute_table_addr / 8;
            let areaY = 2 * attributeTableY;
            let attributeTableX = attribute_table_addr % 8;
            let areaX = 2 * attributeTableX;
            self.attribute_table_cache[(areaY * 8 + areaX) as usize] = topLeft;
            self.attribute_table_cache[(areaY * 8 + areaX + 1) as usize] = topRight;
            self.attribute_table_cache[((areaY + 1) * 8 + areaX) as usize] = bottomLeft;
            self.attribute_table_cache[((areaY + 1) * 8 + areaX + 1) as usize] = bottomRight;
        }
    }

    pub fn next_cycle(&mut self, frame_buffer: &mut [u8]){
        self.refreshAttributeTable();
        self.draw(frame_buffer);
    }

    pub fn draw(&self, frame_buffer: &mut [u8]){
        const BLOCK_WIDTH: u32 = 256 / 8;
        let main_screen = self.ppu_reg[0] & 0x03;
        let start_addr = 0x2000 + (main_screen as u16 * 0x400);
        let end_addr = start_addr + 32 * 30;

        // BG描画
        for addr in start_addr..end_addr {
            let tile_id = self.ppu_ram[addr as usize];
            if tile_id > 0 {
                let offset_block_x = (addr - 0x2000) as u32 % BLOCK_WIDTH;
                let offset_block_y = (addr - 0x2000) as u32 / BLOCK_WIDTH;
                let offset_x = offset_block_x * 8;
                let offset_y = offset_block_y * 8;
                // 8x8の描画
                for color_table_index in 0..64 {
                    let x = offset_x + (color_table_index % 8);
                    let y = offset_y + (color_table_index / 8);
                    let attX = offset_x % 16;
                    let attY = offset_y / 16;
                    let palette = self.attribute_table_cache[(attX * 16 + attY) as usize];
                    // let tmp = self.bgColorTables[tileId][colorTableIndex] & 0xFF;
                    let frame_buffer_index = y * 256 + x;
                    //sys.frame_buffer[frame_buffer_index as usize] = getColor(getBGColorId(palette, tmp));
                    frame_buffer[(frame_buffer_index * 4 + 0) as usize] = 127;
                    frame_buffer[(frame_buffer_index * 4 + 1) as usize] = 127;
                    frame_buffer[(frame_buffer_index * 4 + 2) as usize] = 127;
                    frame_buffer[(frame_buffer_index * 4 + 3) as usize] = 255;
                }
            }
        }


        // Sprite描画
        for spriteAddr in (0..0x100).step_by(4) {
            let tileY       = self.ppu_oam[spriteAddr + 0] & 0xFF;
            let tileX       = self.ppu_oam[spriteAddr + 3] & 0xFF;
            if tileY >= 240 {
                break;
            }

            let mut tileId = 0x00;
            if (self.ppu_reg[0] & 0x20) > 0 { // スプライトサイズ8 * 16の場合
                tileId = self.ppu_oam[spriteAddr + 1] & 0xF7;
                // TODO: スプライト下半分の処理
                // tileId = (ppuOAM[spriteAddr + 1] & 0xF7) | 1;
            }
            else {
                tileId = self.ppu_oam[spriteAddr + 1] & 0xFF;
            }
            let attr    = self.ppu_oam[spriteAddr + 2] & 0xFF;
            if tileId > 0 {
                let palette = attr & 0x03;

                for colorTableIndex in 0..64 {
                    let x = tileX + (colorTableIndex % 8);
                    let y = tileY + (colorTableIndex / 8);
                    // let tmp = spColorTables[tileId][colorTableIndex] & 0xFF;
                    // let colorId = getSPColorId(palette, tmp);
                    // if( colorId > 0 ){ // 背景色ではない
                    //     let color = getColor(colorId);
                    //     frame_buffer[y * 256 + x] = color;
                    // }
                    // tileId
                    let frame_buffer_index = y as u16 * 256 + x as u16;
                    frame_buffer[(frame_buffer_index * 4 + 0) as usize] = 127;
                    frame_buffer[(frame_buffer_index * 4 + 1) as usize] = 127;
                    frame_buffer[(frame_buffer_index * 4 + 2) as usize] = 127;
                    frame_buffer[(frame_buffer_index * 4 + 3) as usize] = 255;
                }
            }
        }
    }
}