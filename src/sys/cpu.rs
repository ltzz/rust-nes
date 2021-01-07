use std::convert::TryInto;

use super::{memory_map::MemoryMap};

pub struct Cpu {
    pub program_counter: u32,
    pub reg_a: u8,
    pub reg_x: u8,
    pub reg_y: u8,
    pub reg_s: u8,
    pub reg_p: u8
}

pub enum Addressing {
    Implied,
    Accumulator,
    Immediate,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Relative,
    Indirect,
    IndirectX,
    Indirect_Y
}

impl Cpu{

    pub fn new() -> Cpu {
        let program_counter: u32 = 0x0000;
        let reg_a: u8 = 0;
        let reg_x: u8 = 0;
        let reg_y: u8 = 0;
        let reg_s: u8 = 0;
        let reg_p: u8 = 0;
        Cpu{program_counter, reg_a, reg_x, reg_y, reg_s, reg_p}
    }

    pub fn init(&mut self){
        self.reg_p = 0x34;
        self.reg_s = 0xFD;
    }
    
    pub fn next_cycle(&mut self, memory_map: &mut MemoryMap){
        self.interpret(memory_map.get_from_address(self.program_counter), memory_map);
    }

    pub fn set_flag_i(&mut self, value: bool){
        self.setP(value, 2);
    }
    pub fn get_flag_i(&mut self) -> bool{
        let interrupt_flag : bool = !((self.reg_p & 0x04) == 0);
        return interrupt_flag;
    }

    pub fn set_flag_b(&mut self, value: bool){
        self.setP(value, 4);
    }

    pub fn set_flag_n(&mut self, value: bool){
        self.setP(value, 7);
    }
    pub fn get_flag_n(&self) -> bool{
        let negative_flag: bool = !((self.reg_p & 0x80) == 0);
        return negative_flag;
    }

    pub fn set_flag_z(&mut self, value: bool){
        self.setP(value, 1);
    }
    pub fn get_flag_z(&self) -> bool{
        let zero_flag: bool = (self.reg_p & 0x02) != 0;
        return zero_flag;
    }

    pub fn set_flag_c(&mut self, value: bool){
        self.setP(value, 0);
    }
    pub fn get_flag_c(&self) -> bool{
        let carry_flag : bool = !((self.reg_p & 0x01) == 0);
        return carry_flag;
    }

    pub fn set_flag_v(&mut self, value: bool){
        self.setP(value, 6);
    }
    pub fn get_flag_v(&self) -> bool{
        let overflow_flag: bool = !((self.reg_p & 0x40) == 0);
        return overflow_flag;
    }

    pub fn setP(&mut self, flag: bool, bitpos: u8){
        if flag {
            self.reg_p |= 1 << bitpos;
        }
        else {
            self.reg_p &= !(1 << bitpos); // FIXME: 移植で壊れてないか要確認
        }
    }

    pub fn getIm16(&self, memory_map: &mut MemoryMap) -> u16{
        return memory_map.get_from_address16(self.program_counter + 1);
    }

    pub fn getIm8(&self, memory_map: &mut MemoryMap) -> u8{
        return memory_map.get_from_address(self.program_counter + 1);
    }
    
    pub fn get_operand_address(&self, addressing: &Addressing, memory_map: &mut MemoryMap) -> u32{
        let address = match addressing{
            Addressing::ZeroPage =>
                (self.getIm8(memory_map) & 0xFF) as u32,
            Addressing::ZeroPageX =>
                (((self.getIm8(memory_map) & 0xFF).wrapping_add(self.reg_x & 0xFF)) & 0xFF) as u32,
            Addressing::ZeroPageY =>
                (((self.getIm8(memory_map) & 0xFF).wrapping_add(self.reg_y & 0xFF)) & 0xFF) as u32,
            Addressing::Absolute =>
                self.getIm16(memory_map) as u32,
            Addressing::AbsoluteX =>
                ((self.getIm16(memory_map).wrapping_add((self.reg_x & 0xFF) as u16)) & 0xFFFF) as u32,
            Addressing::AbsoluteY =>
                ((self.getIm16(memory_map).wrapping_add((self.reg_y & 0xFF) as u16)) & 0xFFFF) as u32,
            Addressing::Indirect => {
                let immediate16 = self.getIm16(memory_map);
                memory_map.get_from_address_in_page(immediate16 as u32) as u32
            },
            Addressing::IndirectX => {
                let tmp_address = (self.getIm8(memory_map)).wrapping_add(self.reg_x);
                memory_map.get_from_address16_by_address8(tmp_address) as u32
            },
            Addressing::Indirect_Y =>{
                let tmp_address = self.getIm8(memory_map) & 0xFF;
                let mut ret_address = memory_map.get_from_address16_by_address8(tmp_address).wrapping_add((self.reg_y & 0xFF) as u16);
                ret_address &= 0xFFFF;
                ret_address as u32
            },
            _ => 0x0000
        };
        return address;
    }

    pub fn get_operand(&self, addressing: &Addressing, memory_map: &mut MemoryMap) -> u8{
        let data: u8 = match addressing{
            Addressing::Immediate =>
                self.getIm8(memory_map),
            Addressing::ZeroPage => {
                let address = self.get_operand_address(&Addressing::ZeroPage, memory_map);
                memory_map.get_from_address(address)
            },
            Addressing::ZeroPageX => {
                let address = self.get_operand_address(&Addressing::ZeroPageX, memory_map);
                memory_map.get_from_address(address)
            },
            Addressing::ZeroPageY => {
                let address = self.get_operand_address(&Addressing::ZeroPageY, memory_map);
                memory_map.get_from_address(address)
            },
            Addressing::Absolute => {
                let address = self.get_operand_address(&Addressing::Absolute, memory_map);
                memory_map.get_from_address(address)
            },
            Addressing::AbsoluteX => {
                let address = self.get_operand_address(&Addressing::AbsoluteX, memory_map);
                memory_map.get_from_address(address)
            },
            Addressing::AbsoluteY => {
                let address = self.get_operand_address(&Addressing::AbsoluteY, memory_map);
                memory_map.get_from_address(address)
            },
            Addressing::Indirect => {
                let address = self.get_operand_address(&Addressing::Indirect, memory_map);
                memory_map.get_from_address(address)
            },
            Addressing::IndirectX => {
                let address = self.get_operand_address(&Addressing::IndirectX, memory_map);
                memory_map.get_from_address(address)
            },
            Addressing::Indirect_Y => {
                let address = self.get_operand_address(&Addressing::Indirect_Y, memory_map);
                memory_map.get_from_address(address)
            },
            _ => 0x00
        };
        return data;
    }
    
