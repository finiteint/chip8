use std::process;

use cassowary::progloader;
use cassowary::{Memory, MemoryError, System};

fn load_firmware(mem: &mut Memory) -> Result<(), MemoryError> {
    progloader::load_from_hex(include_str!("firmware.mem"), mem)
}

fn hex_to_decimal(mem: &mut Memory) -> Result<(), MemoryError> {
    // This is an example from The CHIP-8 Classic Manual
    // http://www.CHIP-8.com/
    // hex to decimal
    progloader::load_from_hex(
        "0000   1200
         0200   00E0 6380 6400 6500 A500 F333 F265 F029
         0210   D455 F129 7408 D455 F229 7408 D455 F000
        ",
        mem,
    )
}

fn double_sum(mem: &mut Memory) -> Result<(), MemoryError> {
    // This is an example from Tim McNamara's "Rust in Action"
    progloader::load_from_hex(
        "0200   2100 2100 0000
         0100   8014 8014 00EE
        ",
        mem,
    )
}

fn display_a_hex_sprite(mem: &mut Memory) -> Result<(), MemoryError> {
    // display a hex sprite
    progloader::load_from_hex(
        "# Display 'E'
         # LDR $1 3; LDR $2 2; LDR $3 E; LDSPR $3; DISP $1 $2 5
         0200   6103 6202 630E F329 D125
        ",
        mem,
    )
}

fn timer_display_sprites(mem: &mut Memory) -> Result<(), MemoryError> {
    // play sound for c. 1 s and display sprites with delay
    progloader::load_from_hex(
        "# LDR $3 @60; SETSOUND $3;
         0200  633C F318
         # LDR $7 A; CALL 300  
         0204  670A 2300
         # LDR $4 @30; SETDELAY $4;
         0208  641E F415
         # GETDELAY $4; SKIPEQ $4, 0 ; JMP 20C
         020C  F407 3400 120C 
         # ADDI $7 1; CALL 300
         0212  7701 2300

         # DISPCLR; LDR 5 3; LDR 6 2; LDSPR 7; DISP 5 6 5; RET;
         0300 00E0 6503 6602 F729 D565 00EE
        ",
        mem,
    )
}

fn scratch(mem: &mut Memory) -> Result<(), MemoryError> {
    // random scratch
    progloader::load_from_hex(
        "# LDR $3 @10; SETSOUND $3;
         0200  630A F318
         0204  7A01 1204
        ",
        mem,
    )
}

fn load_program(mem: &mut Memory) -> Result<(), MemoryError> {
    match 2 {
        1 => hex_to_decimal(mem),
        2 => scratch(mem),
        3 => display_a_hex_sprite(mem),
        4 => timer_display_sprites(mem),
        _ => double_sum(mem),
    }
}

fn main() {
    println!("Cassowary - A Dodgy & Shoddy CHIP-8 Emulator");
    let mut system = System::new().expect("setup failed");
    {
        let mem = system.memory_mut();
        load_firmware(mem).unwrap();
        load_program(mem).unwrap()
    }

    if let Err(err) = system.run() {
        eprintln!("ERROR: {}", err);
        process::exit(1);
    }

    system.cpu().dump();
    system.memory().dump();
}
