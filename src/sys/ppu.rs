pub struct Ppu {
    // pub ppu_ram: [u8],
    // pub ppu_oam: [u8],
    // pub ppu_reg: [u8],
    // pub ppu_chr_rom: [u8],
    // pub ppu_addr_count: [u32], //TODO: 直せたら直す
    // pub ppu_addr: [u32],
    // pub frame_buffer: [i32],
    // pub bg_color_tables: [[u8]],
    // pub sp_color_tables: [[u8]],
    // pub attributeTableCache: [u8], // 各16x16pixelの画面領域で使うパレット
    pub timing: u8 // TODO: 正式なScanline変数にする
    // TODO: PPURAMWrite作る ミラー領域とかの考慮のため
}

impl Ppu {
    pub fn new() -> Ppu {
        Ppu{timing: 0}
    }
}