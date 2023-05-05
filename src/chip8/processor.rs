use rand::prelude::*;
use std::fs;

#[derive(Clone, Copy)]
pub struct Processor {
    pub memory: [u8; Processor::MEM_SIZE as usize],
    pub registers: [u8; 16],
    pub index: u16,
    pub instruction_ptr: u16,
    pub opcode: u16,
    pub stack: [u16; 16],
    pub keypad: [bool; 16],
    pub display: [u8; 64 * 32],
    pub stack_ptr: u8,
    pub delay_timer: u8,
    pub sound_timer: u8,
}

impl Processor {
    const MEM_SIZE: u16 = 4096;
    const FONTSET_SIZE: usize = 80;
    const ROM_START_ADDR: u16 = 0x0200;
    const FONTSET_START_ADDR: u16 = 0x0050;
    const DISPLAY_WIDTH: u16 = 64;
    const DISPLAY_HEIGHT: u16 = 32;

    const FONTSET: [u8; Self::FONTSET_SIZE] = [
        0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
        0x20, 0x60, 0x20, 0x20, 0x70, // 1
        0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
        0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
        0x90, 0x90, 0xF0, 0x10, 0x10, // 4
        0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
        0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
        0xF0, 0x10, 0x20, 0x40, 0x40, // 7
        0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
        0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
        0xF0, 0x90, 0xF0, 0x90, 0x90, // A
        0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
        0xF0, 0x80, 0x80, 0x80, 0xF0, // C
        0xE0, 0x90, 0x90, 0x90, 0xE0, // D
        0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
        0xF0, 0x80, 0xF0, 0x80, 0x80, // F
    ];

