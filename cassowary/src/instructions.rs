//! The information used here is taken from https://en.wikipedia.org/wiki/CHIP-8
//! and http://devernay.free.fr/hacks/chip8/C8TECH10.HTM
//!

pub type MemAddr = usize;
pub type RegId = usize;

///
/// The following convention is used in the descriptions:
/// - `VX` `VY` for register variables
/// - `VN` where `N` is the register number (`0` to `F`), e.g. `V0`
/// - `N`, `NN`, and `NNN`:  4, 8, and 12 bit immediate values
/// - `I` index/base register (from which memory addresses are computed)
/// - `[X]` where `X` is a register: memory location from address stored in `X`, e.g. `[I]`
#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    // Const
    /// `LD VX NN`
    /// `VX <- NN`
    AssignXImm(RegId, u8),

    /// `ADD VX NN`
    /// `VX <- VX + NN`
    AddXImm(RegId, u8),

    /// `LD VX VY`
    /// `VX <- VY`
    AssignXY(RegId, RegId),

    // BitOp/Math
    /// `OR VX VY`
    /// `VX <- VX | VY`
    OrXY(RegId, RegId),

    /// `AND VX VY`
    /// `VX <- VX & VY`
    AndXY(RegId, RegId),

    /// `XOR VX VY`
    /// `VX <- VX ^ VY`
    XorXY(RegId, RegId),

    /// `ADD VX VY`
    /// `VX <- VX + VY`
    /// `VF <- OVERFLOW` where `OVERFLOW` is 1 on overflow, 0 otherwise
    AddXY(RegId, RegId),

    /// `SUB VX VY`
    /// `VX <- VX - VY`
    /// `VF <- CARRY` where `CARRY` is 0 on carry, 1 otherwise
    SubXY(RegId, RegId),

    /// `SHR VX`
    /// `VX <- VX >> 1`
    /// `VF <- LSB` where `LSB` is the least significant bit before shift
    Shr1X(RegId),

    /// `SUBN VX VY`
    /// `VX <- VY - VX`
    /// `VF <- CARRY` where `CARRY` is 0 on carry, 1 otherwise
    SubYX(RegId, RegId),

    /// `SHL VX`
    /// `VX <- VX << 1`
    /// `VF <- MSB` where `MSB` is the most significant bit before shift
    Shl1X(RegId),

    // Display
    /// `CLS`
    /// clear display
    DispClear,

    /// `DRW VX VY N`
    /// Display sprite of `N` height at coordinate VX, VY.
    /// The sprite is read from `I`.
    /// `VF <- COLLISTION` `COLLISION` is if any pixels were flipped
    DispDraw(RegId, RegId, u8),

    // Cond
    /// `SE VX NN`
    /// skip next instruction if `VX = NN`
    SkipIfEqX(RegId, u8),

    /// `SNE VX NN`
    /// skip next instruction if `VX != NN`
    SkipIfNeX(RegId, u8),

    /// `SE VX VY`
    /// skip next instruction if `VX = VY`
    SkipIfEqXY(RegId, RegId),

    /// `SNE VX VY`
    /// skip next instruction if `VX != VY`
    SkipIfNeXY(RegId, RegId),

    // Flow
    /// `JP NNN`
    /// `PC <- NNN`
    Jump(MemAddr),

    /// `JPV0 NNN`
    /// `PC <- V0 + NNN`
    JumpV0(MemAddr),

    /// `CALL NNN`
    /// call subroutine at `NNN`
    Call(MemAddr),

    /// return from subroutine
    Ret,

    /// No-op
    /// originally `SYS NNN` (equivalent to `Jump`)
    NoOp(u16),

    /// `SKP VX`
    /// `PC <- PC + 2` if `key() = VX`
    SkipIfKeyEqX(RegId),

    /// `SKNP VX`
    /// `PC <- PC + 2` if `key() != VX`
    SkipIfKeyNeX(RegId),

    /// `LDDT VX`
    /// `VX <- DT` where `DT` is the value of delay timer
    GetDelayX(RegId),

    /// `LDK VX `
    /// `VX <- KEY` where `KEY` is current key (await until key press)
    AwaitKeyX(RegId),

    /// `STDT VX`
    /// `DT <- VX`
    SetDelayX(RegId),

    /// `STST VX`
    /// `ST <- VX` where `ST`
    SetSoundX(RegId),

    //
    /// `LD I NNN`
    /// `I <- NNN`
    SetI(MemAddr),

    /// `ADDI VX`
    /// `I <- I + VX`
    AddIX(RegId),

    /// `LDSPR VX`
    /// `I <- SPRITE(VX)` where `SPRITE()` address of sprite
    /// 'hexadecimal sprite`
    SpriteAddrIX(RegId),

    /// `STBCD VX`
    /// convert `VX` to BCD (3 bytes)
    /// write the bytes most significant digit first from `I`.
    DumpBcdIX(RegId),

    /// `STREGS VX`
    /// write values of registers from `V0` to `VX` starting at `I`.
    RegDumpIX(RegId),

    /// `LDREGS VX`
    /// read values of registers `V0` to `VX` from memory starting at `I`.
    RegLoadIX(RegId),

    /// `RND VX NN`
    /// `VX <- RAND & NN` where `RAND` is a random number (0 to 255)
    RandX(RegId, u8),

    /// halts CPU
    Halt,

    Unsupported(u16),
}

