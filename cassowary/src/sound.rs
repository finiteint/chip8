use std::time::Duration;
use std::{sync::Arc, thread};

use crossbeam_channel::unbounded;
use rodio::{self, source::SineWave};
use thiserror::Error;

use crate::timer::Timer;

const TIMER_TICK: Duration = Duration::from_millis(16);

#[derive(Error, Debug)]
pub enum SoundError {
    #[error("error setting up sound: {0}")]
    SetupError(String),
}

pub struct SoundSystem {
    timer: Arc<Timer>,
}

impl SoundSystem {
    pub fn start_new() -> Result<Self, SoundError> {
        let tone = setup_tone(440)?;
        let (changed_tx, changed_rx) = unbounded();
        thread::spawn(move || {
            let mut playing = false;
            for change in changed_rx {
                if change == 0 {
                    if playing {
                        tone.pause();
                        playing = false;
                        println!("BEEP.end.");
                    }
                } else {
                    if !playing {
                        tone.play();
                        playing = true;
                        println!("BEEP.start.");
                    }
                }
            }
        });
        let timer = Timer::start_new(TIMER_TICK, Some(changed_tx));
        Ok(Self { timer })
    }

    pub fn set_timer(&mut self, value: u8) {
        self.timer.set(value);
    }
}

fn setup_tone(tone_hz: u32) -> Result<rodio::Sink, SoundError> {
    let (_stream, stream_handle) = rodio::OutputStream::try_default()
        .map_err(|err| SoundError::SetupError(err.to_string()))?;
    let sink = rodio::Sink::try_new(&stream_handle)
        .map_err(|err| SoundError::SetupError(err.to_string()))?;
    sink.pause();
    sink.append(SineWave::new(tone_hz));
    sink.set_volume(0.9);
    Ok(sink)
}
