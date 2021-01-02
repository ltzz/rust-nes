use super::{memory_map::{self, MemoryMap}, rom::Rom, system::Nes};

pub struct Ppu {
    pub ppu_ram: [u8; 0x4000], // TODO: 今は容量適当
    pub ppu_oam: [u8; 0x100],
    pub ppu_reg: [u8; 8],
    // pub ppu_chr_rom: [u8],
    pub ppu_addr_count: u8, //TODO: 直せたら直す
    pub ppu_addr: u16,
    // pub frame_buffer: [i32],
    pub bg_color_tables: Vec<Vec<u8>>,// [[u8; 64]; 4096],
    pub sp_color_tables: Vec<Vec<u8>>,// [[u8; 64]; 4096],
    pub attribute_table_cache: [u8; 16*16] // 各16x16pixelの画面領域で使うパレット
    // TODO: PPURAMWrite作る ミラー領域とかの考慮のため
}


const fn convert_to_rgb24(octal: u32) -> u32{
    let b = octal & 0x07;
    let g = (octal & (0x07 << 3)) >> 3;
    let r = (octal & (0x07 << 6)) >> 6;
    let rgb = (((b * 255) / 7) << 16) |
            (((g * 255) / 7) << 8) |
            ((r * 255) / 7);
    return rgb;
}
const COLOR_PALETTE_OCTAL: [u32; 64] = [
    0333,0014,0006,0326,0403,0503,0510,0420,0320,0120,0031,0040,0022,0000,0000,0000,
    0555,0036,0027,0407,0507,0704,0700,0630,0430,0140,0040,0053,0044,0000,0000,0000,
    0777,0357,0447,0637,0707,0737,0740,0750,0660,0360,0070,0276,0077,0000,0000,0000,
    0777,0567,0657,0757,0747,0755,0764,0772,0773,0572,0473,0276,0467,0000,0000,0000
];

const COLOR_PALETTE: [u8; 64 * 3] = generate_color_palette();

const fn get_color(color: u8) -> u32{
    return convert_to_rgb24(COLOR_PALETTE_OCTAL[color as usize]);
}