    pub fn eval_NZ(&mut self, data: u8){
        if (data & 0xFF) < 128 {
            self.set_flag_n(false);
        }else{
            self.set_flag_n(true);
        }
        if data == 0 {
            self.set_flag_z(true);
        }
        else {
            self.set_flag_z(false);
        }
    }

    pub fn op_txs(&mut self){
        self.reg_s = self.reg_x;
    }
    pub fn op_tsx(&mut self){
        self.eval_NZ(self.reg_s);
        self.reg_x = self.reg_s;
    }
    pub fn op_tax(&mut self){
        self.eval_NZ(self.reg_a);
        self.reg_x = self.reg_a;
    }
    pub fn op_txa(&mut self){
        self.eval_NZ(self.reg_x);
        self.reg_a = self.reg_x;
    }
    pub fn op_tya(&mut self){
        self.eval_NZ(self.reg_y);
        self.reg_a = self.reg_y;
    }
    pub fn op_tay(&mut self){
        self.eval_NZ(self.reg_a);
        self.reg_y = self.reg_a;
    }

    pub fn op_cpx(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let value:u8 = self.get_operand(addressing, memory_map);
        self.set_reg_at_compare(self.reg_x, value);
    }

    pub fn op_cpy(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let value:u8 = self.get_operand(addressing, memory_map);
        self.set_reg_at_compare(self.reg_y, value);
    }

    pub fn op_cmp(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let value:u8 = self.get_operand(addressing, memory_map);
        self.set_reg_at_compare(self.reg_a, value);
    }

    pub fn op_dcm(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let value:u8 = self.get_operand(addressing, memory_map);
        let result_value:u8 = value.wrapping_sub(1);
        let address = self.get_operand_address(addressing, memory_map);
        memory_map.set_from_address(address, result_value);
        self.set_reg_at_compare(self.reg_a, result_value);
    }

    pub fn op_isc(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let value: u8 = self.get_operand(addressing, memory_map);
        let result_value: u8 = value.wrapping_add(1);
        let address = self.get_operand_address(addressing, memory_map);
        memory_map.set_from_address(address, result_value);
        self.op_sbc_impl(result_value);
    }


    pub fn set_reg_at_compare(&mut self, reg: u8, value: u8){
        let reg_value = reg & 0xFF;
        let target_value = value & 0xFF;
        if reg_value >= target_value {
            self.set_flag_c(true);
        }else{
            self.set_flag_c(false);
        }
        if reg_value == target_value {
            self.set_flag_z(true);
        }else{
            self.set_flag_z(false);
        }
        if ((reg_value.wrapping_sub(target_value)) & 0x80) > 0 {
            self.set_flag_n(true);
        }else{
            self.set_flag_n(false);
        }
    }

    pub fn op_bit(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let address = self.get_operand_address(addressing, memory_map);
        let value = memory_map.get_from_address(address);
        if (value & 0x80) > 0 {
            self.set_flag_n(true);
        }
        else{
            self.set_flag_n(false);
        }
        if (value & 0x40) > 0 {
            self.set_flag_v(true);
        }
        else{
            self.set_flag_v(false);
        }
        if (self.reg_a & value) == 0 { // TODO: ロジック確認してないので要確認
            self.set_flag_z(true);
        }
        else{
            self.set_flag_z(false);
        }
    }


    pub fn op_and(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap) {
        let value = self.get_operand(addressing, memory_map);
        let result_value = self.reg_a & value;
        self.reg_a = result_value;
        self.eval_NZ(self.reg_a);
    }

    pub fn op_eor(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap) {
        let value = self.get_operand(addressing, memory_map);
        let result_value = self.reg_a ^ value;
        self.reg_a = result_value;
        self.eval_NZ(self.reg_a);
    }

    pub fn op_ora(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap) {
        let value = self.get_operand(addressing, memory_map);
        let result_value = self.reg_a | value;
        self.reg_a = result_value;
        self.eval_NZ(self.reg_a);
    }


    pub fn op_adc(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap) {
        let value = self.get_operand(addressing, memory_map);
        let carry = self.reg_p & 0x01;
        let result_value: u16 = (self.reg_a & 0xFF) as u16 + (value & 0xFF) as u16 + carry as u16;
        let reg_a_old = self.reg_a;
        self.reg_a = (result_value & 0xFF).try_into().unwrap();
        self.eval_NZ(self.reg_a);
        if result_value >= 0x100 {
            self.set_flag_c(true);
        }
        else{
            self.set_flag_c(false);
        }
        let result_value_byte: u8 = (result_value & 0xFF).try_into().unwrap();
        if ((result_value_byte ^ value) & (result_value_byte ^ reg_a_old) & 0x80) !=0 {
            self.set_flag_v(true);
        }
        else {
            self.set_flag_v(false);
        }
    }

    pub fn op_sbc(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap) {
        let value = self.get_operand(addressing, memory_map);
        self.op_sbc_impl(value);
    }

    pub fn op_sbc_impl(&mut self, value: u8){
        let carry = self.reg_p & 0x01;
        let not_carry: u8 = if carry > 0 {0} else {1};
        let result_value: i32 = (self.reg_a & 0xFF) as i32 - value as  i32 - not_carry as i32;
        let reg_a_old = self.reg_a;
        self.reg_a = (result_value & 0xFF) as u8;
        self.eval_NZ(self.reg_a);
        if result_value < 0 { // TODO: ロジック確認してないので要確認
            self.set_flag_c(false);
        }
        else{
            self.set_flag_c(true);
        }

        let result_value_u8 = (result_value & 0xFF) as u8;
        let borrowed_value: u8 = (((value ^ 0xFF) as u16 + 0x100 as u16) & 0xFF).try_into().unwrap();
        if ((result_value_u8 ^ borrowed_value) & (result_value_u8 ^ reg_a_old) & 0x80) > 0 {
            self.set_flag_v(true);
        }
        else {
            self.set_flag_v(false);
        }
    }

