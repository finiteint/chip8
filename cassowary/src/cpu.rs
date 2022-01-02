use crate::display::Display;
use crate::instructions::{Instruction, MemAddr, RegId};
use crate::keyboard::KeyBoard;
use crate::memory::{Memory, MemoryError};
use crate::sound::SoundSystem;
use crate::timer::DelayTimer;

use rand::{self, Rng};
use thiserror::Error;

const TRACE: bool = false;
const COND_REG: RegId = 0xF;
const HEX_SPRITE_BASE: MemAddr = 0x0100;
const HEX_SPRITE_HEIGHT: MemAddr = 5;

#[derive(Error, Debug)]
pub enum CpuError {
    #[error("stack overflowed")]
    StackOverflow,
    #[error("stack underflowed")]
    StackUnderflow,
    #[error("memory address overflow")]
    MemoryAddressOverflow,
    #[error("illegal instruction {0:04X}")]
    IllegalInstruction(u16),
    #[error("memory access error")]
    MemoryError(#[from] MemoryError),
    #[error("halted")]
    Halt,
}

pub struct Cpu {
    registers: [u8; 16],
    pc: MemAddr,
    sp: usize,
    index: MemAddr,
    stack: [MemAddr; 16],
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: [0; 16],
            pc: 0,
            sp: 0,
            index: 0,
            stack: [0; 16],
        }
    }

    pub fn get_register(&self, idx: usize) -> u8 {
        self.registers[idx]
    }

    pub fn set_register(&mut self, idx: usize, value: u8) {
        self.registers[idx] = value;
    }

    pub fn dump(&self) {
        println!("PC: {:03X}    I: {:03X}", self.pc, self.index);
        println!("Regs: ");
        let mut nl = false;
        for (reg_id, value) in self.registers.iter().enumerate() {
            nl = false;
            if reg_id != 0 {
                print!("")
            }
            print!("   {:X}: {:02X} ({:3})", reg_id, value, value);
            if (reg_id + 1) % 4 == 0 {
                println!();
                nl = true;
            }
        }
        if !nl {
            println!();
        }
        println!("Stack: ");
        let mut nl = false;
        for (reg_id, value) in self.stack.iter().enumerate() {
            nl = false;
            let mark = if self.sp == reg_id { ">>" } else { "  " };
            print!(" {:2}{:X}: {:03X}", mark, reg_id, value);
            if (reg_id + 1) % 8 == 0 {
                println!();
                nl = true;
            }
        }
        if !nl {
            println!();
        }
    }
}

impl Cpu {
    pub fn run(
        &mut self,
        mem: &mut Memory,
        delay: &mut DelayTimer,
        display: &mut Display,
        keyboard: &mut KeyBoard,
        sound_timer: &mut SoundSystem,
    ) -> Result<(), CpuError> {
        loop {
            let instr = Instruction::decode(self.fetch(mem)?);
            if TRACE {
                println!("TRACE: {:?}", instr);
            }
            match self.execute(instr, mem, delay, display, keyboard, sound_timer) {
                Err(CpuError::Halt) => return Ok(()),
                Err(err) => return Err(err),
                _ => {
                    if TRACE {
                        self.dump();
                    }
                    continue;
                }
            }
        }
    }

    fn fetch(&mut self, mem: &Memory) -> Result<u16, CpuError> {
        let opcode = mem.load_u16(self.pc)?;
        self.inc_pc()?;
        Ok(opcode)
    }

