use thiserror::Error;

use crate::instructions::MemAddr;

#[derive(Error, Debug)]
pub enum MemoryError {
    #[error("Out of Bounds")]
    OutOfBounds,
}

pub struct Memory([u8; 4096]);

impl Memory {
    pub fn new() -> Self {
        Self([0; 4096])
    }

    pub fn dump(&self) {
        const BLOCK: usize = 16;
        let mut skipped = false;
        println!("Memory:");
        for (addr, block) in (0..self.0.len()).step_by(BLOCK).zip(self.0.chunks(BLOCK)) {
            if block.iter().copied().all(|x| x == 0) {
                skipped = true;
            } else {
                if skipped {
                    println!("   ...");
                }
                print!(" {:03X}:", addr);
                for (i, b) in block.into_iter().enumerate() {
                    if i % 2 == 0 {
                        print!(" ");
                    }
                    if i % 4 == 0 && i != 0 {
                        print!("  {:03X}: ", addr + (i / 4) * 4);
                    }
                    print!("{:02X}", b);
                }
                println!();
                skipped = false;
            }
        }
    }

    pub fn set_mem_from(&mut self, start: MemAddr, data: &[u8]) -> Result<(), MemoryError> {
        if start >= self.0.len() || (start + data.len()) > self.0.len() {
            return Err(MemoryError::OutOfBounds);
        }
        self.0[start..(start + data.len())].copy_from_slice(data);
        Ok(())
    }

    pub(crate) fn load_u16(&self, addr: MemAddr) -> Result<u16, MemoryError> {
        if addr >= self.0.len() {
            return Err(MemoryError::OutOfBounds);
        }

        let high_byte = self.0[addr as usize] as u16;
        let low_byte = self.0[addr as usize + 1] as u16;
        Ok((high_byte << 8) | low_byte)
    }

    pub(crate) fn store_byte(&mut self, addr: MemAddr, value: u8) -> Result<(), MemoryError> {
        if addr >= self.0.len() {
            return Err(MemoryError::OutOfBounds);
        }

        self.0[addr] = value;
        Ok(())
    }

    pub(crate) fn load_byte(&self, addr: MemAddr) -> Result<u8, MemoryError> {
        if addr >= self.0.len() {
            return Err(MemoryError::OutOfBounds);
        }

        Ok(self.0[addr])
    }
}
