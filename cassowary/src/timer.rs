use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;

use crossbeam_channel::{select, tick, unbounded, Receiver, Sender};
use std::thread;
use std::time::{Duration, Instant};

const TIMER_TICK: Duration = Duration::from_millis(16);

pub(crate) struct Timer {
    ticks: AtomicU8,
    updates_tx: Sender<u8>,
    updates_rx: Receiver<u8>,
    ticker: Receiver<Instant>,
    changed: Option<Sender<u8>>,
}

impl Timer {
    pub(crate) fn start_new(tick_len: Duration, changed: Option<Sender<u8>>) -> Arc<Self> {
        let timer = Arc::new(Timer::new(tick_len, changed));
        {
            let timer = Arc::clone(&timer);
            thread::spawn(move || timer.tick_loop());
        }
        timer
    }

    pub(crate) fn new(tick_len: Duration, changed: Option<Sender<u8>>) -> Self {
        let (updates_tx, updates_rx) = unbounded();
        Self {
            ticks: AtomicU8::new(0),
            ticker: tick(tick_len),
            updates_tx,
            updates_rx,
            changed,
        }
    }

    pub fn tick_loop(&self) {
        loop {
            select! {
                recv(self.ticker) -> _tick => {
                    let ticks = self.load_tick();
                    if ticks > 0 {
                        self.store_tick(ticks.saturating_sub(1));
                    }
                }
                recv(self.updates_rx) -> new_ticks => {
                    if let Ok(new_ticks) = new_ticks {
                        self.store_tick(new_ticks);
                    }
                }
            }
        }
    }

    fn load_tick(&self) -> u8 {
        self.ticks.load(Ordering::Acquire)
    }

    fn store_tick(&self, updated: u8) {
        self.ticks.store(updated, Ordering::Release);
        if let Some(ref changed) = self.changed {
            changed.send(self.ticks.load(Ordering::SeqCst)).unwrap();
        }
    }

    pub fn set(&self, ticks: u8) {
        self.updates_tx.send(ticks).unwrap();
    }

    pub fn get(&self) -> u8 {
        self.ticks.load(Ordering::Relaxed)
    }
}

pub struct DelayTimer(Arc<Timer>);

impl DelayTimer {
    pub fn start_new() -> Self {
        Self(Timer::start_new(TIMER_TICK, None))
    }

    pub fn get(&mut self) -> u8 {
        self.0.get()
    }

    pub fn set(&mut self, value: u8) {
        self.0.set(value);
    }
}
