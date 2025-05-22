use bit_vec::BitVec;
use grid::*;
use raylib::prelude::*;
use std::ops::Range;
use std::{
    fs,
    io::{self, Error},
    ops::{RangeBounds, RangeToInclusive},
};

use sdl3::event::Event;
use sdl3::keyboard::Keycode;
use sdl3::pixels::Color;
use std::time::Duration;

#[derive(Clone, Copy)]
struct Point {
    pub x: u8,
    pub y: u8,
}

//font byte representation
// 0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
// 0x20, 0x60, 0x20, 0x20, 0x70, // 1
// 0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
// 0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
// 0x90, 0x90, 0xF0, 0x10, 0x10, // 4
// 0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
// 0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
// 0xF0, 0x10, 0x20, 0x40, 0x40, // 7
// 0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
// 0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
// 0xF0, 0x90, 0xF0, 0x90, 0x90, // A
// 0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
// 0xF0, 0x80, 0x80, 0x80, 0xF0, // C
// 0xE0, 0x90, 0x90, 0x90, 0xE0, // D
// 0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
// 0xF0, 0x80, 0xF0, 0x80, 0x80  // F

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
    // hardware components
    let mut pc: usize = 0;
    let mut screen_grid: Grid<bool> = Grid::new(64, 32);
    let mut ram: [u8; 4000] = [0; 4000];
    let mut stack: Vec<u16> = Vec::new();
    let mut rom: Vec<u8> = fs::read("roms/ibm.ch8")?;

    // rendering
    let pixel_scale = 8;
    let (mut rl, thread) = raylib::init()
        .size(64 * pixel_scale, 32 * pixel_scale)
        .title("chipple")
        .build();
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
        let instruction_slice = fetch(pc, &rom);
        pc += 2;
        match instruction_slice {
            Some(value) => {
                let instruction = decode(value);
                if instruction.is_err() {
                    panic!("fakeass instruction")
                }
                execute(instruction.unwrap());
            }
            None => {
                println!("we've reached the end of the rom.");
                panic!("balls");
            }
        }
        //        execute(instruction);
    }

    Ok(())
}

fn fetch(pc: usize, rom: &[u8]) -> Option<&[u8]> {
    if pc >= rom.len() - 1 {
        None
    } else {
        Some(rom.get(pc..=pc + 1)?)
    }
}

fn decode(instruction_bytes: &[u8]) -> Result<Instruction, &'static str> {
    /*
    println!(
        "{:#01x} | {:#01x} \n",
        instruction_bytes.get(0)?,
        instruction_bytes.get(1)?
    );
    */

    use Instruction::*;
    // let first_byte = instruction_bytes.first()?;
    // let second_byte = instruction_bytes.last()?;
    let bit_vec = BitVec::from_bytes(instruction_bytes);

    match hex_digit(bit_vec.clone(), 0) {
        0x0 => match hex_digit(bit_vec.clone(), 3) {
            //0xEE
            0xE => Ok(Return),
            //0xE0
            0x0 => Ok(ClearScreen),
            _ => Err("invalid instruction"),
        },
        0x1 => Ok(Jump(bits_to_value(
            bit_vec.clone(),
            Range { start: 4, end: 16 },
        ))),
        0x2 => Ok(SubRoutine(bits_to_value(
            bit_vec.clone(),
            Range { start: 4, end: 16 },
        ))),
        0x6 => Ok(Set {
            value: (bits_to_value(bit_vec.clone(), Range { start: 8, end: 16 }) as u8),
            address: hex_digit(bit_vec.clone(), 1),
        }),
        0x7 => Ok(Add {
            value: (bits_to_value(bit_vec.clone(), Range { start: 8, end: 16 }) as u8),
            address: hex_digit(bit_vec.clone(), 1),
        }),
        _ => Err("invalid instruction"),
    }
}

fn bits_to_value(mut bits: BitVec, range: Range<usize>) -> u16 {
    // remove bits before and after range
    bits.split_off(range.end);
    let split_bits = bits.split_off(range.start);

    let scaled_range = Range {
        start: 0,
        end: range.end - range.start,
    };
    let end = scaled_range.end - 1;

    let mut res: u16 = 0;
    for i in scaled_range.rev() {
        // skip 0s
        if !split_bits.get(end - i).unwrap() {
            continue;
        }
        // add 2^i to the value
        let val = u16::pow(2, (i) as u32);
        res += val;
    }
    res
}

fn hex_digit(bits: BitVec, offset: u8) -> u8 {
    //returns the indivisual hex digit at that position
    // FF => 255 => 1111 1111
    match offset {
        0 => bits_to_value(bits, Range { start: 0, end: 4 }) as u8,
        1 => bits_to_value(bits, Range { start: 4, end: 8 }) as u8,
        2 => bits_to_value(bits, Range { start: 8, end: 12 }) as u8,
        3 => bits_to_value(bits, Range { start: 12, end: 16 }) as u8,
        _ => panic!("invalid offset idiot"),
    }
}

fn execute(instruction: Instruction) {
    use Instruction::*;
    match Instruction {
        ClearScreen => clear_screen(),
        _ => println!("unimplemented"),
    }
}

fn clear_screen() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bit_2_val_tests() {
        let result = bits_to_value(BitVec::from_bytes(&[245]), Range { start: 4, end: 8 });
        assert_eq!(result, 5);
        let result = bits_to_value(BitVec::from_bytes(&[240]), Range { start: 1, end: 7 });
        assert_eq!(result, 56);
    }
    #[test]
    fn bits_2_hex_tests() {
        let result = hex_digit(BitVec::from_bytes(&[0xEE, 0xDA]), 3);
        assert_eq!(result, 0xA);
    }
}