    fn execute(
        &mut self,
        instr: Instruction,
        mem: &mut Memory,
        delay: &mut DelayTimer,
        display: &mut Display,
        keyboard: &mut KeyBoard,
        sound_timer: &mut SoundSystem,
    ) -> Result<(), CpuError> {
        match instr {
            Instruction::AssignXImm(x, imm) => self.assign_x_imm(x, imm),
            Instruction::AddXImm(x, imm) => self.add_x_imm(x, imm),
            Instruction::AssignXY(x, y) => self.assign_xy(x, y),
            Instruction::OrXY(x, y) => self.or_xy(x, y),
            Instruction::AndXY(x, y) => self.and_xy(x, y),
            Instruction::XorXY(x, y) => self.xor_xy(x, y),
            Instruction::AddXY(x, y) => self.add_xy(x, y),
            Instruction::SubXY(x, y) => self.sub_xy(x, y),
            Instruction::Shr1X(x) => self.shr1_x(x),
            Instruction::SubYX(x, y) => self.sub_yx(x, y),
            Instruction::Shl1X(x) => self.shl1_x(x),
            Instruction::SkipIfEqX(x, imm) => self.skip_if_eq_x(x, imm),
            Instruction::SkipIfNeX(x, imm) => self.skip_if_ne_x(x, imm),
            Instruction::SkipIfEqXY(x, y) => self.skip_if_eq_xy(x, y),
            Instruction::SkipIfNeXY(x, y) => self.skip_if_ne_xy(x, y),
            Instruction::Jump(addr) => self.jump(addr),
            Instruction::JumpV0(addr) => self.jump_v0(addr),
            Instruction::Call(addr) => self.call(addr),
            Instruction::SkipIfKeyEqX(x) => self.skip_if_key_eq_x(x, keyboard),
            Instruction::SkipIfKeyNeX(x) => self.skip_if_key_ne_x(x, keyboard),
            Instruction::GetDelayX(x) => self.get_delay_x(x, delay),
            Instruction::SetDelayX(x) => self.set_delay_x(x, delay),
            Instruction::SetSoundX(x) => self.set_sound_x(x, sound_timer),
            Instruction::AwaitKeyX(x) => self.await_key_x(x, keyboard),
            Instruction::RandX(x, imm) => self.rand_x(x, imm),
            Instruction::AddIX(x) => self.add_i_x(x),
            Instruction::SetI(addr) => self.set_i(addr),
            Instruction::SpriteAddrIX(x) => self.sprite_addr_i_x(x),
            Instruction::DumpBcdIX(x) => self.dump_bcd_i_x(x, mem),
            Instruction::RegDumpIX(x) => self.reg_dump_i_x(x, mem),
            Instruction::RegLoadIX(x) => self.reg_load_i_x(x, mem),
            Instruction::Ret => self.ret(),
            Instruction::Halt => Err(CpuError::Halt),
            Instruction::DispClear => self.display_clear(display),
            Instruction::DispDraw(x, y, imm) => self.display_draw(x, y, imm, display, mem),
            Instruction::NoOp(_) => Ok(()),
            Instruction::Unsupported(opcode) => Err(CpuError::IllegalInstruction(opcode)),
        }
    }

    fn push_stack(&mut self, addr: MemAddr) -> Result<(), CpuError> {
        if self.sp > self.stack.len() {
            return Err(CpuError::StackOverflow);
        }

        self.stack[self.sp] = addr;
        self.sp += 1;
        Ok(())
    }

    fn pop_stack(&mut self) -> Result<MemAddr, CpuError> {
        if self.sp <= 0 {
            return Err(CpuError::StackUnderflow);
        }

        self.sp -= 1;
        Ok(self.stack[self.sp])
    }

    fn set_condition(&mut self, condition: u8) {
        self.registers[COND_REG] = condition;
    }

    fn rand_byte(&mut self) -> u8 {
        rand::thread_rng().gen()
    }

    fn skip_instruction(&mut self) -> Result<(), CpuError> {
        self.inc_pc()
    }

    fn inc_pc(&mut self) -> Result<(), CpuError> {
        self.pc = mem_addr_add(self.pc, 2)?;
        Ok(())
    }
}

// instructions
impl Cpu {
    fn call(&mut self, addr: MemAddr) -> Result<(), CpuError> {
        self.push_stack(self.pc)?;
        self.jump(addr)
    }

    fn jump(&mut self, addr: MemAddr) -> Result<(), CpuError> {
        self.pc = addr;
        Ok(())
    }

    fn jump_v0(&mut self, offset: MemAddr) -> Result<(), CpuError> {
        let v0 = self.registers[0];
        self.pc = mem_addr_add(v0 as MemAddr, offset)?;
        Ok(())
    }

