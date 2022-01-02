mod cpu;
mod display;
mod instructions;
mod keyboard;
mod memory;
pub mod progloader;
mod sound;
mod timer;

pub use crate::cpu::{Cpu, CpuError};
pub use crate::display::Display;
pub use crate::keyboard::KeyBoard;
pub use crate::memory::{Memory, MemoryError};
pub use crate::sound::{SoundError, SoundSystem};
pub use crate::timer::DelayTimer;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SystemError {
    #[error("sound system error: {0}")]
    SoundError(#[from] SoundError),
}

pub struct System {
    cpu: Cpu,
    mem: Memory,
    delay: DelayTimer,
    sound: SoundSystem,
    display: Display,
    keyboard: KeyBoard,
}

impl System {
    pub fn new() -> Result<Self, SystemError> {
        let sound = SoundSystem::start_new()?;
        let delay = DelayTimer::start_new();
        Ok(Self {
            cpu: Cpu::new(),
            mem: Memory::new(),
            delay,
            sound,
            display: Display::new(),
            keyboard: KeyBoard::new(),
        })
    }

    pub fn run(&mut self) -> Result<(), CpuError> {
        self.cpu.run(
            &mut self.mem,
            &mut self.delay,
            &mut self.display,
            &mut self.keyboard,
            &mut self.sound,
        )
    }

    pub fn memory(&mut self) -> &Memory {
        &self.mem
    }

    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mem
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }
}
