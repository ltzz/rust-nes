use std::convert::TryInto;

use super::{memory_map::MemoryMap};

pub struct Cpu {
    pub program_counter: u32,
    pub regA: u8,
    pub regX: u8,
    pub regY: u8,
    pub regS: u8,
    pub regP: u8
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
        let regA: u8 = 0;
        let regX: u8 = 0;
        let regY: u8 = 0;
        let regS: u8 = 0;
        let regP: u8 = 0;
        Cpu{program_counter, regA, regX, regY, regS, regP}
    }

    pub fn init(&mut self){
        self.regP = 0x34;
        self.regS = 0xFD;
    }
    
    pub fn next_cycle(&mut self, memory_map: &mut MemoryMap){
        self.interpret(memory_map.get_from_address(self.program_counter), memory_map);
    }

    pub fn set_flagI(&mut self, value: bool){
        self.setP(value, 2);
    }
    pub fn set_flagB(&mut self, value: bool){
        self.setP(value, 4);
    }

    pub fn set_flagN(&mut self, value: bool){
        self.setP(value, 7);
    }
    pub fn get_flagN(&self) -> bool{
        let negative_flag: bool = !((self.regP & 0x80) == 0);
        return negative_flag;
    }

    pub fn set_flagZ(&mut self, value: bool){
        self.setP(value, 1);
    }
    pub fn get_flagZ(&self) -> bool{
        let zero_flag: bool = (self.regP & 0x02) != 0;
        return zero_flag;
    }

    pub fn set_flagC(&mut self, value: bool){
        self.setP(value, 0);
    }
    pub fn get_flagC(&self) -> bool{
        let carry_flag : bool = !((self.regP & 0x01) == 0);
        return carry_flag;
    }

    pub fn set_flagV(&mut self, value: bool){
        self.setP(value, 6);
    }
    pub fn get_flagV(&self) -> bool{
        let overflow_flag: bool = !((self.regP & 0x40) == 0);
        return overflow_flag;
    }

    pub fn setP(&mut self, flag: bool, bitpos: u8){
        if flag {
            self.regP |= 1 << bitpos;
        }
        else {
            self.regP &= !(1 << bitpos); // FIXME: 移植で壊れてないか要確認
        }
    }

    pub fn getIm16(&self, memory_map: &MemoryMap) -> u16{
        return memory_map.get_from_address16(self.program_counter + 1);
    }

    pub fn getIm8(&self, memory_map: &MemoryMap) -> u8{
        return memory_map.get_from_address(self.program_counter + 1);
    }
    
    pub fn get_operand_address(&self, addressing: &Addressing, memory_map: &MemoryMap) -> u32{
        let address = match addressing{
            Addressing::ZeroPage =>
                (self.getIm8(memory_map) & 0xFF) as u32,
            Addressing::ZeroPageX =>
                (((self.getIm8(memory_map) & 0xFF).wrapping_add(self.regX & 0xFF)) & 0xFF) as u32,
            Addressing::ZeroPageY =>
                (((self.getIm8(memory_map) & 0xFF).wrapping_add(self.regY & 0xFF)) & 0xFF) as u32,
            Addressing::Absolute =>
                self.getIm16(memory_map) as u32,
            Addressing::AbsoluteX =>
                ((self.getIm16(memory_map) + ((self.regX & 0xFF)) as u16) & 0xFFFF) as u32,
            Addressing::AbsoluteY =>
                ((self.getIm16(memory_map) + ((self.regY & 0xFF)) as u16) & 0xFFFF) as u32,
            Addressing::Indirect => {
                let immediate16 = self.getIm16(memory_map);
                memory_map.get_from_address_in_page(immediate16 as u32) as u32
            },
            Addressing::IndirectX => {
                let tmp_address = (self.getIm8(memory_map)).wrapping_add(self.regX);
                memory_map.get_from_address16_by_address8(tmp_address) as u32
            },
            Addressing::Indirect_Y =>{
                let tmp_address = self.getIm8(memory_map) & 0xFF;
                let mut ret_address = memory_map.get_from_address16_by_address8(tmp_address) + (self.regY & 0xFF) as u16;
                ret_address &= 0xFFFF;
                ret_address as u32
            },
            _ => 0x0000
        };
        return address;
    }

    pub fn getOperand(&self, addressing: &Addressing, memory_map: &MemoryMap) -> u8{
        let data: u8 = match addressing{
            Addressing::Immediate =>
                self.getIm8(memory_map),
            Addressing::ZeroPage =>
                memory_map.get_from_address(self.get_operand_address(&Addressing::ZeroPage, memory_map)),
            Addressing::ZeroPageX =>
                memory_map.get_from_address(self.get_operand_address(&Addressing::ZeroPageX, memory_map)),
            Addressing::ZeroPageY =>
                memory_map.get_from_address(self.get_operand_address(&Addressing::ZeroPageY, memory_map)),
            Addressing::Absolute =>
                memory_map.get_from_address(self.get_operand_address(&Addressing::Absolute, memory_map)),
            Addressing::AbsoluteX =>
                memory_map.get_from_address(self.get_operand_address(&Addressing::AbsoluteX, memory_map)),
            Addressing::AbsoluteY =>
                memory_map.get_from_address(self.get_operand_address(&Addressing::AbsoluteY, memory_map)),
            Addressing::Indirect =>
                memory_map.get_from_address(self.get_operand_address(&Addressing::Indirect, memory_map)),
            Addressing::IndirectX =>
                memory_map.get_from_address(self.get_operand_address(&Addressing::IndirectX, memory_map)),
            Addressing::Indirect_Y =>
                memory_map.get_from_address(self.get_operand_address(&Addressing::Indirect_Y, memory_map)),
            _ => 0x00
        };
        return data;
    }
    
    pub fn eval_NZ(&mut self, data: u8){
        if (data & 0xFF) < 128 {
            self.set_flagN(false);
        }else{
            self.set_flagN(true);
        }
        if data == 0 {
            self.set_flagZ(true);
        }
        else {
            self.set_flagZ(false);
        }
    }

    pub fn op_TXS(&mut self){
        self.regS = self.regX;
    }
    pub fn op_TSX(&mut self){
        self.eval_NZ(self.regS);
        self.regX = self.regS;
    }
    pub fn op_TAX(&mut self){
        self.eval_NZ(self.regA);
        self.regX = self.regA;
    }
    pub fn op_TXA(&mut self){
        self.eval_NZ(self.regX);
        self.regA = self.regX;
    }
    pub fn op_TYA(&mut self){
        self.eval_NZ(self.regY);
        self.regA = self.regY;
    }
    pub fn op_TAY(&mut self){
        self.eval_NZ(self.regA);
        self.regY = self.regA;
    }

    pub fn op_CPX(&mut self, addressing: &Addressing, memory_map: &MemoryMap){
        let value:u8 = self.getOperand(addressing, memory_map);
        self.setRegAtCompare(self.regX, value);
    }

    pub fn op_CPY(&mut self, addressing: &Addressing, memory_map: &MemoryMap){
        let value:u8 = self.getOperand(addressing, memory_map);
        self.setRegAtCompare(self.regY, value);
    }

    pub fn op_CMP(&mut self, addressing: &Addressing, memory_map: &MemoryMap){
        let value:u8 = self.getOperand(addressing, memory_map);
        self.setRegAtCompare(self.regA, value);
    }

    pub fn op_DCM(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let value:u8 = self.getOperand(addressing, memory_map);
        let result_value:u8 = value - 1;
        memory_map.set_from_address(self.get_operand_address(addressing, memory_map), result_value);
        self.setRegAtCompare(self.regA, result_value);
    }

    pub fn op_ISC(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let value: u8 = self.getOperand(addressing, memory_map);
        let result_value: u8 = value + 1;
        memory_map.set_from_address(self.get_operand_address(addressing, memory_map), result_value);
        self.opSBC_impl(result_value);
    }


    pub fn setRegAtCompare(&mut self, reg: u8, value: u8){
        let reg_value = reg & 0xFF;
        let target_value = value & 0xFF;
        if reg_value >= target_value {
            self.set_flagC(true);
        }else{
            self.set_flagC(false);
        }
        if reg_value == target_value {
            self.set_flagZ(true);
        }else{
            self.set_flagZ(false);
        }
        if ((reg_value.wrapping_sub(target_value)) & 0x80) > 0 {
            self.set_flagN(true);
        }else{
            self.set_flagN(false);
        }
    }

    pub fn opBIT(&mut self, addressing: &Addressing, memory_map: &MemoryMap){
        let address = self.get_operand_address(addressing, memory_map);
        let value = memory_map.get_from_address(address);
        if (value & 0x80) > 0 {
            self.set_flagN(true);
        }
        else{
            self.set_flagN(false);
        }
        if (value & 0x40) > 0 {
            self.set_flagV(true);
        }
        else{
            self.set_flagV(false);
        }
        if (self.regA & value) == 0 { // TODO: ロジック確認してないので要確認
            self.set_flagZ(true);
        }
        else{
            self.set_flagZ(false);
        }
    }


    pub fn opAND(&mut self, addressing: &Addressing, memory_map: &MemoryMap) {
        let value = self.getOperand(addressing, memory_map);
        let result_value = self.regA & value;
        self.regA = result_value;
        self.eval_NZ(self.regA);
    }

    pub fn opEOR(&mut self, addressing: &Addressing, memory_map: &MemoryMap) {
        let value = self.getOperand(addressing, memory_map);
        let result_value = self.regA ^ value;
        self.regA = result_value;
        self.eval_NZ(self.regA);
    }

    pub fn opORA(&mut self, addressing: &Addressing, memory_map: &MemoryMap) {
        let value = self.getOperand(addressing, memory_map);
        let result_value = self.regA | value;
        self.regA = result_value;
        self.eval_NZ(self.regA);
    }


    pub fn opADC(&mut self, addressing: &Addressing, memory_map: &MemoryMap) {
        let value = self.getOperand(addressing, memory_map);
        let carry = self.regP & 0x01;
        let result_value: u16 = (self.regA & 0xFF) as u16 + (value & 0xFF) as u16 + carry as u16;
        let regA_old = self.regA;
        self.regA = (result_value & 0xFF).try_into().unwrap();
        self.eval_NZ(self.regA);
        if result_value >= 0x100 {
            self.set_flagC(true);
        }
        else{
            self.set_flagC(false);
        }
        let result_value_byte: u8 = (result_value & 0xFF).try_into().unwrap();
        if ((result_value_byte ^ value) & (result_value_byte ^ regA_old) & 0x80) !=0 {
            self.set_flagV(true);
        }
        else {
            self.set_flagV(false);
        }
    }

    pub fn opSBC(&mut self, addressing: &Addressing, memory_map: &MemoryMap) {
        let value = self.getOperand(addressing, memory_map);
        self.opSBC_impl(value);
    }

    pub fn opSBC_impl(&mut self, value: u8){
        let carry = self.regP & 0x01;
        let not_carry: u8 = if carry > 0 {0} else {1};
        let result_value: i32 = (self.regA & 0xFF) as i32 - value as  i32 - not_carry as i32;
        let regA_old = self.regA;
        self.regA = (result_value & 0xFF) as u8;
        self.eval_NZ(self.regA);
        if result_value < 0 { // TODO: ロジック確認してないので要確認
            self.set_flagC(false);
        }
        else{
            self.set_flagC(true);
        }

        let result_value_u8 = (result_value & 0xFF) as u8;
        let borrowed_value: u8 = (((value ^ 0xFF) as u16 + 0x100 as u16) & 0xFF).try_into().unwrap();
        if ((result_value_u8 ^ borrowed_value) & (result_value_u8 ^ regA_old) & 0x80) > 0 {
            self.set_flagV(true);
        }
        else {
            self.set_flagV(false);
        }
    }

    pub fn opLSR(&mut self) {
        self.regA = self.opLSR_impl(self.regA);
    }

    pub fn opLSR_with_addressing(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let mut data: u8 = self.getOperand(addressing, memory_map);
        data = self.opLSR_impl(data);
        memory_map.set_from_address(self.get_operand_address(addressing, memory_map), data);
    }

    pub fn opLSR_impl(&mut self, mut data: u8) -> u8{
        let carry: u8 = data & 0x01;
        let result_value: u8 = (data & 0xFF) >> 1;
        data = result_value as u8;
        self.eval_NZ(data);
        if carry > 0 { // TODO: ロジック確認してないので要確認
            self.set_flagC(true);
        }
        else{
            self.set_flagC(false);
        }
        return data;
    }

    pub fn opROR(&mut self) {
        self.regA = self.opROR_impl(self.regA);
    }

    pub fn opROR_with_addressing(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let mut data: u8 = self.getOperand(addressing, memory_map);
        data = self.opROR_impl(data);
        memory_map.set_from_address(self.get_operand_address(addressing, memory_map), data);
    }

    pub fn opROR_impl(&mut self, mut data: u8) -> u8{
        let carry = if self.get_flagC() {1} else {0};
        let output_carry = data & 0x01;
        let result_value: u8 = (data & 0xFF) >> 1;
        data = result_value as u8;
        data |= carry << 7;
        self.eval_NZ(data);
        if output_carry > 0 { // TODO: ロジック確認してないので要確認
            self.set_flagC(true);
        }
        else{
            self.set_flagC(false);
        }
        data
    }

    pub fn opROL(&mut self) {
        self.regA = self.opROL_impl(self.regA);
    }
    pub fn opROL_with_addressing(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap) {
        let mut data: u8 = self.getOperand(addressing, memory_map);
        data = self.opROL_impl(data);
        memory_map.set_from_address(self.get_operand_address(addressing, memory_map), data);
    }

    pub fn opROL_impl(&mut self, mut data: u8) -> u8{
        let output_carry: u8 = data & 0x80;
        let result_value: u16 = ((data & 0xFF) << 1) as u16;
        data = result_value as u8;
        data |= if self.get_flagC() {0x01} else {0x00};
        self.eval_NZ(data);
        if output_carry > 0 { // TODO: ロジック確認してないので要確認
            self.set_flagC(true);
        }
        else{
            self.set_flagC(false);
        }
        return data;
    }

    pub fn opASL(&mut self) {
        let result_value: u16 = ((self.regA & 0xFF) << 1) as u16;
        self.regA = (result_value) as u8;
        self.eval_NZ(self.regA);
        if result_value >= 0x100 { // TODO: ロジック確認してないので要確認
            self.set_flagC(true);
        }
        else{
            self.set_flagC(false);
        }
    }

    pub fn opASL_with_addressing(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap) {
        let address: u32 = self.get_operand_address(addressing, memory_map);
        let mut value: u8 = memory_map.get_from_address(address);
        let result_value: u16 = ((value & 0xFF) << 1) as u16;
        value = (result_value) as u8;
        memory_map.set_from_address(address, value);
        self.eval_NZ(value);
        if result_value >= 0x100 { // TODO: ロジック確認してないので要確認
            self.set_flagC(true);
        }
        else{
            self.set_flagC(false);
        }
    }

    pub fn opINX(&mut self){
        self.regX = (self.regX.wrapping_add(1)) as u8;
        self.eval_NZ(self.regX);
    }

    pub fn opINC(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let address: u32 = self.get_operand_address(addressing, memory_map);
        let mut value: u8 = memory_map.get_from_address(address);
        value = (value.wrapping_add(1)) as u8;
        memory_map.set_from_address(address, value);
        self.eval_NZ(value);
    }

    pub fn opDEC(&mut self, addressing: &Addressing, memory_map: &mut MemoryMap){
        let address: u32 = self.get_operand_address(addressing, memory_map);
        let mut value: u8 = memory_map.get_from_address(address);
        value = (value.wrapping_sub(1)) as u8;
        memory_map.set_from_address(address, value);
        self.eval_NZ(value);
    }

    pub fn opINY(&mut self){
        self.regY = (self.regY.wrapping_add(1)) as u8;
        self.eval_NZ(self.regY);
    }

    pub fn opDEX(&mut self){
        self.regX = (self.regX.wrapping_sub(1)) as u8;
        self.eval_NZ(self.regX);
    }

    pub fn opDEY(&mut self){
        self.regY = (self.regY.wrapping_sub(1)) as u8;
        self.eval_NZ(self.regY);
    }

    pub fn op_CLC(&mut self){
        self.regP = (self.regP & 0xFE) as u8;
    }
    pub fn op_CLD(&mut self){
        self.regP = (self.regP & 0xF7) as u8;
    }
    pub fn op_CLV(&mut self){
        self.regP = (self.regP & 0xBF) as u8;
    }
    pub fn op_SEC(&mut self){
        self.regP = (self.regP | 0x01) as u8;
    }

    pub fn op_SED(&mut self){
        self.regP = (self.regP | 0x08) as u8;
    }

    pub fn opSTA(&self, addressing: &Addressing, memory_map: &mut MemoryMap){
        memory_map.set_from_address(self.get_operand_address(addressing, memory_map), self.regA);
    }

    pub fn opSTX(&self, addressing: &Addressing, memory_map: &mut MemoryMap){
        memory_map.set_from_address(self.get_operand_address(addressing, memory_map), self.regX);
    }

    pub fn op_STY(&self, addressing: &Addressing, memory_map: &mut MemoryMap){
        memory_map.set_from_address(self.get_operand_address(addressing, memory_map), self.regY);
    }
    pub fn op_SAX(&self, addressing: &Addressing, memory_map: &mut MemoryMap){
        memory_map.set_from_address(self.get_operand_address(addressing, memory_map), (self.regA & self.regX) as u8);
    }

    pub fn opLDA(&mut self, addressing: &Addressing, memory_map: &MemoryMap){
        let operand: u8 = self.getOperand(addressing, memory_map);
        self.eval_NZ(operand);
        self.regA = operand;
    }
    pub fn op_LDX(&mut self, addressing: &Addressing, memory_map: &MemoryMap){
        let operand: u8 = self.getOperand(addressing, memory_map);
        self.eval_NZ(operand);
        self.regX = operand;
    }
    pub fn opLDY(&mut self, addressing: &Addressing, memory_map: &MemoryMap){
        let operand: u8 = self.getOperand(addressing, memory_map);
        self.eval_NZ(operand);
        self.regY = operand;
    }
    pub fn opLAX(&mut self, addressing: &Addressing, memory_map: &MemoryMap){
        let operand: u8 = self.getOperand(addressing, memory_map);
        self.eval_NZ(operand);
        self.regA = operand;
        self.regX = operand;
    }

    pub fn opBNE(&mut self, memory_map: &MemoryMap){
        let zero_flag: bool = self.get_flagZ();
        if !zero_flag {
            let relative: u8 = self.getIm8(memory_map);
            let relative_ = if relative > 0x80 {relative as i16 - 0x100 as i16} else {relative as i16};
            self.program_counter = (self.program_counter as i32 + relative_ as i32) as u32;
        }
    }
    pub fn opBPL(&mut self, memory_map: &MemoryMap){
        let negative_flag: bool = self.get_flagN();
        if !negative_flag {
            let relative: u8 = self.getIm8(memory_map);
            let relative_ = if relative > 0x80 {relative as i16 - 0x100 as i16} else {relative as i16};
            self.program_counter = (self.program_counter as i32 + relative_ as i32) as u32;
        }
    }
    pub fn opBCC(&mut self, memory_map: &MemoryMap){
        let carry_flag: bool = self.get_flagC();
        if !carry_flag {
            let relative: u8 = self.getIm8(memory_map);
            let relative_ = if relative > 0x80 {relative as i16 - 0x100 as i16} else {relative as i16};
            self.program_counter = (self.program_counter as i32 + relative_ as i32) as u32;
        }
    }
    pub fn opBCS(&mut self, memory_map: &MemoryMap){
        let carry_flag: bool = self.get_flagC();
        if carry_flag {
            let relative: u8 = self.getIm8(memory_map);
            let relative_ = if relative > 0x80 {relative as i16 - 0x100 as i16} else {relative as i16};
            self.program_counter = (self.program_counter as i32 + relative_ as i32) as u32;
        }
    }
    pub fn opBVS(&mut self, memory_map: &MemoryMap){
        let overflow_flag: bool = self.get_flagV();
        if overflow_flag {
            let relative: u8 = self.getIm8(memory_map);
            let relative_ = if relative > 0x80 {relative as i16 - 0x100 as i16} else {relative as i16};
            self.program_counter = (self.program_counter as i32 + relative_ as i32) as u32;
        }
    }
    pub fn opBVC(&mut self, memory_map: &MemoryMap){
        let overflow_flag: bool = self.get_flagV();
        if !overflow_flag {
            let relative: u8 = self.getIm8(memory_map);
            let relative_ = if relative > 0x80 {relative as i16 - 0x100 as i16} else {relative as i16};
            self.program_counter = (self.program_counter as i32 + relative_ as i32) as u32;
        }
    }
    pub fn opBMI(&mut self, memory_map: &MemoryMap){
        let negative_flag: bool = self.get_flagN();
        if negative_flag {
            let relative: u8 = self.getIm8(memory_map);
            let relative_ = if relative > 0x80 {relative as i16 - 0x100 as i16} else {relative as i16};
            self.program_counter = (self.program_counter as i32 + relative_ as i32) as u32;
        }
    }
    pub fn opBEQ(&mut self, memory_map: &MemoryMap){
        let zero_flag: bool = self.get_flagZ();
        if zero_flag {
            let relative: u8 = self.getIm8(memory_map);
            let relative_ = if relative > 0x80 {relative as i16 - 0x100 as i16} else {relative as i16};
            self.program_counter = (self.program_counter as i32 + relative_ as i32) as u32;
        }
    }

    pub fn opJSR(&mut self, memory_map: &mut MemoryMap){
        let absolute: u16 = self.getIm16(memory_map);
        let return_address = self.program_counter + 2; // この命令の最後のアドレスをpush
        let upper: u8 = ((return_address >> 8) & 0xFF) as u8;
        let lower: u8 = (return_address & 0xFF) as u8;
        let stack_address: u32 = 0x100 + (self.regS & 0xFF) as u32;
        memory_map.set_from_address(stack_address, upper);
        memory_map.set_from_address(stack_address- 1, lower);
        self.regS = (self.regS - 2) as u8;
        self.program_counter = absolute as u32;
    }

    pub fn opPHA(&mut self, memory_map: &mut MemoryMap){
        let stack_address: u32 = (0x100 as u16 + (self.regS & 0xFF) as u16) as u32;
        memory_map.set_from_address(stack_address, self.regA);
        self.regS = (self.regS - 1) as u8;
    }

    pub fn opPHP(&mut self, memory_map: &mut MemoryMap){
        let stack_address: u32 = (0x100 as u16 + (self.regS & 0xFF) as u16) as u32;
        let value = self.regP | 0x10; // ファミコンの仕様 PHPによってスタックに格納する状態フラグでは、ブレイクフラグをセット
        memory_map.set_from_address(stack_address, value);
        self.regS = (self.regS - 1) as u8;
    }
    pub fn opPLP(&mut self, memory_map: &MemoryMap){
        let stack_address: u32 = (0x100 as u16 + (self.regS & 0xFF) as u16 + 1) as u32;
        let mut value = memory_map.get_from_address(stack_address);
        value = value & 0xEF | 0x20;
        // bit4: ブレイクフラグは実際には存在しないためPへのセット時クリア
        // bit5: Rフラグはは常にセット
        self.regP = value;
        self.regS = (self.regS + 1) as u8;
    }

    pub fn opRTS(&mut self, memory_map: &MemoryMap){
        let stack_address: u32 = (0x100 as u16 + (self.regS & 0xFF) as u16 + 1) as u32;
        let lower: u8 = memory_map.get_from_address(stack_address);
        let upper: u8 = memory_map.get_from_address(stack_address + 1);
        self.program_counter = ((((upper & 0xFF) as u16) << 8) | (lower & 0xFF) as u16) as u32;
        //programCounter = (upper << 8) | lower;
        self.regS = (self.regS + 2) as u8;
    }

    pub fn opRTI(&mut self, memory_map: &MemoryMap) {

        // Pをpull
        let stack_addressP: u32 = (0x100 as u16 + (self.regS & 0xFF) as u16 + 1) as u32;
        self.regP = memory_map.get_from_address(stack_addressP);
        self.regS = (self.regS + 1) as u8;;

        // プログラムカウンタをpull
        let stack_address: u32 = (0x100 as u16 + (self.regS & 0xFF) as u16 + 1) as u32;
        let lower: u8 = memory_map.get_from_address(stack_address);
        let upper: u8 = memory_map.get_from_address(stack_address + 1);
        self.program_counter = ((((upper & 0xFF) as u16) << 8) | (lower & 0xFF) as u16) as u32;
        //programCounter = (upper << 8) | lower;
        self.regS = (self.regS + 2) as u8;

    }

    pub fn opPLA(&mut self, memory_map: &MemoryMap){
        let address: u32 = (0x100 as u16 + (self.regS & 0xFF) as u16 + 1) as u32;
        let value: u8 = memory_map.get_from_address(address);
        self.regA = value;
        self.eval_NZ(self.regA);
        self.regS = (self.regS + 1) as u8;
    }

    pub fn opBRK(&mut self){
        self.set_flagI(true);
        self.set_flagB(true);
    }

    pub fn opJMP_Abs(&mut self, memory_map: &MemoryMap){
        let absolute : u16 = self.getIm16(memory_map);
        self.program_counter = absolute as u32;
    }
    pub fn opJMP_Indirect(&mut self, memory_map: &MemoryMap){
        let address: u32 = self.get_operand_address(&Addressing::Indirect, memory_map);
        self.program_counter = address;
    }

    pub fn interpret(&mut self, opcode: u8, memory_map: &mut MemoryMap){

        let opcodeInt: u8 = opcode & 0xFF;
        match(opcodeInt){
            0xA2 =>//LDX(Immediate):メモリからXにロード(2バイト/2サイクル)
            {
                self.op_LDX(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0xA6 =>//LDX(Zeropage):メモリからXにロード(2バイト/3サイクル)
            {
                self.op_LDX(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xB6 =>//LDX(Zeropage,Y):メモリからXにロード(2バイト/4サイクル)
            {
                self.op_LDX(&Addressing::ZeroPageY, memory_map);
                self.program_counter += 2;
            },
            0xAE =>//LDX(Absolute):メモリからXにロード(3バイト/4サイクル)
            {
                self.op_LDX(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            }
            0xBE =>//LDX(Absolute, Y):メモリからXにロード(3バイト/4サイクル)
            {
                self.op_LDX(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            }
            0x78 =>//SEI:IRQ割り込みの禁止(1バイト/2サイクル)
            {
                self.set_flagI(true);
                self.program_counter += 1;
            },
            0xA9 =>//LDA(Immediate):メモリからAにロード(2バイト/2サイクル)
            {
                self.opLDA(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0xA5 =>//LDA(Zeropage):メモリからAにロード(2バイト/3サイクル)
            {
                self.opLDA(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xB5 =>//LDA(ZeropageX):メモリからAにロード(2バイト/4サイクル)
            {
                self.opLDA(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0xAD =>//LDA(Absolute):メモリからAにロード(3バイト/4サイクル)
            {
                self.opLDA(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xBD =>//LDA(AbsoluteX):メモリからAにロード(3バイト/4サイクル)
            {
                self.opLDA(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0xB9 =>//LDA(AbsoluteY):メモリからAにロード(3バイト/4サイクル)
            {
                self.opLDA(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0xA1 =>//LDA(IndirectX):メモリからAにロード(2バイト/6サイクル)
            {
                self.opLDA(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0xB1 =>//LDA(Indirect_Y):メモリからAにロード(2バイト/5サイクル)
            {
                self.opLDA(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0xA0 =>//LDY(Immediate):メモリからYにロード(2バイト/2サイクル)
            {
                self.opLDY(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0xA4 =>//LDY(ZeroPage):メモリからYにロード(2バイト/3サイクル)
            {
                self.opLDY(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xB4 =>//LDY(ZeroPageX):メモリからYにロード(2バイト/4サイクル)
            {
                self.opLDY(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0xAC =>//LDY(Absolute):メモリからAにロード(3バイト/4サイクル)
            {
                self.opLDY(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xBC =>//LDY(Absolute, X):メモリからAにロード(3バイト/4サイクル)
            {
                self.opLDY(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0xA7 => // LAX ※拡張命令
            {
                self.opLAX(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xB7 => // LAX ※拡張命令
            {
                self.opLAX(&Addressing::ZeroPageY, memory_map);
                self.program_counter += 2;
            },
            0xAF => // LAX ※拡張命令
            {
                self.opLAX(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xBF => // LAX ※拡張命令
            {
                self.opLAX(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0xA3 => // LAX ※拡張命令
            {
                self.opLAX(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0xB3 => // LAX ※拡張命令
            {
                self.opLAX(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x85 =>//STA(Zeropage):Aからメモリにストア(2バイト/3サイクル)
            {
                self.opSTA(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x95 =>//STA(ZeropageX):Aからメモリにストア(2バイト/4サイクル)
            {
                self.opSTA(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x8D =>//STA(Absolute):Aからメモリにストア(3バイト/4サイクル)
            {
                self.opSTA(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x9D =>//STA(AbsoluteX):Aからメモリにストア(3バイト/5サイクル)
            {
                self.opSTA(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x99 =>//STA(AbsoluteY):Aからメモリにストア(3バイト/5サイクル)
            {
                self.opSTA(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0x91 =>//STA(Indirect_Y):Aからメモリにストア(2バイト/6サイクル)
            {
                self.opSTA(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x81 =>//STA(Indirect,X):Aからメモリにストア(2バイト/6サイクル)
            {
                self.opSTA(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0x86 =>//STX(Zeropage):Xからメモリにストア(2バイト/3サイクル)
            {
                self.opSTX(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x96 =>//STX(Zeropage,Y):Xからメモリにストア(2バイト/4サイクル)
            {
                self.opSTX(&Addressing::ZeroPageY, memory_map);
                self.program_counter += 2;
            },
            0x8E =>//STX(Absolute):Xからメモリにストア(3バイト/4サイクル)
            {
                self.opSTX(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x8C =>//STY(Absolute):Yからメモリにストア(3バイト/4サイクル)
            {
                self.op_STY(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x84 =>//STY(Zeropage):Yからメモリにストア(2バイト/3サイクル)
            {
                self.op_STY(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x94 =>//STY(ZeropageX):Yからメモリにストア(2バイト/4サイクル)
            {
                self.op_STY(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x87 =>//SAX ※拡張命令
            {
                self.op_SAX(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x97 =>//SAX ※拡張命令
            {
                self.op_SAX(&Addressing::ZeroPageY, memory_map);
                self.program_counter += 2;
            },
            0x8F =>//SAX ※拡張命令
            {
                self.op_SAX(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x83 =>//SAX ※拡張命令
            {
                self.op_SAX(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0x9A =>
                // TODO: Sに0を入れているROMがあり、うまく動作しない（あるいは入れる元の計算結果が誤り
            {
                self.op_TXS();
                self.program_counter += 1;
            },
            0xBA =>
            {
                self.op_TSX();
                self.program_counter += 1;
            },
            0xAA =>
            {
                self.op_TAX();
                self.program_counter += 1;
            },
            0x8A =>
            {
                self.op_TXA();
                self.program_counter += 1;
            },
            0x98 =>
            {
                self.op_TYA();
                self.program_counter += 1;
            },
            0xA8 =>
            {
                self.op_TAY();
                self.program_counter += 1;
            },
            0xC0 =>
            {
                self.op_CPY(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0xC4 =>
            {
                self.op_CPY(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xCC =>
            {
                self.op_CPY(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xE0 =>
            {
                self.op_CPX(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0xE4 =>
            {
                self.op_CPX(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xEC =>
            {
                self.op_CPX(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xC5 =>
            {
                self.op_CMP(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xD5 =>
            {
                self.op_CMP(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0xC9 =>
            {
                self.op_CMP(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0xCD =>
            {
                self.op_CMP(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xDD =>
            {
                self.op_CMP(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0xD9 =>
            {
                self.op_CMP(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0xC1 =>
            {
                self.op_CMP(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0xD1 =>
            {
                self.op_CMP(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x2C =>
            {
                self.opBIT(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x24 =>
            {
                self.opBIT(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x29 =>
            {
                self.opAND(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0x25 =>
            {
                self.opAND(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x35 =>
            {
                self.opAND(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x2D =>
            {
                self.opAND(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x3D =>
            {
                self.opAND(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x39 =>
            {
                self.opAND(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0x21 =>
            {
                self.opAND(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0x31 =>
            {
                self.opAND(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x49 =>
            {
                self.opEOR(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0x45 =>
            {
                self.opEOR(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x55 =>
            {
                self.opEOR(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x4D =>
            {
                self.opEOR(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x5D =>
            {
                self.opEOR(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x59 =>
            {
                self.opEOR(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0x41 =>
            {
                self.opEOR(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0x51 =>
            {
                self.opEOR(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x09 =>
            {
                self.opORA(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0x05 =>
            {
                self.opORA(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x15 =>
            {
                self.opORA(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x0D =>
            {
                self.opORA(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x1D =>
            {
                self.opORA(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x19 =>
            {
                self.opORA(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0x01 =>
            {
                self.opORA(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0x11 =>
            {
                self.opORA(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x69 =>
            {
                self.opADC(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0x61 =>
            {
                self.opADC(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0x71 =>
            {
                self.opADC(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x65 =>
            {
                self.opADC(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x75 =>
            {
                self.opADC(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x6D =>
            {
                self.opADC(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x7D =>
            {
                self.opADC(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x79 =>
            {
                self.opADC(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0xE9 =>
            {
                self.opSBC(&Addressing::Immediate, memory_map);
                self.program_counter += 2;
            },
            0xE5 =>
            {
                self.opSBC(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xF5 =>
            {
                self.opSBC(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0xED =>
            {
                self.opSBC(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xFD =>
            {
                self.opSBC(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0xF9 =>
            {
                self.opSBC(&Addressing::AbsoluteY, memory_map);
                self.program_counter += 3;
            },
            0xE1 =>
            {
                self.opSBC(&Addressing::IndirectX, memory_map);
                self.program_counter += 2;
            },
            0xF1 =>
            {
                self.opSBC(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0x06 =>
            {
                self.opASL_with_addressing(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x16 =>
            {
                self.opASL_with_addressing(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x0E =>
            {
                self.opASL_with_addressing(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x1E =>
            {
                self.opASL_with_addressing(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x0A =>
            {
                self.opASL();
                self.program_counter += 1;
            },
            0x4A =>
            {
                self.opLSR();
                self.program_counter += 1;
            },
            0x46 =>
            {
                self.opLSR_with_addressing(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x56 =>
            {
                self.opLSR_with_addressing(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x4E =>
            {
                self.opLSR_with_addressing(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x5E =>
            {
                self.opLSR_with_addressing(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x6A =>
            {
                self.opROR();
                self.program_counter += 1;
            },
            0x66 =>
            {
                self.opROR_with_addressing(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x76 =>
            {
                self.opROR_with_addressing(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x6E =>
            {
                self.opROR_with_addressing(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x7E =>
            {
                self.opROR_with_addressing(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0x2A =>
            {
                self.opROL();
                self.program_counter += 1;
            },
            0x26 =>
            {
                self.opROL_with_addressing(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0x36 =>
            {
                self.opROL_with_addressing(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0x2E =>
            {
                self.opROL_with_addressing(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0x3E =>
            {
                self.opROL_with_addressing(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0xE8 =>
            {
                self.opINX();
                self.program_counter += 1;
            },
            0xC8 =>
            {
                self.opINY();
                self.program_counter += 1;
            },
            0xE6 => // (2バイト/5サイクル)
            {
                self.opINC(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xF6 => // (2バイト/6サイクル)
            {
                self.opINC(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0xEE => // (3バイト/6サイクル)
            {
                self.opINC(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xFE => // (3バイト/7サイクル)
            {
                self.opINC(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0xCA =>
            {
                self.opDEX();
                self.program_counter += 1;
            },
            0x88 =>
            {
                self.opDEY();
                self.program_counter += 1;
            },
            0xC6 =>
            {
                self.opDEC(&Addressing::ZeroPage, memory_map);
                self.program_counter += 2;
            },
            0xD6 =>
            {
                self.opDEC(&Addressing::ZeroPageX, memory_map);
                self.program_counter += 2;
            },
            0xCE =>
            {
                self.opDEC(&Addressing::Absolute, memory_map);
                self.program_counter += 3;
            },
            0xDE =>
            {
                self.opDEC(&Addressing::AbsoluteX, memory_map);
                self.program_counter += 3;
            },
            0xD0 =>
            {
                self.opBNE(memory_map);
                self.program_counter += 2;
            },
            0x10 =>
            {
                self.opBPL(memory_map);
                self.program_counter += 2;
            },
            0x90 =>
            {
                self.opBCC(memory_map);
                self.program_counter += 2;
            },
            0xB0 =>
            {
                self.opBCS(memory_map);
                self.program_counter += 2;
            },
            0x70 =>
            {
                self.opBVS(memory_map);
                self.program_counter += 2;
            },
            0x50 =>
            {
                self.opBVC(memory_map);
                self.program_counter += 2;
            },
            0x30 =>
            {
                self.opBMI(memory_map);
                self.program_counter += 2;
            },
            0xF0 =>
            {
                self.opBEQ(memory_map);
                self.program_counter += 2;
            },
            0x20 =>
            {
                self.opJSR(memory_map);
            },
            0x48 =>
            {
                self.opPHA(memory_map);
                self.program_counter += 1;
            },
            0x08 =>
            {
                self.opPHP(memory_map);
                self.program_counter += 1;
            },
            0x28 =>
            {
                self.opPLP(memory_map);
                self.program_counter += 1;
            },
            0x68 =>
            {
                self.opPLA(memory_map);
                self.program_counter += 1;
            },
            0x60 =>
            {
                self.opRTS(memory_map);
                self.program_counter += 1;
            },
            0x40 =>
            {
                self.opRTI(memory_map);
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
                self.op_SEC();
                self.program_counter += 1;
            },
            0xF8 => // SED ファミコン用6502ではフラグ変更のみ
            {
                self.op_SED();
                self.program_counter += 1;
            },
            0x18 =>
            {
                self.op_CLC();
                self.program_counter += 1;
            },
            0xD8 => // CLD ファミコン用6502ではフラグ変更のみ
            {
                self.op_CLD();
                self.program_counter += 1;
            },
            0xB8 =>
            {
                self.op_CLV();
                self.program_counter += 1;
            },
            0xEB => // SBC ※拡張命令
            {
                self.opSBC(&Addressing::Immediate, memory_map);
                self.program_counter+= 2;
            },
            0xC7 => // DCM(DCP) ※拡張命令
            {
                self.op_DCM(&Addressing::ZeroPage, memory_map);
                self.program_counter+= 2;
            },
            0xD7 => // DCM(DCP) ※拡張命令
            {
                self.op_DCM(&Addressing::ZeroPageX, memory_map);
                self.program_counter+= 2;
            },
            0xCF => // DCM(DCP) ※拡張命令
            {
                self.op_DCM(&Addressing::Absolute, memory_map);
                self.program_counter+= 3;
            },
            0xDF => // DCM(DCP) ※拡張命令
            {
                self.op_DCM(&Addressing::AbsoluteX, memory_map);
                self.program_counter+= 3;
            },
            0xDB => // DCM(DCP) ※拡張命令
            {
                self.op_DCM(&Addressing::AbsoluteY, memory_map);
                self.program_counter+= 3;
            },
            0xC3 => // DCM(DCP) ※拡張命令
            {
                self.op_DCM(&Addressing::IndirectX, memory_map);
                self.program_counter+= 2;
            },
            0xD3 => // DCM(DCP) ※拡張命令
            {
                self.op_DCM(&Addressing::Indirect_Y, memory_map);
                self.program_counter+= 2;
            },
            0xE7 => // ISC ※拡張命令
            {
                self.op_ISC(&Addressing::ZeroPage, memory_map);
                self.program_counter+= 2;
            },
            0xF7 => // ISC(ISB) ※拡張命令
            {
                self.op_ISC(&Addressing::ZeroPageX, memory_map);
                self.program_counter+= 2;
            },
            0xEF => // ISC(ISB) ※拡張命令
            {
                self.op_ISC(&Addressing::Absolute, memory_map);
                self.program_counter+= 3;
            },
            0xFF => // ISC(ISB) ※拡張命令
            {
                self.op_ISC(&Addressing::AbsoluteX, memory_map);
                self.program_counter+= 3;
            },
            0xFB => // ISC(ISB) ※拡張命令
            {
                self.op_ISC(&Addressing::AbsoluteY, memory_map);
                self.program_counter+= 3;
            },
            0xE3 => // ISC(ISB) ※拡張命令
            {
                self.op_ISC(&Addressing::IndirectX, memory_map);
                self.program_counter+= 2;
            },
            0xF3 => // ISC(ISB) ※拡張命令
            {
                self.op_ISC(&Addressing::Indirect_Y, memory_map);
                self.program_counter += 2;
            },
            0xEA =>
            {
                // NOP
                self.program_counter += 1;
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