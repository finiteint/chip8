use crate::memory::{Memory, MemoryError};

pub fn load_from_hex(hex_def: &str, mem: &mut Memory) -> Result<(), MemoryError> {
    for (addr, data) in hex_to_bin(hex_def) {
        mem.set_mem_from(addr as usize, &data)?;
    }
    Ok(())
}

fn hex_to_bin(hex_def: &str) -> Vec<(u16, Vec<u8>)> {
    hex_def
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() || line.starts_with("#") {
                return None;
            }

            let parts: Vec<_> = line.split_whitespace().collect();
            if parts.len() < 2 {
                eprintln!("Bogus line: {}", line);
                return None;
            }

            if let Some(addr) = hex_to_u16(parts[0]) {
                let data: String = (&parts[1..]).into_iter().map(|&x| x).collect();
                if data.len() % 2 != 0 {
                    eprintln!("Bogus line: {}", line);
                    return None;
                }
                let nibbles: Vec<u8> = data.bytes().map(|c| hex_to_u8(c)).collect();

                let mut bytes: Vec<u8> = Vec::with_capacity(nibbles.len() / 2);
                for byte in nibbles.chunks_exact(2) {
                    if let &[msn, lsn] = byte {
                        if msn > 15 || lsn > 15 {
                            eprintln!("Bogus line: {}", line);
                            return None;
                        }
                        bytes.push((msn << 4) | lsn);
                    }
                }
                Some((addr, bytes))
            } else {
                eprintln!("Bogus line: {}", line);
                None
            }
        })
        .collect()
}

fn hex_to_u16(hex: &str) -> Option<u16> {
    if hex.len() != 4 {
        return None;
    }
    let mut value = 0x0000_u16;
    for (shift, nibble) in hex.bytes().rev().map(|c| hex_to_u8(c)).enumerate() {
        if nibble > 15 {
            return None;
        }
        value |= (nibble as u16) << (shift * 4);
    }
    Some(value)
}

/// Note: returns `16` on invalid input
fn hex_to_u8(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => (c - b'0'),
        b'A'..=b'F' => 10 + c - b'A',
        b'a'..=b'f' => 10 + c - b'a',
        _ => 16,
    }
}
