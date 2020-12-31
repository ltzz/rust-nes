
pub struct Rom {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>
}

const INES_HEADER_SIZE: usize = 0x10;


pub fn from_array(rom: &[u8]) -> Rom {
    // TODO headerチェック
    let prg_rom_size_kb: usize  = (rom[4] as usize) * 16;
    let chr_rom_size_kb: usize  = (rom[5] as usize) * 8;
    let mirroring = rom[6] & 0x01;

    let prg_rom_size= prg_rom_size_kb * 1024;
    let range = INES_HEADER_SIZE .. (INES_HEADER_SIZE + prg_rom_size_kb * 1024);

    // if mirroring > 0 {
    //     // TODO: ちゃんとミラーされるようにコピーをやめる
    //     // TODO: アクセスの際に上位ビット無視すればミラーと同じ挙動？
    //     let tmp_rom = &rom[range];
    //     let mirror_rom: [u8; 32 * 1024] = [0; 32 * 1024];
        
    //     System.arraycopy(tmp_rom, 0, mirror_rom, 0, tmp_rom.len());
        
    //     if tmp_rom.len() < 32 * 1024 && tmp_rom.len() + tmp_rom.len() <= 32 * 1024 {
    //         System.arraycopy(tmp_rom, 0, mirror_rom, tmp_rom.len(), tmp_rom.len());
    //     }
    //     prg_rom = mirror_rom;
    // }
    // else{
    //     let tmp_rom = &rom[range];
    //     prg_rom = tmp_rom;
    // }
    let prg_rom = rom[range].to_vec();

    let chr_rom_start_addr: usize = INES_HEADER_SIZE + prg_rom_size_kb * 1024;
    let chr_rom = rom[chr_rom_start_addr .. (chr_rom_start_addr + chr_rom_size_kb * 1024)].to_vec();
    Rom{prg_rom, chr_rom}
}