    fn ret(&mut self) -> Result<(), CpuError> {
        self.pc = self.pop_stack()?;
        Ok(())
    }

    fn display_clear(&mut self, display: &mut Display) -> Result<(), CpuError> {
        display.clear();
        Ok(())
    }

    fn add_xy(&mut self, x: RegId, y: RegId) -> Result<(), CpuError> {
        let xv = self.registers[x];
        let yv = self.registers[y];
        let (res, overflowed) = xv.overflowing_add(yv);
        self.registers[x] = res;
        self.set_condition(if overflowed { 1 } else { 0 });
        Ok(())
    }

    fn sub_xy(&mut self, x: RegId, y: RegId) -> Result<(), CpuError> {
        let xv = self.registers[x];
        let yv = self.registers[y];
        let (res, carry) = xv.overflowing_sub(yv);
        self.registers[x] = res;
        self.set_condition(if carry { 0 } else { 1 });
        Ok(())
    }

    fn sub_yx(&mut self, x: RegId, y: RegId) -> Result<(), CpuError> {
        let xv = self.registers[x];
        let yv = self.registers[y];
        let (res, carry) = yv.overflowing_sub(xv);
        self.registers[x] = res;
        self.set_condition(if carry { 0 } else { 1 });
        Ok(())
    }

    fn and_xy(&mut self, x: RegId, y: RegId) -> Result<(), CpuError> {
        let xv = self.registers[x];
        let yv = self.registers[y];
        self.registers[x] = xv & yv;
        Ok(())
    }

    fn or_xy(&mut self, x: RegId, y: RegId) -> Result<(), CpuError> {
        let xv = self.registers[x];
        let yv = self.registers[y];
        self.registers[x] = xv | yv;
        Ok(())
    }

    fn xor_xy(&mut self, x: RegId, y: RegId) -> Result<(), CpuError> {
        let xv = self.registers[x];
        let yv = self.registers[y];
        self.registers[x] = xv ^ yv;
        Ok(())
    }

    fn assign_xy(&mut self, x: RegId, y: RegId) -> Result<(), CpuError> {
        self.registers[x] = self.registers[y];
        Ok(())
    }

    fn shr1_x(&mut self, x: RegId) -> Result<(), CpuError> {
        let xv = self.registers[x];
        let lsb = xv & 0x01;
        self.registers[x] = xv >> 1;
        self.set_condition(lsb);
        Ok(())
    }

    fn shl1_x(&mut self, x: RegId) -> Result<(), CpuError> {
        let xv = self.registers[x];
        let msb = (xv & 0x80) >> 7;
        self.registers[x] = xv << 1;
        self.set_condition(msb);
        Ok(())
    }

    fn add_x_imm(&mut self, x: RegId, imm: u8) -> Result<(), CpuError> {
        let xv = self.registers[x];
        self.registers[x] = xv.wrapping_add(imm);
        Ok(())
    }

    fn assign_x_imm(&mut self, x: RegId, imm: u8) -> Result<(), CpuError> {
        self.registers[x] = imm;
        Ok(())
    }

    fn skip_if_ne_x(&mut self, x: RegId, imm: u8) -> Result<(), CpuError> {
        let xv = self.registers[x];
        if xv != imm {
            self.skip_instruction()?;
        }
        Ok(())
    }

    fn skip_if_eq_x(&mut self, x: RegId, imm: u8) -> Result<(), CpuError> {
        let xv = self.registers[x];
        if xv == imm {
            self.skip_instruction()?;
        }
        Ok(())
    }

    fn skip_if_eq_xy(&mut self, x: RegId, y: RegId) -> Result<(), CpuError> {
        let xv = self.registers[x];
        let yv = self.registers[y];
        if xv == yv {
            self.skip_instruction()?;
        }
        Ok(())
    }

    fn skip_if_ne_xy(&mut self, x: RegId, y: RegId) -> Result<(), CpuError> {
        let xv = self.registers[x];
        let yv = self.registers[y];
        if xv != yv {
            self.skip_instruction()?;
        }
        Ok(())
    }

