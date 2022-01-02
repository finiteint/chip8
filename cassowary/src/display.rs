use crate::{instructions::MemAddr, memory::MemoryError, Memory};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct Display([[u8; WIDTH]; HEIGHT]);

impl Display {
    pub fn new() -> Self {
        Self([[0; WIDTH]; HEIGHT])
    }

    pub fn clear(&mut self) {
        for row in &mut self.0 {
            row.fill(0);
        }
        self.refresh();
    }

    pub(crate) fn draw(
        &mut self,
        x: u8,
        y: u8,
        height: u8,
        start: MemAddr,
        mem: &Memory,
    ) -> Result<bool, MemoryError> {
        let x = x as usize % WIDTH;
        let y = y as usize % HEIGHT;
        let mut changed = false;
        for (ri, addr) in (start..(start + height as MemAddr)).enumerate() {
            let sprite_line = mem.load_byte(addr)?.reverse_bits();
            let row = (y + ri) % HEIGHT;
            for ci in 0..8 {
                let col = (x + ci) % WIDTH;
                let pixel = (sprite_line >> ci) & 0x01;
                let old = self.0[row][col] == 1;
                (&mut self.0[row])[col] = pixel;
                if !changed {
                    changed = old ^ (pixel == 1);
                }
            }
        }
        self.refresh();
        Ok(changed)
    }

    fn refresh(&self) {
        let border: String = std::iter::repeat('-').take(64).collect();
        println!("/{}\\", border);
        for row in self.0 {
            print!("|");
            for col in row {
                print!("{}", if col == 0 { ' ' } else { '*' });
            }
            println!("|");
        }
        println!("\\{}/", border);
    }
}