    pub fn op_lsr(&mut self) {
        self.reg_a = self.op_lsr_impl(self.reg_a);
    }

    pub fn op_lsr_with_addressing(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let mut data: u8 = self.get_operand(addressing, memory_map);
        data = self.op_lsr_impl(data);
        let address = self.get_operand_address(addressing, memory_map);
        memory_map.set_from_address(address, data);
    }

    pub fn op_lsr_impl(&mut self, mut data: u8) -> u8{
        let carry: u8 = data & 0x01;
        let result_value: u8 = (data & 0xFF) >> 1;
        data = result_value as u8;
        self.eval_NZ(data);
        if carry > 0 { // TODO: ロジック確認してないので要確認
            self.set_flag_c(true);
        }
        else{
            self.set_flag_c(false);
        }
        return data;
    }

    pub fn op_ror(&mut self) {
        self.reg_a = self.op_ror_impl(self.reg_a);
    }

    pub fn op_ror_with_addressing(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let mut data: u8 = self.get_operand(addressing, memory_map);
        data = self.op_ror_impl(data);
        let address = self.get_operand_address(addressing, memory_map);
        memory_map.set_from_address(address, data);
    }

    pub fn op_ror_impl(&mut self, mut data: u8) -> u8{
        let carry = if self.get_flag_c() {1} else {0};
        let output_carry = data & 0x01;
        let result_value: u8 = (data & 0xFF) >> 1;
        data = result_value as u8;
        data |= carry << 7;
        self.eval_NZ(data);
        if output_carry > 0 { // TODO: ロジック確認してないので要確認
            self.set_flag_c(true);
        }
        else{
            self.set_flag_c(false);
        }
        data
    }

    pub fn op_rol(&mut self) {
        self.reg_a = self.op_rol_impl(self.reg_a);
    }
    pub fn op_rol_with_addressing(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap) {
        let mut data: u8 = self.get_operand(addressing, memory_map);
        data = self.op_rol_impl(data);
        let address = self.get_operand_address(addressing, memory_map);
        memory_map.set_from_address(address, data);
    }

    pub fn op_rol_impl(&mut self, mut data: u8) -> u8{
        let output_carry: u8 = data & 0x80;
        let result_value: u16 = ((data & 0xFF) << 1) as u16;
        data = result_value as u8;
        data |= if self.get_flag_c() {0x01} else {0x00};
        self.eval_NZ(data);
        if output_carry > 0 { // TODO: ロジック確認してないので要確認
            self.set_flag_c(true);
        }
        else{
            self.set_flag_c(false);
        }
        return data;
    }

    pub fn op_asl(&mut self) {
        let result_value: u16 = (((self.reg_a & 0xFF) as u16) << 1) as u16;
        self.reg_a = (result_value) as u8;
        self.eval_NZ(self.reg_a);
        if result_value >= 0x100 { // TODO: ロジック確認してないので要確認
            self.set_flag_c(true);
        }
        else{
            self.set_flag_c(false);
        }
    }

    pub fn op_asl_with_addressing(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap) {
        let address: u32 = self.get_operand_address(addressing, memory_map);
        let value: u8 = memory_map.get_from_address(address);
        let result_value: u16 = (((value & 0xFF) as u16) << 1) as u16;
        let res_value8 = (result_value) as u8;
        memory_map.set_from_address(address, res_value8);
        self.eval_NZ(res_value8);
        if result_value >= 0x100 { // TODO: ロジック確認してないので要確認
            self.set_flag_c(true);
        }
        else{
            self.set_flag_c(false);
        }
    }

    
    pub fn op_aso_with_addressing(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap) {
        self.op_asl_with_addressing(addressing, memory_map);
        self.op_ora(addressing, memory_map);
    }

    pub fn op_rla_with_addressing(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap) {
        self.op_rol_with_addressing(addressing, memory_map);
        self.op_and(addressing, memory_map);
    }

    pub fn op_lse_with_addressing(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap) {
        self.op_lsr_with_addressing(addressing, memory_map);
        self.op_eor(addressing, memory_map);
    }

    pub fn op_rra_with_addressing(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap) {
        self.op_ror_with_addressing(addressing, memory_map);
        self.op_adc(addressing, memory_map);
    }

    pub fn op_inx(&mut self){
        self.reg_x = (self.reg_x.wrapping_add(1)) as u8;
        self.eval_NZ(self.reg_x);
    }

    pub fn op_inc(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let address: u32 = self.get_operand_address(addressing, memory_map);
        let mut value: u8 = memory_map.get_from_address(address);
        value = (value.wrapping_add(1)) as u8;
        memory_map.set_from_address(address, value);
        self.eval_NZ(value);
    }

    pub fn op_dec(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let address: u32 = self.get_operand_address(addressing, memory_map);
        let mut value: u8 = memory_map.get_from_address(address);
        value = (value.wrapping_sub(1)) as u8;
        memory_map.set_from_address(address, value);
        self.eval_NZ(value);
    }

    pub fn op_iny(&mut self){
        self.reg_y = (self.reg_y.wrapping_add(1)) as u8;
        self.eval_NZ(self.reg_y);
    }

    pub fn op_dex(&mut self){
        self.reg_x = (self.reg_x.wrapping_sub(1)) as u8;
        self.eval_NZ(self.reg_x);
    }

    pub fn op_dey(&mut self){
        self.reg_y = (self.reg_y.wrapping_sub(1)) as u8;
        self.eval_NZ(self.reg_y);
    }

    pub fn op_clc(&mut self){
        self.reg_p = (self.reg_p & 0xFE) as u8;
    }
    pub fn op_cld(&mut self){
        self.reg_p = (self.reg_p & 0xF7) as u8;
    }
    pub fn op_clv(&mut self){
        self.reg_p = (self.reg_p & 0xBF) as u8;
    }
    pub fn op_sec(&mut self){
        self.reg_p = (self.reg_p | 0x01) as u8;
    }

