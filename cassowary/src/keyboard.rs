pub struct KeyBoard {
    pressed: Option<u8>,
}

impl KeyBoard {
    pub fn new() -> Self {
        Self {
            pressed: Some(b'q'),
        }
    }

    pub fn press(&mut self, key: u8) {
        self.pressed = Some(key);
    }

    pub(crate) fn await_key_press(&mut self) -> u8 {
        loop {
            if let Some(key) = self.pressed.take() {
                return key;
            }
        }
    }

    pub(crate) fn get_key_pressed(&self) -> u8 {
        self.pressed.unwrap_or(b'\0')
    }
}
