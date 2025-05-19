use grid::*;
use raylib::prelude::*;
use std::fs;

#[derive(Clone, Copy)]
struct Point {
    pub x: u8,
    pub y: u8,
}

enum Instruction {
    ClearScreen,
    Jump(u16),
    SubRoutine(u16),
    Return,
    CompareReg {
        addrx: u8,
        addrb: u8,
        want_equal: bool,
    },
    CompareImm {
        addrx: u8,
        imm: u8,
        want_equal: bool,
    },
    Add {
        value: u8,
        address: u8,
    },
    Set {
        value: u8,
        address: u8,
    },
    SetIReg(u16),
    Draw {
        addrx: u16,
        addry: u16,
        height: u8,
    },
}

fn main() -> Result<(), std::io::Error> {
    let mut pc: usize = 0;
    let mut screen_grid: Grid<u8> = Grid::new(64, 32);

    let mut stack: Vec<u16> = Vec::new();
    let pixel_scale = 8;
    let (mut rl, thread) = raylib::init()
        .size(64 * pixel_scale, 32 * pixel_scale)
        .title("chipple")
        .build();
    let rom: Vec<u8> = fs::read("roms/ibm.ch8")?;
    println!("{:?}", rom);
    // main loop

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        let mut framebuffer = d.load_render_texture(&thread, 64, 32);
        d.begin_texture_mode(&thread, &mut *framebuffer.unwrap());
        d.clear_background(Color::BLACK);
        d.draw_rectangle(
            16 * pixel_scale,
            16 * pixel_scale,
            pixel_scale,
            pixel_scale,
            Color::WHITE,
        );
        let instruction_bytes: usize = fetch(pc.clone(), &rom);
        pc += 2;
        let instruction = decode(instruction_bytes);
        //        execute(instruction);
    }

    Ok(())
}

fn fetch(pc: usize, rom: &Vec<u8>) -> usize {
    return (rom[pc] + rom[pc + 1]) as usize;
}

fn decode(instruction_bytes: usize) -> Instruction {
    Instruction::Return
}

fn execute(instruction: Instruction) {
    use Instruction::*;
    //match Instruction {}
}