    fn skip_if_key_eq_x(&mut self, x: RegId, keyboard: &mut KeyBoard) -> Result<(), CpuError> {
        let xv = self.registers[x];
        let key = keyboard.get_key_pressed();
        if xv == key {
            self.skip_instruction()?;
        }
        Ok(())
    }

    fn skip_if_key_ne_x(&mut self, x: RegId, keyboard: &mut KeyBoard) -> Result<(), CpuError> {
        let xv = self.registers[x];
        let key = keyboard.get_key_pressed();
        if xv != key {
            self.skip_instruction()?;
        }
        Ok(())
    }

    fn await_key_x(&mut self, x: RegId, keyboard: &mut KeyBoard) -> Result<(), CpuError> {
        self.registers[x] = keyboard.await_key_press();
        Ok(())
    }

    fn rand_x(&mut self, x: RegId, imm: u8) -> Result<(), CpuError> {
        self.registers[x] = self.rand_byte() & imm;
        Ok(())
    }

    fn get_delay_x(&mut self, x: RegId, delay: &mut DelayTimer) -> Result<(), CpuError> {
        self.registers[x] = delay.get();
        Ok(())
    }

    fn set_delay_x(&mut self, x: RegId, delay: &mut DelayTimer) -> Result<(), CpuError> {
        delay.set(self.registers[x]);
        Ok(())
    }

    fn set_sound_x(&mut self, x: RegId, sound_timer: &mut SoundSystem) -> Result<(), CpuError> {
        sound_timer.set_timer(self.registers[x]);
        Ok(())
    }

    fn add_i_x(&mut self, x: RegId) -> Result<(), CpuError> {
        self.index += self.registers[x] as MemAddr;
        Ok(())
    }

    fn sprite_addr_i_x(&mut self, x: RegId) -> Result<(), CpuError> {
        let xv = self.registers[x];
        self.index = HEX_SPRITE_BASE + (xv as MemAddr) * HEX_SPRITE_HEIGHT;
        Ok(())
    }

    fn dump_bcd_i_x(&mut self, x: RegId, mem: &mut Memory) -> Result<(), CpuError> {
        let base = self.index;
        let bcdx = format!("{:03}", self.registers[x]);
        let bcds = bcdx.chars().map(|c| char_to_bcd(c));
        for (offset, d) in bcds.into_iter().enumerate() {
            let addr = mem_addr_add(base, offset)?;
            mem.store_byte(addr, d)?;
        }
        Ok(())
    }

    fn reg_dump_i_x(&mut self, x: RegId, mem: &mut Memory) -> Result<(), CpuError> {
        let base = self.index;
        for (offset, r) in (0..=x).enumerate() {
            let addr = mem_addr_add(base, offset)?;
            mem.store_byte(addr, self.registers[r])?;
        }
        Ok(())
    }

    fn reg_load_i_x(&mut self, x: RegId, mem: &mut Memory) -> Result<(), CpuError> {
        let base = self.index;
        for (offset, r) in (0..=x).enumerate() {
            self.registers[r] = mem.load_byte(base + offset)?;
        }
        Ok(())
    }

    fn set_i(&mut self, addr: MemAddr) -> Result<(), CpuError> {
        self.index = addr;
        Ok(())
    }

    fn display_draw(
        &mut self,
        x: RegId,
        y: RegId,
        imm: u8,
        display: &mut Display,
        mem: &Memory,
    ) -> Result<(), CpuError> {
        let xv = self.registers[x];
        let yv = self.registers[y];
        let collision = display.draw(xv, yv, imm, self.index, mem)?;
        self.set_condition(if collision { 0x01 } else { 0x00 });
        Ok(())
    }
}

fn mem_addr_add(base: MemAddr, offset: MemAddr) -> Result<MemAddr, CpuError> {
    base.checked_add(offset)
        .ok_or(CpuError::MemoryAddressOverflow)
}

fn char_to_bcd(c: char) -> u8 {
    match c {
        '0'..='9' => (c as u8) - b'0',
        _ => unreachable!(),
    }
}