    pub fn op_sed(&mut self){
        self.reg_p = (self.reg_p | 0x08) as u8;
    }

    pub fn op_sta(&self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let address = self.get_operand_address(addressing, memory_map);
        memory_map.set_from_address(address, self.reg_a);
    }

    pub fn op_stx(&self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let address = self.get_operand_address(addressing, memory_map);
        memory_map.set_from_address(address, self.reg_x);
    }

    pub fn op_sty(&self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let address = self.get_operand_address(addressing, memory_map);
        memory_map.set_from_address(address, self.reg_y);
    }
    pub fn op_sax(&self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let address = self.get_operand_address(addressing, memory_map);
        memory_map.set_from_address(address, (self.reg_a & self.reg_x) as u8);
    }

    pub fn op_lda(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let operand: u8 = self.get_operand(addressing, memory_map);
        self.eval_NZ(operand);
        self.reg_a = operand;
    }
    pub fn op_ldx(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let operand: u8 = self.get_operand(addressing, memory_map);
        self.eval_NZ(operand);
        self.reg_x = operand;
    }
    pub fn op_ldy(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let operand: u8 = self.get_operand(addressing, memory_map);
        self.eval_NZ(operand);
        self.reg_y = operand;
    }
    pub fn op_lax(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let operand: u8 = self.get_operand(addressing, memory_map);
        self.eval_NZ(operand);
        self.reg_a = operand;
        self.reg_x = operand;
    }

    pub fn op_bne(&mut self, memory_map: &mut MemoryMap){
        let zero_flag: bool = self.get_flag_z();
        if !zero_flag {
            let relative: u8 = self.getIm8(memory_map);
            let relative_ = if relative > 0x80 {relative as i16 - 0x100 as i16} else {relative as i16};
            self.program_counter = (self.program_counter as i32 + relative_ as i32) as u32;
        }
    }
    pub fn op_bpl(&mut self, memory_map: &mut MemoryMap){
        let negative_flag: bool = self.get_flag_n();
        if !negative_flag {
            let relative: u8 = self.getIm8(memory_map);
            let relative_ = if relative > 0x80 {relative as i16 - 0x100 as i16} else {relative as i16};
            self.program_counter = (self.program_counter as i32 + relative_ as i32) as u32;
        }
    }
    pub fn op_bcc(&mut self, memory_map: &mut MemoryMap){
        let carry_flag: bool = self.get_flag_c();
        if !carry_flag {
            let relative: u8 = self.getIm8(memory_map);
            let relative_ = if relative > 0x80 {relative as i16 - 0x100 as i16} else {relative as i16};
            self.program_counter = (self.program_counter as i32 + relative_ as i32) as u32;
        }
    }
    pub fn op_bcs(&mut self, memory_map: &mut MemoryMap){
        let carry_flag: bool = self.get_flag_c();
        if carry_flag {
            let relative: u8 = self.getIm8(memory_map);
            let relative_ = if relative > 0x80 {relative as i16 - 0x100 as i16} else {relative as i16};
            self.program_counter = (self.program_counter as i32 + relative_ as i32) as u32;
        }
    }
    pub fn op_bvs(&mut self, memory_map: &mut MemoryMap){
        let overflow_flag: bool = self.get_flag_v();
        if overflow_flag {
            let relative: u8 = self.getIm8(memory_map);
            let relative_ = if relative > 0x80 {relative as i16 - 0x100 as i16} else {relative as i16};
            self.program_counter = (self.program_counter as i32 + relative_ as i32) as u32;
        }
    }
    pub fn op_bvc(&mut self, memory_map: &mut MemoryMap){
        let overflow_flag: bool = self.get_flag_v();
        if !overflow_flag {
            let relative: u8 = self.getIm8(memory_map);
            let relative_ = if relative > 0x80 {relative as i16 - 0x100 as i16} else {relative as i16};
            self.program_counter = (self.program_counter as i32 + relative_ as i32) as u32;
        }
    }
    pub fn op_bmi(&mut self, memory_map: &mut MemoryMap){
        let negative_flag: bool = self.get_flag_n();
        if negative_flag {
            let relative: u8 = self.getIm8(memory_map);
            let relative_ = if relative > 0x80 {relative as i16 - 0x100 as i16} else {relative as i16};
            self.program_counter = (self.program_counter as i32 + relative_ as i32) as u32;
        }
    }
    pub fn op_beq(&mut self, memory_map: &mut MemoryMap){
        let zero_flag: bool = self.get_flag_z();
        if zero_flag {
            let relative: u8 = self.getIm8(memory_map);
            let relative_ = if relative > 0x80 {relative as i16 - 0x100 as i16} else {relative as i16};
            self.program_counter = (self.program_counter as i32 + relative_ as i32) as u32;
        }
    }

    pub fn op_jsr(&mut self, memory_map: &mut MemoryMap){
        let absolute: u16 = self.getIm16(memory_map);
        let return_address = self.program_counter + 2; // この命令の最後のアドレスをpush
        let upper: u8 = ((return_address >> 8) & 0xFF) as u8;
        let lower: u8 = (return_address & 0xFF) as u8;
        let stack_address: u32 = 0x100 + (self.reg_s & 0xFF) as u32;
        memory_map.set_from_address(stack_address, upper);
        memory_map.set_from_address(stack_address- 1, lower);
        self.reg_s = (self.reg_s - 2) as u8;
        self.program_counter = absolute as u32;
    }

    pub fn op_pha(&mut self, memory_map: &mut MemoryMap){
        let stack_address: u32 = (0x100 as u16 + (self.reg_s & 0xFF) as u16) as u32;
        memory_map.set_from_address(stack_address, self.reg_a);
        self.reg_s = (self.reg_s - 1) as u8;
    }

    pub fn op_php(&mut self, memory_map: &mut MemoryMap){
        let stack_address: u32 = (0x100 as u16 + (self.reg_s & 0xFF) as u16) as u32;
        let value = self.reg_p | 0x10; // ファミコンの仕様 PHPによってスタックに格納する状態フラグでは、ブレイクフラグをセット
        memory_map.set_from_address(stack_address, value);
        self.reg_s = (self.reg_s - 1) as u8;
    }
    pub fn op_plp(&mut self, memory_map: &mut MemoryMap){
        let stack_address: u32 = (0x100 as u16 + (self.reg_s & 0xFF) as u16 + 1) as u32;
        let mut value = memory_map.get_from_address(stack_address);
        self.set_reg_p(value);
        self.reg_s = (self.reg_s + 1) as u8;
    }

