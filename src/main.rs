use bit_vec::BitVec;
use grid::*;
use std::collections::HashSet;
use std::ops::Range;
use std::{
    fs,
    io::{self, Error},
    ops::{RangeBounds, RangeToInclusive},
};

use macroquad::prelude::*;

#[derive(Clone, Copy)]
struct Point {
    pub x: u8,
    pub y: u8,
}
// fn conf() -> Conf {
//     Conf {
//         window_title: String::from("Macroquad"),
//         window_width: 1260,
//         window_height: 768,
//         fullscreen: false,
//         ..Default::default()
//     }
// }
//
// #[macroquad::main(conf)]
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

trait UpdateDisplay {
    fn update_display(&mut self, x: usize, y: usize, state: bool) {}
    fn render_display(&self, pixel_scale: usize) {}
}

impl UpdateDisplay for Grid<bool> {
    fn update_display(&mut self, x: usize, y: usize, state: bool) {
        let row = self.remove_row(x);
        let mut row = row.unwrap();
        row[y] = state;
        self.insert_row(x, row);
    }
    fn render_display(&self, pixel_scale: usize) {
        for (point, on) in self.indexed_iter() {
            //println!("pixel at {}x{} is {}", point.0, point.1, on);
            draw_rectangle(
                (point.0 * pixel_scale) as f32,
                (point.1 * pixel_scale) as f32,
                pixel_scale as f32,
                pixel_scale as f32,
                match on {
                    true => WHITE,
                    false => BLACK,
                },
            );
        }
    }
}

impl Cpu {
    fn fetch(&self) -> Option<&[u8]> {
        // appeasing the borrow checker
        let pc = self.pc - 2;
        print!("pc = {:#04X} ", pc);
        if pc >= self.rom.len() - 1 {
            None
        } else {
            Some(self.rom.get(pc..=pc + 1)?)
        }
    }
    // we don't really need the cpu to implement decode because it's a static function
    fn execute(&mut self, instruction: Instruction) {
        use Instruction::*;
        match instruction {
            ClearScreen => {
                self.screen_grid = Grid::new(64, 32);
            }
            Jump(addr) => {
                println!("setting pc to {:#04X}", addr);
                self.pc = addr as usize;
            }
            Add { value, address } => {
                self.registers[address as usize] += value;
            }
            Set { value, address } => {
                self.registers[address as usize] = value;
            }
            SetIReg(value) => {
                self.index_register = value;
            }
            Draw {
                addrx,
                addry,
                height,
            } => {
                let mut x = self.registers[addrx as usize] % 64;
                let mut y = self.registers[addry as usize] % 32;
                let mut vf = 0;
                for i in 0..=height {
                    let byte = self.ram[self.index_register as usize + i as usize];
                    let bit_vec = BitVec::from_bytes(&[byte]);
                    for bit in bit_vec.iter() {
                        if bit && *self.screen_grid.get(y, x).unwrap() {
                            self.screen_grid
                                .update_display(x as usize, y as usize, false);
                            vf = 1;
                        } else if bit && !self.screen_grid.get(y, x).unwrap() {
                            self.screen_grid
                                .update_display(x as usize, y as usize, true);
                        }
                        x += 1;
                    }
                    y += 1;
                }
            }
            _ => panic!("didn't implement this yet."),
        }
    }
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

#[derive(Clone)]
struct Cpu {
    screen_grid: Grid<bool>,
    pc: usize,
    registers: [u8; 16],
    ram: [u8; 4000],
    stack: Vec<u16>,
    index_register: u16,
    rom: Vec<u8>,
}

#[macroquad::main("chip-8")]
async fn main() -> Result<(), std::io::Error> {
    // hardware components
    let mut cpu = Cpu {
        pc: 0,
        screen_grid: Grid::new(64, 32),
        registers: [0; 16],
        ram: [0; 4000],
        stack: Vec::new(),
        index_register: 0,
        rom: fs::read("roms/ibm.ch8")?,
    };
    // rendering
    clear_background(BLACK);
    let pixel_scale: usize = 16;
    println!("{:?}", cpu.rom);
    // main loop

    'running: loop {
        request_new_screen_size(64. * pixel_scale as f32, 32. * pixel_scale as f32);
        cpu.screen_grid.render_display(pixel_scale);
        cpu.pc += 2;
        let instruction_slice = cpu.fetch();
        match &instruction_slice {
            Some(value) => {
                println!(
                    "INSTRUCTION AT {:#04X}: {:#04X},{:#04X}",
                    cpu.pc - 2,
                    value.first().unwrap(),
                    value.last().unwrap()
                );
                let instruction = decode(value);
                if instruction.is_err() {
                    panic!("fakeass instruction")
                }
                cpu.execute(instruction.unwrap());
            }
            None => {
                println!("we've reached the end of the rom.");
                panic!("balls")
            }
        }
        if is_key_pressed(KeyCode::Escape) {
            break 'running;
        }
        next_frame().await
    }
    Ok(())
}

fn decode(instruction_bytes: &[u8]) -> Result<Instruction, &'static str> {
    use Instruction::*;
    let bit_vec = BitVec::from_bytes(instruction_bytes);

    match hex_digit(bit_vec.clone(), 0) {
        0x0 => match hex_digit(bit_vec.clone(), 3) {
            //0xEE
            0xE => Ok(Return),
            //0xE0
            0x0 => Ok(ClearScreen),
            _ => Err("invalid 0x0 instruction"),
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
        0xA => Ok(SetIReg(bits_to_value(
            bit_vec.clone(),
            Range { start: 4, end: 16 },
        ))),
        0xD => Ok(Draw {
            addrx: hex_digit(bit_vec.clone(), 1) as u16,
            addry: hex_digit(bit_vec.clone(), 2) as u16,
            height: hex_digit(bit_vec.clone(), 3),
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