impl Instruction {
    pub fn decode(opcode: u16) -> Self {
        let op_cls = (opcode & 0xF000) >> 12;
        match op_cls {
            0x0 => {
                if opcode == 0x0000 {
                    Instruction::Halt
                } else if opcode == 0x00E0 {
                    Instruction::DispClear
                } else if opcode == 0x00EE {
                    Instruction::Ret
                } else {
                    Instruction::NoOp(opcode)
                }
            }
            0x1 => Instruction::Jump((opcode & 0x0FFF) as MemAddr),
            0x2 => Instruction::Call((opcode & 0x0FFF) as MemAddr),
            0x3 | 0x4 | 0x6 | 0x7 => {
                let x = ((opcode & 0x0F00) >> 8) as RegId;
                let imm = (opcode & 0x00FF) as u8;
                match op_cls {
                    0x3 => Instruction::SkipIfEqX(x, imm),
                    0x4 => Instruction::SkipIfNeX(x, imm),
                    0x6 => Instruction::AssignXImm(x, imm),
                    0x7 => Instruction::AddXImm(x, imm),
                    _ => unreachable!(),
                }
            }
            0x5 => {
                let x = ((opcode & 0x0F00) >> 8) as RegId;
                let y = ((opcode & 0x00F0) >> 4) as RegId;
                let op = opcode & 0x000F;
                match op {
                    0 => Instruction::SkipIfEqXY(x, y),
                    _ => Instruction::Unsupported(opcode),
                }
            }
            0x8 => {
                let x = ((opcode & 0x0F00) >> 8) as RegId;
                let y = ((opcode & 0x00F0) >> 4) as RegId;
                let op = opcode & 0x000F;
                match op {
                    0x0 => Instruction::AssignXY(x, y),
                    0x1 => Instruction::OrXY(x, y),
                    0x2 => Instruction::AndXY(x, y),
                    0x3 => Instruction::XorXY(x, y),
                    0x4 => Instruction::AddXY(x, y),
                    0x5 => Instruction::SubXY(x, y),
                    0x6 => Instruction::Shr1X(x),
                    0x7 => Instruction::SubYX(x, y),
                    0xE => Instruction::Shl1X(x),
                    _ => Instruction::Unsupported(opcode),
                }
            }
            0x9 => {
                let x = ((opcode & 0x0F00) >> 8) as RegId;
                let y = ((opcode & 0x00F0) >> 4) as RegId;
                let op = opcode & 0x000F;
                match op {
                    0 => Instruction::SkipIfNeXY(x, y),
                    _ => Instruction::Unsupported(opcode),
                }
            }
            0xA => Instruction::SetI((opcode & 0x0FFF) as MemAddr),
            0xB => Instruction::JumpV0((opcode & 0x0FFF) as MemAddr),
            0xC => {
                let x = ((opcode & 0x0F00) >> 8) as RegId;
                let imm = (opcode & 0x00FF) as u8;
                Instruction::RandX(x, imm)
            }
            0xD => {
                let x = ((opcode & 0x0F00) >> 8) as RegId;
                let y = ((opcode & 0x00F0) >> 4) as RegId;
                let imm = (opcode & 0x000F) as u8;
                Instruction::DispDraw(x, y, imm)
            }
            0xE => {
                let x = ((opcode & 0x0F00) >> 8) as RegId;
                let op = opcode & 0x00FF;
                match op {
                    0x9E => Instruction::SkipIfKeyEqX(x),
                    0xA1 => Instruction::SkipIfKeyNeX(x),
                    _ => Instruction::Unsupported(opcode),
                }
            }
            0xF => {
                let x = ((opcode & 0x0F00) >> 8) as RegId;
                let op = opcode & 0x00FF;
                match op {
                    0x07 => Instruction::GetDelayX(x),
                    0x0A => Instruction::AwaitKeyX(x),
                    0x15 => Instruction::SetDelayX(x),
                    0x18 => Instruction::SetSoundX(x),
                    0x1E => Instruction::AddIX(x),
                    0x29 => Instruction::SpriteAddrIX(x),
                    0x33 => Instruction::DumpBcdIX(x),
                    0x55 => Instruction::RegDumpIX(x),
                    0x65 => Instruction::RegLoadIX(x),
                    0x17 => Instruction::NoOp(opcode),
                    _ => match opcode {
                        0xF000 => Instruction::Halt,
                        _ => Instruction::Unsupported(opcode),
                    },
                }
            }
            _ => Instruction::Unsupported(opcode),
        }
    }
}