const fn generate_color_palette() -> [u8; 64 * 3]{
    let mut result = [0u8; 64 * 3];
    let mut i = 0;
    while i < 64{
        let color = get_color(i);
        result[(i * 3 + 0)as usize] = (color & 0xFF) as u8;
        result[(i * 3 + 1)as usize] = ((color >> 8) & 0xFF) as u8;
        result[(i * 3 + 2)as usize] = ((color >> 16) & 0xFF) as u8;
        i += 1;
    }
    result
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu{
            ppu_ram: [0; 0x4000],
            ppu_oam: [0; 0x100],
            ppu_reg: [0; 8],
            ppu_addr_count: 0,
            ppu_addr: 0,
            bg_color_tables: vec![vec![0u8; 64]; 0x1000],
            sp_color_tables: vec![vec![0u8; 64]; 0x1000],
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

    pub fn refresh_attribute_table(&mut self){
        let main_screen = self.ppu_reg[0] & 0x03;
        let start_addr = 0x2000 + (main_screen as u16 * 0x400);
        let end_addr = start_addr + 32 * 30;
        let attribute_table_start_addr = start_addr + 0x3C0;
        for attribute_table_addr in attribute_table_start_addr..end_addr{
            let value = self.ppu_ram[attribute_table_addr as usize] & 0xFF;
            let top_left = value & 0x03;
            let top_right = (value & 0x0C) >> 2;
            let bottom_left = (value & 0x30) >> 4;
            let bottom_right = (value & 0xC0) >> 6;
            let attributeTableY = attribute_table_addr / 8;
            let area_y = 2 * attributeTableY;
            let attributeTableX = attribute_table_addr % 8;
            let area_x = 2 * attributeTableX;
            self.attribute_table_cache[(area_y * 8 + area_x) as usize] = top_left;
            self.attribute_table_cache[(area_y * 8 + area_x + 1) as usize] = top_right;
            self.attribute_table_cache[((area_y + 1) * 8 + area_x) as usize] = bottom_left;
            self.attribute_table_cache[((area_y + 1) * 8 + area_x + 1) as usize] = bottom_right;
        }
    }

    pub fn refresh_color_tables(&mut self, rom: &Rom){
        let bg_offset_addr: u32 = if (self.ppu_reg[0] & 0x10) > 0 {0x1000} else {0};
        let sp_offset_addr: u32 = if (self.ppu_reg[0] & 0x08) > 0 {0x1000} else {0};
        for tile_id in 0..0x100{
            let tileIdOffsetAddress = tile_id * 16;

            for chr_index in 0..8{ // 前半
                let chr_value = rom.chr_rom[(bg_offset_addr + tileIdOffsetAddress + chr_index)as usize];
                let y_cache_index = chr_index * 8;
                for xIndex in 0..8{
                    let shift = 7 - xIndex;
                    self.bg_color_tables[tile_id as usize][(y_cache_index + xIndex) as usize] = (chr_value & (1 << shift)) >> shift;
                }
            }
            for chr_index in 0..8{ // 後半
                let chr_value = rom.chr_rom[(bg_offset_addr + tileIdOffsetAddress + 8 + chr_index)as usize];
                let y_cache_index = chr_index * 8;
                for xIndex in 0..8{
                    let shift = 7 - xIndex;
                    self.bg_color_tables[tile_id as usize][(y_cache_index + xIndex) as usize] = ((chr_value & (1 << shift)) >> shift) * 2;
                }
            }
        }
        for tile_id in 0..0x100{
            let tileIdOffsetAddress = tile_id * 16;

            for chr_index in 0..8{ // 前半
                let chr_value = rom.chr_rom[(sp_offset_addr + tileIdOffsetAddress + chr_index)as usize];
                let y_cache_index = chr_index * 8;
                for xIndex in 0..8{
                    let shift = 7 - xIndex;
                    self.sp_color_tables[tile_id as usize][(y_cache_index + xIndex) as usize] = (chr_value & (1 << shift)) >> shift;
                }
            }
            for chr_index in 0..8{ // 後半
                let chr_value = rom.chr_rom[(sp_offset_addr + tileIdOffsetAddress + 8 + chr_index)as usize];
                let y_cache_index = chr_index * 8;
                for xIndex in 0..8{
                    let shift = 7 - xIndex;
                    self.sp_color_tables[tile_id as usize][(y_cache_index + xIndex) as usize] = ((chr_value & (1 << shift)) >> shift) * 2;
                }
            }
        }
    }

    pub fn next_cycle(&mut self, frame_buffer: &mut [u8], rom: &Rom){
        self.refresh_attribute_table();
        self.refresh_color_tables(rom);
        self.draw(frame_buffer);
    }
    
    fn get_bg_color_id(&self, palette: i32, num: i32) -> u8{
        let palette_table_bg_addr = 0x3F00;
        return self.ppu_ram[(palette_table_bg_addr + 4 * palette + num) as usize];
    }

    fn get_sp_color_id(&self, palette: i32, num: i32) -> u8{
        let palette_table_sp_addr = 0x3F10;
        return self.ppu_ram[(palette_table_sp_addr + 4 * palette + num) as usize];
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
                    let frame_buffer_index = y * 256 + x;
                    let tmp = self.bg_color_tables[tile_id as usize][color_table_index as usize] & 0xFF;
                    let color_id = self.get_bg_color_id(palette.into(), tmp.into());
                    let r = COLOR_PALETTE[(color_id * 3 + 0) as usize];
                    let g = COLOR_PALETTE[(color_id * 3 + 1) as usize];
                    let b = COLOR_PALETTE[(color_id * 3 + 2) as usize];
                    //sys.frame_buffer[frame_buffer_index as usize] = getColor(getBGColorId(palette, tmp));
                    frame_buffer[(frame_buffer_index * 4 + 0) as usize] = r;
                    frame_buffer[(frame_buffer_index * 4 + 1) as usize] = g;
                    frame_buffer[(frame_buffer_index * 4 + 2) as usize] = b;
                }
            }
        }


        // Sprite描画
        for sprite_addr in (0..0x100).step_by(4) {
            let tileY       = self.ppu_oam[sprite_addr + 0] & 0xFF;
            let tileX       = self.ppu_oam[sprite_addr + 3] & 0xFF;
            if tileY >= 240 {
                break;
            }

            let mut tile_id = 0x00;
            if (self.ppu_reg[0] & 0x20) > 0 { // スプライトサイズ8 * 16の場合
                tile_id = self.ppu_oam[sprite_addr + 1] & 0xF7;
                // TODO: スプライト下半分の処理
                // tileId = (ppuOAM[spriteAddr + 1] & 0xF7) | 1;
            }
            else {
                tile_id = self.ppu_oam[sprite_addr + 1] & 0xFF;
            }
            let attr    = self.ppu_oam[sprite_addr + 2] & 0xFF;
            if tile_id > 0 {
                let palette = attr & 0x03;

                for color_table_index in 0..64 {
                    let x = tileX + (color_table_index % 8);
                    let y = tileY + (color_table_index / 8);
                    let tmp = self.sp_color_tables[tile_id as usize][color_table_index as usize] & 0xFF;
                    
                    let color_id = self.get_sp_color_id(palette.into(), tmp.into());
                    if color_id > 0 { // 背景色ではない
                        let r = COLOR_PALETTE[(color_id * 3 + 0) as usize];
                        let g = COLOR_PALETTE[(color_id * 3 + 1) as usize];
                        let b = COLOR_PALETTE[(color_id * 3 + 2) as usize];
                        let frame_buffer_index = y as u16 * 256 + x as u16;
                        frame_buffer[(frame_buffer_index * 4 + 0) as usize] = r;
                        frame_buffer[(frame_buffer_index * 4 + 1) as usize] = g;
                        frame_buffer[(frame_buffer_index * 4 + 2) as usize] = b;
                    }
                }
            }
        }
    }
}