    pub fn op_rts(&mut self, memory_map: &mut MemoryMap){
        let stack_address: u32 = (0x100 as u16 + (self.reg_s & 0xFF) as u16 + 1) as u32;
        let lower: u8 = memory_map.get_from_address(stack_address);
        let upper: u8 = memory_map.get_from_address(stack_address + 1);
        self.program_counter = ((((upper & 0xFF) as u16) << 8) | (lower & 0xFF) as u16) as u32;
        //programCounter = (upper << 8) | lower;
        self.reg_s = (self.reg_s + 2) as u8;
    }

    pub fn op_rti(&mut self, memory_map: &mut MemoryMap) {

        // Pをpull
        let stack_address_p: u32 = (0x100 as u16 + (self.reg_s & 0xFF) as u16 + 1) as u32;
        let value = memory_map.get_from_address(stack_address_p);
        self.set_reg_p(value);
        self.reg_s = (self.reg_s + 1) as u8;

        // プログラムカウンタをpull
        let stack_address: u32 = (0x100 as u16 + (self.reg_s & 0xFF) as u16 + 1) as u32;
        let lower: u8 = memory_map.get_from_address(stack_address);
        let upper: u8 = memory_map.get_from_address(stack_address + 1);
        self.program_counter = ((((upper & 0xFF) as u16) << 8) | (lower & 0xFF) as u16) as u32;
        //programCounter = (upper << 8) | lower;
        self.reg_s = (self.reg_s + 2) as u8;

    }

    pub fn op_pla(&mut self, memory_map: &mut MemoryMap){
        let address: u32 = (0x100 as u16 + (self.reg_s & 0xFF) as u16 + 1) as u32;
        let value: u8 = memory_map.get_from_address(address);
        self.reg_a = value;
        self.eval_NZ(self.reg_a);
        self.reg_s = (self.reg_s + 1) as u8;
    }

    pub fn op_brk(&mut self){
        self.set_flag_i(true);
        self.set_flag_b(true);
    }

    pub fn opJMP_Abs(&mut self, memory_map: &mut MemoryMap){
        let absolute : u16 = self.getIm16(memory_map);
        self.program_counter = absolute as u32;
    }
    pub fn opJMP_Indirect(&mut self, memory_map: &mut MemoryMap){
        let address: u32 = self.get_operand_address(&Addressing::Indirect, memory_map);
        self.program_counter = address;
    }

    fn set_reg_p(&mut self, value: u8){
        let value = value & 0xEF | 0x20;
        // bit4: ブレイクフラグは実際には存在しないためPへのセット時クリア
        // bit5: Rフラグはは常にセット
        self.reg_p = value;
    }