    pub fn cycle(&mut self) {
        let opcode = u16::from(self.memory[self.instruction_ptr as usize]) << 8
            | u16::from(self.memory[self.instruction_ptr as usize + 1]);

        self.opcode = opcode;
        self.instruction_ptr += 2;

        // bitmasks are fucking SICK
        match opcode & 0xF000 {
            0x0000 => match opcode & 0x00FF {
                0x00E0 => self.op_00e0(),
                0x00EE => self.op_00ee(),
                0x0000 => {}
                _ => panic!("Illegal opcode: {:#06X}", opcode,),
            },

            0x1000 => self.op_1nnn(),
            0x2000 => self.op_2nnn(),
            0x3000 => self.op_3xkk(),
            0x4000 => self.op_4xkk(),
            0x5000 => self.op_5xy0(),
            0x6000 => self.op_6xkk(),
            0x7000 => self.op_7xkk(),

            0x8000 => match self.opcode & 0x000F {
                0x0000 => self.op_8xy0(),
                0x0001 => self.op_8xy1(),
                0x0002 => self.op_8xy2(),
                0x0003 => self.op_8xy3(),
                0x0004 => self.op_8xy4(),
                0x0005 => self.op_8xy5(),
                0x0006 => self.op_8xy6(),
                0x0007 => self.op_8xy7(),
                0x000E => self.op_8xye(),
                _ => panic!("Illegal opcode: {:#06X}", opcode),
            },

            0x9000 => self.op_9xy0(),
            0xA000 => self.op_annn(),
            0xB000 => self.op_bnnn(),
            0xC000 => self.op_cxkk(),
            0xD000 => self.op_dxyn(),
            0xE000 => match opcode & 0x00FF {
                0x009E => self.op_ex9e(),
                0x00A1 => self.op_exa1(),
                _ => panic!("Illegal opcode: {:#06X}", opcode),
            },

            0xF000 => match opcode & 0x00FF {
                0x0007 => self.op_fx07(),
                0x000A => self.op_fx0a(),
                0x0015 => self.op_fx15(),
                0x0018 => self.op_fx18(),
                0x001E => self.op_fx1e(),
                0x0029 => self.op_fx29(),
                0x0033 => self.op_fx33(),
                0x0055 => self.op_fx55(),
                0x0065 => self.op_fx65(),
                _ => println!("Illegal opcode: 0x{:#06X}", opcode),
            },

            _ => panic!(
                "Illegal opcode: 0x{:04X}::0x{:04X}",
                opcode,
                opcode & 0xF000
            ),
        }

        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn op_fx65(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;

        for i in 0..=register_idx {
            self.registers[i as usize] = self.memory[(self.index + i) as usize]
        }
    }

    pub fn op_fx55(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;

        for i in 0..=register_idx {
            self.memory[(self.index + i) as usize] = self.registers[i as usize];
        }
    }

    pub fn op_fx33(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;
        let mut value = self.registers[register_idx as usize];

        self.memory[self.index as usize + 2] = value % 10;
        value /= 10;

        self.memory[self.index as usize + 1] = value % 10;
        value /= 10;

        self.memory[self.index as usize] = value % 10;
    }

    pub fn op_fx29(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;
        let value = self.registers[register_idx as usize];

        self.index = Self::FONTSET_START_ADDR + (5 * value as u16);
    }

    pub fn op_fx1e(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;
        self.index += u16::from(self.registers[register_idx as usize]);
    }

    pub fn op_fx18(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;

        self.sound_timer = self.registers[register_idx as usize];
    }

    pub fn op_fx15(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;

        self.delay_timer = self.registers[register_idx as usize];
    }

    pub fn op_fx0a(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;

        if self.keypad[0] {
            self.registers[register_idx as usize] = 0;
        } else if self.keypad[1] {
            self.registers[register_idx as usize] = 1;
        } else if self.keypad[2] {
            self.registers[register_idx as usize] = 2;
        } else if self.keypad[3] {
            self.registers[register_idx as usize] = 3;
        } else if self.keypad[4] {
            self.registers[register_idx as usize] = 4;
        } else if self.keypad[5] {
            self.registers[register_idx as usize] = 5;
        } else if self.keypad[6] {
            self.registers[register_idx as usize] = 6;
        } else if self.keypad[7] {
            self.registers[register_idx as usize] = 7;
        } else if self.keypad[8] {
            self.registers[register_idx as usize] = 8;
        } else if self.keypad[9] {
            self.registers[register_idx as usize] = 9;
        } else if self.keypad[10] {
            self.registers[register_idx as usize] = 10;
        } else if self.keypad[11] {
            self.registers[register_idx as usize] = 11;
        } else if self.keypad[12] {
            self.registers[register_idx as usize] = 12;
        } else if self.keypad[13] {
            self.registers[register_idx as usize] = 13;
        } else if self.keypad[14] {
            self.registers[register_idx as usize] = 14;
        } else if self.keypad[15] {
            self.registers[register_idx as usize] = 15;
        } else {
            self.instruction_ptr -= 2;
        }
    }

    pub fn op_fx07(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;

        self.registers[register_idx as usize] = self.delay_timer;
    }

    pub fn op_exa1(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;
        let key = self.registers[register_idx as usize];

        if !self.keypad[key as usize] {
            self.instruction_ptr += 2;
        }
    }

    pub fn op_ex9e(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;
        let key = self.registers[register_idx as usize];

        if self.keypad[key as usize] {
            self.instruction_ptr += 2;
        }
    }

    pub fn op_dxyn(&mut self) {
        let vx_idx = (self.opcode & 0x0F00) >> 8;
        let vy_idx = (self.opcode & 0x00F0) >> 4;
        let height = self.opcode & 0x000F;

        let x_pos = self.registers[vx_idx as usize] % Self::DISPLAY_WIDTH as u8;
        let y_pos = self.registers[vy_idx as usize] % Self::DISPLAY_HEIGHT as u8;

        self.registers[0xF] = 0;

        for row in 0..height {
            let sprite_byte = self.memory[(self.index + row) as usize];

            for col in 0..8 {
                let sprite_pixel = sprite_byte & (0b1000_0000 >> col);

                let display_index = ((u16::from(y_pos) + row) % Self::DISPLAY_HEIGHT)
                    * Self::DISPLAY_WIDTH
                    + ((u16::from(x_pos) + col) % Self::DISPLAY_WIDTH);

                let display_pixel = &mut self.display[display_index as usize];

                if sprite_pixel != 0 {
                    if *display_pixel == 0xFF {
                        self.registers[0xF] = 1;
                    }
                    *display_pixel ^= 0xFF;
                }
            }
        }
    }

    pub fn op_cxkk(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;

        let mut rng = rand::thread_rng();
        let random_byte = rng.gen_range(u8::MIN..u8::MAX) as u8;

        self.registers[register_idx as usize] = random_byte;
    }

    pub fn op_bnnn(&mut self) {
        let addr = self.opcode & 0x0FFF;
        self.instruction_ptr = self.registers[0] as u16 + addr;
    }

    pub fn op_annn(&mut self) {
        let addr = self.opcode & 0x0FFF;
        self.index = addr;
    }

    pub fn op_8xye(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;

        self.registers[0xF] = (self.registers[register_idx as usize] & 0x80) >> 7;

        self.registers[register_idx as usize] <<= 1;
    }

    pub fn op_8xy6(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;

        self.registers[0xF] = self.registers[register_idx as usize] & 0x1;

        self.registers[register_idx as usize] >>= 1;
    }

    pub fn op_8xy7(&mut self) {
        let a_idx = (self.opcode & 0x0F00) >> 8;
        let b_idx = (self.opcode & 0x00F0) >> 4;

        if self.registers[b_idx as usize] > self.registers[a_idx as usize] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[a_idx as usize] =
            self.registers[b_idx as usize].wrapping_sub(self.registers[a_idx as usize]);
    }

    pub fn op_8xy5(&mut self) {
        let a_idx = (self.opcode & 0x0F00) >> 8;
        let b_idx = (self.opcode & 0x00F0) >> 4;

        if self.registers[a_idx as usize] > self.registers[b_idx as usize] {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[a_idx as usize] =
            self.registers[a_idx as usize].wrapping_sub(self.registers[b_idx as usize]);
    }

    pub fn op_8xy4(&mut self) {
        let a_idx = (self.opcode & 0x0F00) >> 8;
        let b_idx = (self.opcode & 0x00F0) >> 4;

        let sum =
            u16::from(self.registers[a_idx as usize]) + u16::from(self.registers[b_idx as usize]);

        if sum > 255 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }

        self.registers[a_idx as usize] = sum as u8;
    }

    pub fn op_8xy3(&mut self) {
        let a_idx = (self.opcode & 0x0F00) >> 8;
        let b_idx = (self.opcode & 0x00F0) >> 4;

        self.registers[a_idx as usize] ^= self.registers[b_idx as usize];
    }

    pub fn op_8xy2(&mut self) {
        let a_idx = (self.opcode & 0x0F00) >> 8;
        let b_idx = (self.opcode & 0x00F0) >> 4;

        self.registers[a_idx as usize] &= self.registers[b_idx as usize];
    }

    pub fn op_8xy1(&mut self) {
        let a_idx = (self.opcode & 0x0F00) >> 8;
        let b_idx = (self.opcode & 0x00F0) >> 4;

        self.registers[a_idx as usize] |= self.registers[b_idx as usize];
    }

    pub fn op_8xy0(&mut self) {
        let a_idx = (self.opcode & 0x0F00) >> 8;
        let b_idx = (self.opcode & 0x00F0) >> 4;

        self.registers[a_idx as usize] = self.registers[b_idx as usize];
    }

    pub fn op_7xkk(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;
        let byte = (self.opcode & 0x00FF) as u8;

        self.registers[register_idx as usize] =
            self.registers[register_idx as usize].wrapping_add(byte);
    }

    pub fn op_6xkk(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;
        let byte = (self.opcode & 0x00FF) as u8;

        self.registers[register_idx as usize] = byte;
    }

    pub fn op_9xy0(&mut self) {
        let a_idx = (self.opcode & 0x0F00) >> 8;
        let b_idx = (self.opcode & 0x00F0) >> 4;

        if self.registers[a_idx as usize] != self.registers[b_idx as usize] {
            self.instruction_ptr += 2;
        }
    }

    pub fn op_5xy0(&mut self) {
        let a_idx = (self.opcode & 0x0F00) >> 8;
        let b_idx = (self.opcode & 0x00F0) >> 4;

        if self.registers[a_idx as usize] == self.registers[b_idx as usize] {
            self.instruction_ptr += 2;
        }
    }

    pub fn op_4xkk(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;
        let byte = (self.opcode & 0x00FF) as u8;

        if self.registers[register_idx as usize] != byte {
            self.instruction_ptr += 2;
        }
    }

    pub fn op_3xkk(&mut self) {
        let register_idx = (self.opcode & 0x0F00) >> 8;
        let byte = (self.opcode & 0x00FF) as u8;

        if self.registers[register_idx as usize] == byte {
            self.instruction_ptr += 2;
        }
    }

    pub fn op_2nnn(&mut self) {
        let address = self.opcode & 0x0FFF;

        self.stack[self.stack_ptr as usize] = self.instruction_ptr;
        self.stack_ptr += 1;
        self.instruction_ptr = address;
    }

    pub fn op_1nnn(&mut self) {
        let address = self.opcode & 0x0FFF;
        self.instruction_ptr = address;
    }

    pub fn op_00ee(&mut self) {
        self.stack_ptr -= 1;
        self.instruction_ptr = self.stack[self.stack_ptr as usize];
    }

    pub fn op_00e0(&mut self) {
        self.display = [0u8; 64 * 32];
    }

    pub fn init(&mut self) {
        for i in 0..Self::FONTSET_SIZE {
            self.memory[Self::FONTSET_START_ADDR as usize + i] = Self::FONTSET[i];
        }
    }

    pub fn load_rom(&mut self, file_path: &str) {
        let rom_data = fs::read(file_path).expect("Failed to read ROM provided");
        let rom_len = rom_data.len();
        let rom_end_addr = Self::ROM_START_ADDR + rom_len as u16;

        if rom_end_addr > Self::MEM_SIZE {
            panic!("Failed to load ROM because file is too large");
        }

        for (i, &byte) in rom_data.iter().enumerate() {
            self.memory[Self::ROM_START_ADDR as usize + i] = byte;
        }
    }

    pub fn new() -> Self {
        Self {
            memory: [0u8; Self::MEM_SIZE as usize],
            registers: [0; 16],
            stack: [0; 16],
            keypad: [false; 16],
            display: [0u8; 64 * 32],
            index: 0,
            opcode: 0,
            instruction_ptr: Self::ROM_START_ADDR,
            stack_ptr: 0,
            delay_timer: 0,
            sound_timer: 0,
        }
    }

    pub fn debug_print(&self) {
        println!("=== Debug ===");
        println!("Memory:");
        for i in 0..Self::MEM_SIZE as usize / 16 {
            let addr = i * 16;
            print!("{:#06x}: ", addr);
            for j in 0..16 {
                print!("{:02x} ", self.memory[addr + j]);
            }
            println!();
        }
        println!();
        print!("Registers: [");
        for r in self.registers {
            print!("{}, ", r);
        }
        println!("]");

        println!("Index: {:#06X}", self.index);
        println!("Instruction Pointer: {:#06X}", self.instruction_ptr);

        println!("Stack:");

        print!("[");
        for r in self.stack {
            print!("{:#06X}, ", r);
        }
        print!("]");
        println!();

        println!("Stack Pointer: {:#06X}", self.stack_ptr);
        println!("Delay Timer: {}", self.delay_timer);
        println!("Sound Timer: {}", self.sound_timer);
        println!("Keypad:");
        print!("[");
        for r in self.keypad {
            print!("{}", if r { 1 } else { 0 });
        }
        println!("]");
        println!("====================");
    }
}