    pub fn interpret(&mut self, opcode: u8, memory_map: &mut MemoryMap){

        let opcode: u8 = opcode & 0xFF;
        match(opcode){
            0xA2 =>//LDX(Immediate):メモリからXにロード(2バイト/2サイクル)
            {
                self.op_ldx(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0xA6 =>//LDX(Zeropage):メモリからXにロード(2バイト/3サイクル)
            {
                self.op_ldx(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xB6 =>//LDX(Zeropage,Y):メモリからXにロード(2バイト/4サイクル)
            {
                self.op_ldx(&Addressing::ZeroPageY, memory_map);
                self.program_counter += 2;
            },
            0xAE =>//LDX(Absolute):メモリからXにロード(3バイト/4サイクル)
            {
                self.op_ldx(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            }
            0xBE =>//LDX(Absolute, Y):メモリからXにロード(3バイト/4サイクル)
            {
                self.op_ldx(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            }
            0x78 =>//SEI:IRQ割り込みの禁止(1バイト/2サイクル)
            {
                self.set_flag_i(true);
                self.program_counter += 1;
            },
            0x58 =>//CLI:IRQ割り込みの許可(1バイト/2サイクル)
            {
                self.set_flag_i(false);
                self.program_counter += 1;
            },
            0xA9 =>//LDA(Immediate):メモリからAにロード(2バイト/2サイクル)
            {
                self.op_lda(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0xA5 =>//LDA(Zeropage):メモリからAにロード(2バイト/3サイクル)
            {
                self.op_lda(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xB5 =>//LDA(ZeropageX):メモリからAにロード(2バイト/4サイクル)
            {
                self.op_lda(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0xAD =>//LDA(Absolute):メモリからAにロード(3バイト/4サイクル)
            {
                self.op_lda(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xBD =>//LDA(AbsoluteX):メモリからAにロード(3バイト/4サイクル)
            {
                self.op_lda(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0xB9 =>//LDA(AbsoluteY):メモリからAにロード(3バイト/4サイクル)
            {
                self.op_lda(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0xA1 =>//LDA(IndirectX):メモリからAにロード(2バイト/6サイクル)
            {
                self.op_lda(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0xB1 =>//LDA(Indirect_Y):メモリからAにロード(2バイト/5サイクル)
            {
                self.op_lda(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0xA0 =>//LDY(Immediate):メモリからYにロード(2バイト/2サイクル)
            {
                self.op_ldy(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0xA4 =>//LDY(ZeroPage):メモリからYにロード(2バイト/3サイクル)
            {
                self.op_ldy(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xB4 =>//LDY(ZeroPageX):メモリからYにロード(2バイト/4サイクル)
            {
                self.op_ldy(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0xAC =>//LDY(Absolute):メモリからAにロード(3バイト/4サイクル)
            {
                self.op_ldy(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xBC =>//LDY(Absolute, X):メモリからAにロード(3バイト/4サイクル)
            {
                self.op_ldy(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0xA7 => // LAX ※拡張命令
            {
                self.op_lax(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xB7 => // LAX ※拡張命令
            {
                self.op_lax(&Addressing::ZeroPageY, memory_map);
                self.program_counter += 2;
            },
            0xAF => // LAX ※拡張命令
            {
                self.op_lax(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xBF => // LAX ※拡張命令
            {
                self.op_lax(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0xA3 => // LAX ※拡張命令
            {
                self.op_lax(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0xB3 => // LAX ※拡張命令
            {
                self.op_lax(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x85 =>//STA(Zeropage):Aからメモリにストア(2バイト/3サイクル)
            {
                self.op_sta(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x95 =>//STA(ZeropageX):Aからメモリにストア(2バイト/4サイクル)
            {
                self.op_sta(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x8D =>//STA(Absolute):Aからメモリにストア(3バイト/4サイクル)
            {
                self.op_sta(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x9D =>//STA(AbsoluteX):Aからメモリにストア(3バイト/5サイクル)
            {
                self.op_sta(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x99 =>//STA(AbsoluteY):Aからメモリにストア(3バイト/5サイクル)
            {
                self.op_sta(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0x91 =>//STA(Indirect_Y):Aからメモリにストア(2バイト/6サイクル)
            {
                self.op_sta(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x81 =>//STA(Indirect,X):Aからメモリにストア(2バイト/6サイクル)
            {
                self.op_sta(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0x86 =>//STX(Zeropage):Xからメモリにストア(2バイト/3サイクル)
            {
                self.op_stx(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x96 =>//STX(Zeropage,Y):Xからメモリにストア(2バイト/4サイクル)
            {
                self.op_stx(&Addressing::ZeroPageY, memory_map);
                self.program_counter += 2;
            },
            0x8E =>//STX(Absolute):Xからメモリにストア(3バイト/4サイクル)
            {
                self.op_stx(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x8C =>//STY(Absolute):Yからメモリにストア(3バイト/4サイクル)
            {
                self.op_sty(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x84 =>//STY(Zeropage):Yからメモリにストア(2バイト/3サイクル)
            {
                self.op_sty(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x94 =>//STY(ZeropageX):Yからメモリにストア(2バイト/4サイクル)
            {
                self.op_sty(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x87 =>//SAX ※拡張命令
            {
                self.op_sax(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x97 =>//SAX ※拡張命令
            {
                self.op_sax(&Addressing::ZeroPageY, memory_map);
                self.program_counter += 2;
            },
            0x8F =>//SAX ※拡張命令
            {
                self.op_sax(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x83 =>//SAX ※拡張命令
            {
                self.op_sax(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0x9A =>
                // TODO: Sに0を入れているROMがあり、うまく動作しない（あるいは入れる元の計算結果が誤り
            {
                self.op_txs();
                self.program_counter += 1;
            },
            0xBA =>
            {
                self.op_tsx();
                self.program_counter += 1;
            },
            0xAA =>
            {
                self.op_tax();
                self.program_counter += 1;
            },
            0x8A =>
            {
                self.op_txa();
                self.program_counter += 1;
            },
            0x98 =>
            {
                self.op_tya();
                self.program_counter += 1;
            },
            0xA8 =>
            {
                self.op_tay();
                self.program_counter += 1;
            },
            0xC0 =>
            {
                self.op_cpy(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0xC4 =>
            {
                self.op_cpy(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xCC =>
            {
                self.op_cpy(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xE0 =>
            {
                self.op_cpx(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0xE4 =>
            {
                self.op_cpx(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xEC =>
            {
                self.op_cpx(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xC5 =>
            {
                self.op_cmp(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xD5 =>
            {
                self.op_cmp(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0xC9 =>
            {
                self.op_cmp(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0xCD =>
            {
                self.op_cmp(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xDD =>
            {
                self.op_cmp(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0xD9 =>
            {
                self.op_cmp(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0xC1 =>
            {
                self.op_cmp(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0xD1 =>
            {
                self.op_cmp(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x2C =>
            {
                self.op_bit(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x24 =>
            {
                self.op_bit(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x29 =>
            {
                self.op_and(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0x25 =>
            {
                self.op_and(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x35 =>
            {
                self.op_and(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x2D =>
            {
                self.op_and(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x3D =>
            {
                self.op_and(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x39 =>
            {
                self.op_and(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0x21 =>
            {
                self.op_and(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0x31 =>
            {
                self.op_and(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x49 =>
            {
                self.op_eor(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0x45 =>
            {
                self.op_eor(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x55 =>
            {
                self.op_eor(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x4D =>
            {
                self.op_eor(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x5D =>
            {
                self.op_eor(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x59 =>
            {
                self.op_eor(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0x41 =>
            {
                self.op_eor(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0x51 =>
            {
                self.op_eor(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x09 =>
            {
                self.op_ora(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0x05 =>
            {
                self.op_ora(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x15 =>
            {
                self.op_ora(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x0D =>
            {
                self.op_ora(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x1D =>
            {
                self.op_ora(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x19 =>
            {
                self.op_ora(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0x01 =>
            {
                self.op_ora(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0x11 =>
            {
                self.op_ora(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x69 =>
            {
                self.op_adc(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0x61 =>
            {
                self.op_adc(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0x71 =>
            {
                self.op_adc(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x65 =>
            {
                self.op_adc(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x75 =>
            {
                self.op_adc(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x6D =>
            {
                self.op_adc(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x7D =>
            {
                self.op_adc(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x79 =>
            {
                self.op_adc(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0xE9 =>
            {
                self.op_sbc(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0xE5 =>
            {
                self.op_sbc(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xF5 =>
            {
                self.op_sbc(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0xED =>
            {
                self.op_sbc(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xFD =>
            {
                self.op_sbc(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0xF9 =>
            {
                self.op_sbc(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0xE1 =>
            {
                self.op_sbc(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0xF1 =>
            {
                self.op_sbc(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x06 =>
            {
                self.op_asl_with_addressing(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x16 =>
            {
                self.op_asl_with_addressing(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x0E =>
            {
                self.op_asl_with_addressing(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x1E =>
            {
                self.op_asl_with_addressing(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x0A =>
            {
                self.op_asl();
                self.program_counter += 1;
            },
            0x4A =>
            {
                self.op_lsr();
                self.program_counter += 1;
            },
            0x46 =>
            {
                self.op_lsr_with_addressing(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x56 =>
            {
                self.op_lsr_with_addressing(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x4E =>
            {
                self.op_lsr_with_addressing(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x5E =>
            {
                self.op_lsr_with_addressing(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x6A =>
            {
                self.op_ror();
                self.program_counter += 1;
            },
            0x66 =>
            {
                self.op_ror_with_addressing(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x76 =>
            {
                self.op_ror_with_addressing(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x6E =>
            {
                self.op_ror_with_addressing(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x7E =>
            {
                self.op_ror_with_addressing(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x2A =>
            {
                self.op_rol();
                self.program_counter += 1;
            },
            0x26 =>
            {
                self.op_rol_with_addressing(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x36 =>
            {
                self.op_rol_with_addressing(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x2E =>
            {
                self.op_rol_with_addressing(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x3E =>
            {
                self.op_rol_with_addressing(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0xE8 =>
            {
                self.op_inx();
                self.program_counter += 1;
            },
            0xC8 =>
            {
                self.op_iny();
                self.program_counter += 1;
            },
            0xE6 => // (2バイト/5サイクル)
            {
                self.op_inc(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xF6 => // (2バイト/6サイクル)
            {
                self.op_inc(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0xEE => // (3バイト/6サイクル)
            {
                self.op_inc(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xFE => // (3バイト/7サイクル)
            {
                self.op_inc(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0xCA =>
            {
                self.op_dex();
                self.program_counter += 1;
            },
            0x88 =>
            {
                self.op_dey();
                self.program_counter += 1;
            },
            0xC6 =>
            {
                self.op_dec(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xD6 =>
            {
                self.op_dec(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0xCE =>
            {
                self.op_dec(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xDE =>
            {
                self.op_dec(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0xD0 =>
            {
                self.op_bne(memory_map);
                self.program_counter += 2;
            },
            0x10 =>
            {
                self.op_bpl(memory_map);
                self.program_counter += 2;
            },
            0x90 =>
            {
                self.op_bcc(memory_map);
                self.program_counter += 2;
            },
            0xB0 =>
            {
                self.op_bcs(memory_map);
                self.program_counter += 2;
            },
            0x70 =>
            {
                self.op_bvs(memory_map);
                self.program_counter += 2;
            },
            0x50 =>
            {
                self.op_bvc(memory_map);
                self.program_counter += 2;
            },
            0x30 =>
            {
                self.op_bmi(memory_map);
                self.program_counter += 2;
            },
            0xF0 =>
            {
                self.op_beq(memory_map);
                self.program_counter += 2;
            },
            0x20 =>
            {
                self.op_jsr(memory_map);
            },
            0x48 =>
            {
                self.op_pha(memory_map);
                self.program_counter += 1;
            },
            0x08 =>
            {
                self.op_php(memory_map);
                self.program_counter += 1;
            },
            0x28 =>
            {
                self.op_plp(memory_map);
                self.program_counter += 1;
            },
            0x68 =>
            {
                self.op_pla(memory_map);
                self.program_counter += 1;
            },
            0x60 =>
            {
                self.op_rts(memory_map);
                self.program_counter += 1;
            },
            0x40 =>
            {
                self.op_rti(memory_map);
                //self.program_counter += 1;
            },
            0x4C =>
            {
                self.opJMP_Abs(memory_map);
                //self.program_counter += 3;
            },// FIXME: pcインクリメントしないといかん気がする→確認
            0x6C =>
            {
                self.opJMP_Indirect(memory_map);
                //self.program_counter += 3;
            },// FIXME: pcインクリメントしないといかん気がする→確認
            0x38 =>
            {
                self.op_sec();
                self.program_counter += 1;
            },
            0xF8 => // SED ファミコン用6502ではフラグ変更のみ
            {
                self.op_sed();
                self.program_counter += 1;
            },
            0x18 =>
            {
                self.op_clc();
                self.program_counter += 1;
            },
            0xD8 => // CLD ファミコン用6502ではフラグ変更のみ
            {
                self.op_cld();
                self.program_counter += 1;
            },
            0xB8 =>
            {
                self.op_clv();
                self.program_counter += 1;
            },
            0xEB => // SBC ※拡張命令
            {
                self.op_sbc(&Addressing::Immediate, memory_map);
                self.program_counter+= 2;
            },
            0xC7 => // DCM(DCP) ※拡張命令
            {
                self.op_dcm(&Addressing::ZeroPage, memory_map);
                self.program_counter+= 2;
            },
            0xD7 => // DCM(DCP) ※拡張命令
            {
                self.op_dcm(&Addressing::ZeroPageX, memory_map);
                self.program_counter+= 2;
            },
            0xCF => // DCM(DCP) ※拡張命令
            {
                self.op_dcm(&Addressing::Absolute, memory_map);
                self.program_counter+= 3;
            },
            0xDF => // DCM(DCP) ※拡張命令
            {
                self.op_dcm(&Addressing::AbsoluteX, memory_map);
                self.program_counter+= 3;
            },
            0xDB => // DCM(DCP) ※拡張命令
            {
                self.op_dcm(&Addressing::AbsoluteY, memory_map);
                self.program_counter+= 3;
            },
            0xC3 => // DCM(DCP) ※拡張命令
            {
                self.op_dcm(&Addressing::IndirectX, memory_map);
                self.program_counter+= 2;
            },
            0xD3 => // DCM(DCP) ※拡張命令
            {
                self.op_dcm(&Addressing::Indirect_Y, memory_map);
                self.program_counter+= 2;
            },
            0xE7 => // ISC ※拡張命令
            {
                self.op_isc(&Addressing::ZeroPage, memory_map);
                self.program_counter+= 2;
            },
            0xF7 => // ISC(ISB) ※拡張命令
            {
                self.op_isc(&Addressing::ZeroPageX, memory_map);
                self.program_counter+= 2;
            },
            0xEF => // ISC(ISB) ※拡張命令
            {
                self.op_isc(&Addressing::Absolute, memory_map);
                self.program_counter+= 3;
            },
            0xFF => // ISC(ISB) ※拡張命令
            {
                self.op_isc(&Addressing::AbsoluteX, memory_map);
                self.program_counter+= 3;
            },
            0xFB => // ISC(ISB) ※拡張命令
            {
                self.op_isc(&Addressing::AbsoluteY, memory_map);
                self.program_counter+= 3;
            },
            0xE3 => // ISC(ISB) ※拡張命令
            {
                self.op_isc(&Addressing::IndirectX, memory_map);
                self.program_counter+= 2;
            },
            0xF3 => // ISC(ISB) ※拡張命令
            {
                self.op_isc(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0xEA =>
            {
                // NOP
                self.program_counter += 1;
            },
            0x03 => {
                // ASO/SLO 本来 未定義命令
                // memory = shift left memory, A = A OR memory
                self.op_aso_with_addressing(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0x07 => {
                // ASO/SLO 本来 未定義命令
                // memory = shift left memory, A = A OR memory
                self.op_aso_with_addressing(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x0F => {
                // ASO/SLO 本来 未定義命令
                // memory = shift left memory, A = A OR memory
                self.op_aso_with_addressing(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x13 => {
                // ASO/SLO 本来 未定義命令
                // memory = shift left memory, A = A OR memory
                self.op_aso_with_addressing(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x17 => {
                // ASO/SLO 本来 未定義命令
                // memory = shift left memory, A = A OR memory
                self.op_aso_with_addressing(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x1B => {
                // ASO/SLO 本来 未定義命令
                // memory = shift left memory, A = A OR memory
                self.op_aso_with_addressing(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0x1F => {
                // ASO/SLO 本来 未定義命令
                // memory = shift left memory, A = A OR memory
                self.op_aso_with_addressing(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x23 => {
                // RLA 本来 未定義命令
                // memory = rotate left memory, A = A AND memory
                self.op_rla_with_addressing(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0x27 => {
                // RLA 本来 未定義命令
                // memory = rotate left memory, A = A AND memory
                self.op_rla_with_addressing(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x2F => {
                // RLA 本来 未定義命令
                // memory = rotate left memory, A = A AND memory
                self.op_rla_with_addressing(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x33 => {
                // RLA 本来 未定義命令
                // memory = rotate left memory, A = A AND memory
                self.op_rla_with_addressing(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x37 => {
                // RLA 本来 未定義命令
                // memory = rotate left memory, A = A AND memory
                self.op_rla_with_addressing(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x3B => {
                // RLA 本来 未定義命令
                // memory = rotate left memory, A = A AND memory
                self.op_rla_with_addressing(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0x3F => {
                // RLA 本来 未定義命令
                // memory = rotate left memory, A = A AND memory
                self.op_rla_with_addressing(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x43 => {
                // SRE/LSE 本来 未定義命令
                // memory = shift right memory, A = A EOR memory
                self.op_lse_with_addressing(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0x47 => {
                // SRE/LSE 本来 未定義命令
                // memory = shift right memory, A = A EOR memory
                self.op_lse_with_addressing(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x4F => {
                // SRE/LSE 本来 未定義命令
                // memory = shift right memory, A = A EOR memory
                self.op_lse_with_addressing(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x53 => {
                // SRE/LSE 本来 未定義命令
                // memory = shift right memory, A = A EOR memory
                self.op_lse_with_addressing(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x57 => {
                // SRE/LSE 本来 未定義命令
                // memory = shift right memory, A = A EOR memory
                self.op_lse_with_addressing(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x5B => {
                // SRE/LSE 本来 未定義命令
                // memory = shift right memory, A = A EOR memory
                self.op_lse_with_addressing(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0x5F => {
                // SRE/LSE 本来 未定義命令
                // memory = shift right memory, A = A EOR memory
                self.op_lse_with_addressing(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x63 => {
                // RRA 本来 未定義命令
                // memory = rotate right memory, A = A + C + memory
                self.op_rra_with_addressing(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0x67 => {
                // RRA 本来 未定義命令
                // memory = rotate right memory, A = A + C + memory
                self.op_rra_with_addressing(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x6F => {
                // RRA 本来 未定義命令
                // memory = rotate right memory, A = A + C + memory
                self.op_rra_with_addressing(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x73 => {
                // RRA 本来 未定義命令
                // memory = rotate right memory, A = A + C + memory
                self.op_rra_with_addressing(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x77 => {
                // RRA 本来 未定義命令
                // memory = rotate right memory, A = A + C + memory
                self.op_rra_with_addressing(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x7B => {
                // RRA 本来 未定義命令
                // memory = rotate right memory, A = A + C + memory
                self.op_rra_with_addressing(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0x7F => {
                // RRA 本来 未定義命令
                // memory = rotate right memory, A = A + C + memory
                self.op_rra_with_addressing(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x1A |
            0x3A |
            0x5A |
            0x7A |
            0xDA |
            0xFA =>
            {
                // 未実装2バイトNOP
                self.program_counter += 1;
            },
            0x0C |
            0x1C |
            0x3C |
            0x5C |
            0x7C |
            0xDC |
            0xFC =>
            {
                // 未実装3バイトNOP
                self.program_counter += 3;
            },
            0x04 |
            0x44 |
            0x64 |
            0x14 |
            0x34 |
            0x54 |
            0x74 |
            0xD4 |
            0xF4 |
            0x80 =>
            {
                // 未実装2バイトNOP
                self.program_counter += 2;
            },
            _ =>
            {
                // System.out.println(Integer.toHexString(opcode & 0xFF));
            }
        }
    }
}


fn push_stack(cpu: &mut Cpu, value: u8, memory_map: &mut MemoryMap){
    let stack_address: u32 = (0x100 as u16 | cpu.reg_s as u16) as u32;
    memory_map.set_from_address(stack_address, value);
    cpu.reg_s = cpu.reg_s.wrapping_sub(1) as u8;
}

pub fn make_nmi_interrupt(cpu: &mut Cpu, memory_map: &mut MemoryMap){
    if cpu.get_flag_i(){
        // 割り込み禁止中
        return;
    }

    let upper = (cpu.program_counter >> 8) as u8;
    push_stack(cpu, upper, memory_map);
    let lower = (cpu.program_counter) as u8;
    push_stack(cpu, lower, memory_map);
    cpu.set_flag_i(true);
    cpu.set_flag_b(false); // TODO: これ必要？
    let next_program_counter = memory_map.get_from_address16(0xFFFA);
    cpu.program_counter = next_program_counter as u32;
}