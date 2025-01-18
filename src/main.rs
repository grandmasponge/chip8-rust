use std::{fs::File, io::{stdin, stdout, BufReader, Read, Write}, thread::sleep, time::Duration};

use minifb::{Key, Window, WindowOptions};
use rand::prelude::*;

const fonts: [u8; 80] = [
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

struct Chip8 {
    register: [u8;16],
    delay_timer: u8,
    sound_timer: u8,
    keypad: [u8;16],
    index: u16,
    pc: u16,
    stack: [u16;16],
    stack_pointer: u8,
    memory: [u8;4096],
   pub video_memory: Vec<Vec<u32>>,
    current_op: u16,
    rng: ThreadRng
}

impl Chip8 {

    pub fn init() -> Self{
        let mut memory = [0;4096];
        let rng = rand::thread_rng();
        for i in 0..fonts.len() {
            memory[0x50 + i] = fonts[i]; 
        }

        Self { register: [0;16], delay_timer: 255, sound_timer: 255, keypad: [0;16], index: 0, pc: 0x200, stack: [0;16], stack_pointer: 0, memory, video_memory: vec![vec![0; 64]; 32], current_op: 0, rng }
    }

    pub fn cycle(&mut self) {
        
        let first_byte = self.memory[self.pc as usize];
        let second_byte = self.memory[(self.pc + 1) as usize];
        self.current_op = ((first_byte as u16) << 8) | (second_byte as u16);     
        self.pc += 2;
    
        match (self.current_op & 0xF000) >> 12 {
            0x0 => {
                match self.current_op & 0x00FF {
                    0x00E0 => self.OP_00E0(),
                    0x00EE => self.OP_00EE(),
                    _ => println!("Ignoring 0x0NNN instruction: {:#06x}", self.current_op)
                }
            }
            0x1 => self.OP_1nnn(),
            0x2 => self.OP_2nnn(),
            0x3 => self.OP_3xkk(),
            0x4 => self.OP_4xkk(),
            0x5 => self.OP_5xy0(),
            0x6 => self.OP_6xkk(),
            0x7 => self.OP_7xkk(),
            0x8 => {
                match self.current_op & 0x000F {
                    0x0 => self.OP_8xy0(),
                    0x1 => self.OP_8xy1(),
                    0x2 => self.OP_8xy2(),
                    0x3 => self.OP_8xy3(),
                    0x4 => self.OP_8xy4(),
                    0x5 => self.OP_8xy5(),
                    0x6 => self.OP_8xy6(),
                    0x7 => self.OP_8xy7(),
                    0xE => self.OP_8xye(),
                    _ => println!("Unknown 8 opcode: {:#06x}", self.current_op)
                }
            }
            0x9 => self.OP_9xy0(),
            0xA => self.OP_Annn(),
            0xB => self.OP_Bnnn(),
            0xC => self.OP_Cxkk(),
            0xD => {
                self.OP_Dxyn();
            },
            0xE => {
                match self.current_op & 0x00FF {
                    0x9E => self.OP_Ex9E(),
                    0xA1 => self.OP_EXA1(),
                    _ => println!("Unknown E opcode: {:#06x}", self.current_op)
                }
            }
            0xF => {
                match self.current_op & 0x00FF {
                    0x07 => self.OP_Fx07(),
                    0x0A => self.OP_Fx0A(),
                    0x15 => self.OP_Fx15(),
                    0x18 => self.OP_Fx18(),
                    0x1E => self.OP_Fx1E(),
                    0x29 => self.OP_Fx29(),
                    0x33 => self.OP_Fx33(),
                    0x55 => self.OP_Fx55(),
                    0x65 => self.OP_Fx56(),
                    _ => println!("Unknown F opcode: {:#06x}", self.current_op)
                }
            }
            _ => println!("Unknown opcode family: {:#06x}", self.current_op)
        }

        if self.delay_timer > 0{
            self.delay_timer-=1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
        }
    }

    pub fn load_rom(&mut self, filename: String) {
        let file = File::open(filename).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut  buffer = Vec::new();
        let huh = buf_reader.read_to_end(&mut buffer).unwrap();
        println!("read {huh} bytes into the chip8");
        for i in 0..buffer.len() {
            self.memory[0x200 + i] = buffer[i];
        }
    }

    pub fn OP_00E0(&mut self) {
        for y in 0..32 {
            for x in 0..64 {
                self.video_memory[y][x] = 0;
            }
        }
    }

    pub fn OP_00EE(&mut self) {
        self.stack_pointer-=1;
        self.pc = self.stack[self.stack_pointer as usize];
    }

    pub fn OP_1nnn(&mut self) {
        let address: u16 = (self.current_op & 0x0FFF);
        self.pc = address;
    }

    pub fn OP_2nnn(&mut self) {
        let address: u16 = self.current_op & 0x0FFF;
        self.stack[self.stack_pointer as usize] = self.pc;
        self.stack_pointer += 1;
        self.pc = address;
    }

    pub fn OP_3xkk(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let byte: u8 = (self.current_op & 0x00FF) as u8;

        if self.register[Vx as usize] == byte {
            self.pc += 2;
        }
    }

    pub fn OP_4xkk(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let byte: u8 = (self.current_op & 0x00FF) as u8;

        if self.register[Vx as usize] != byte {
            self.pc += 2;
        }
    }

    pub fn OP_5xy0(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let VY: u8 = ((self.current_op & 0x00FF) >> 4) as u8;

        if self.register[Vx as usize] == self.register[VY as usize] {
            self.pc += 2;
        }
    }

    pub fn OP_6xkk(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let byte: u8 = (self.current_op & 0x00FF) as u8;

        self.register[Vx as usize] = byte;
    }

    pub fn OP_7xkk(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let byte: u8 = (self.current_op & 0x00FF) as u8;

        let sum: u16 = self.register[Vx as usize] as u16 + byte as u16;
        if (sum > 255) {
            self.register[0xF] = 1;
        }
        else {
            self.register[0xF] = 0
        }

        self.register[Vx as usize] = (sum & 0xFF) as u8;
    }

    pub fn OP_8xy0(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let VY: u8 = ((self.current_op & 0x00FF) >> 4) as u8;

        self.register[Vx as usize] = self.register[VY as usize];
    }

    pub fn OP_8xy1(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let VY: u8 = ((self.current_op & 0x00FF) >> 4) as u8;

        self.register[Vx as usize] |= self.register[VY as usize];
    }

    pub fn OP_8xy2(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let VY: u8 = ((self.current_op & 0x00FF) >> 4) as u8;

        self.register[Vx as usize] &= self.register[VY as usize];
    }

    pub fn OP_8xy3(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let VY: u8 = ((self.current_op & 0x00FF) >> 4) as u8;

        self.register[Vx as usize] ^= self.register[VY as usize];
    }

    pub fn OP_8xy4(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let VY: u8 = ((self.current_op & 0x00FF) >> 4) as u8;

        let sum: u16  = self.register[Vx as usize] as u16 + self.register[VY as usize] as u16;

        if (sum > 255) {
            self.register[0xF] = 1; 
        }
        else {
            self.register[0xF] = 0; 
        }

        self.register[Vx as usize] = (sum & 0xFF) as u8;
        
    }

    pub fn OP_8xy5(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let VY: u8 = ((self.current_op & 0x00FF) >> 4) as u8;

        if (self.register[Vx as usize] > self.register[VY as usize]) {
            self.register[0xF] = 1; 
        }
        else {
            self.register[0xF] = 0; 
        }
        let result = self.register[Vx as usize].wrapping_sub(self.register[VY as usize]);

        self.register[Vx as usize] = result;
    }

    pub fn OP_8xy6(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        self.register[0xF] = self.register[Vx as usize] & 0x01;

        self.register[Vx as usize] >>= 1;
    }

    pub fn OP_8xy7(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let VY: u8 = ((self.current_op & 0x00FF) >> 4) as u8;

        if (self.register[VY as usize] > self.register[Vx as usize]) {
            self.register[0xF] = 1; 
        }
        else {
            self.register[0xF] = 0; 
        }

        self.register[Vx as usize] = self.register[VY as usize] - self.register[Vx as usize];
    }    

    pub fn OP_8xye(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        self.register[0xF] = (self.register[Vx as usize] & 0x80) >> 7;

        self.register[Vx as usize] <<= 1;
    }

    pub fn OP_9xy0(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let VY: u8 = ((self.current_op & 0x00FF) >> 4) as u8;

        if self.register[Vx as usize] != self.register[VY as usize] {
            self.pc += 2;
        }
    }

    pub fn OP_Annn(&mut self) {
       let address  = self.current_op & 0x0FFF;
       self.index = address;
    }

    pub fn OP_Bnnn(&mut self) {
        let address  = self.current_op & 0x0FFF;
        self.pc = self.register[0x0] as u16 + address;
     }

    pub fn OP_Cxkk(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let byte: u8 = (self.current_op & 0x00FF) as u8;
        let random = self.rng.gen_range(0..256) as u8;
        self.register[Vx as usize] = byte & random;
    }

    pub fn OP_Dxyn(&mut self) {
        let vx = ((self.current_op & 0x0F00) >> 8) as usize;
        let vy = ((self.current_op & 0x00F0) >> 4) as usize;
        let height = (self.current_op & 0x000F) as u8;
    
        let xpos = self.register[vx] % 64; // Screen width is 64
        let ypos = self.register[vy] % 32; // Screen height is 32
        self.register[0xF] = 0; // Reset collision flag
    
        for yline in 0..height {
            let sprite_byte = self.memory[(self.index + yline as u16) as usize];
            for xline in 0..8 {
                let sprite_pixel = (sprite_byte >> (7 - xline)) & 1; // Extract individual sprite pixel
                if sprite_pixel == 1 {
                    let x = ((xpos + xline) as usize) % 64; // Wrap horizontally
                    let y = ((ypos + yline) as usize) % 32; // Wrap vertically
    
                    // Check the current screen pixel
                    if self.video_memory[y][x] == 1 {
                        self.register[0xF] = 1; // Set collision flag if there's an overlap
                    }
    
                    // XOR the pixel value
                    self.video_memory[y][x] ^= 1;
                }
            }
        }
    }
    

    pub fn OP_Ex9E(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let key = self.register[Vx as usize];
        if self.keypad[key as usize] == 1 {
            self.pc += 2;
        }
    }

    pub fn OP_EXA1(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let key = self.register[Vx as usize];
        if self.keypad[key as usize] != 1 {
            self.pc += 2;
        }
    }

    pub fn OP_Fx07(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        self.register[Vx as usize] = self.delay_timer;
    }

    pub fn OP_Fx0A(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let mut key_pressed = None;
    
        // Check for any pressed key
        for (i, &key) in self.keypad.iter().enumerate() {
            if key == 1 {
                key_pressed = Some(i as u8);
                break; // Exit loop on the first key press
            }
        }
    
        match key_pressed {
            Some(key) => {
                self.register[Vx as usize] = key; // Store the key in Vx
            }
            None => {
                self.pc -= 2; // Repeat instruction if no key is pressed
            }
        }
    }
    

    pub fn OP_Fx15(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        self.delay_timer = self.register[Vx as usize];
    }

    pub fn OP_Fx18(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        self.sound_timer = self.register[Vx as usize];
    }

    pub fn OP_Fx1E(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        self.index += self.register[Vx as usize] as u16;
    }

    pub fn OP_Fx29(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let digit =  self.register[Vx as usize];
        self.index = 0x50 + (5 * digit) as u16;
    }

    pub fn OP_Fx33(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;
        let mut value = self.register[Vx as usize];
        self.memory[(self.index + 2) as usize] = value % 10;
        value /= 10;
        self.memory[(self.index + 1) as usize] = value % 10;
        value /= 10;
        self.memory[self.index as usize] = value % 10;
    }

    pub fn OP_Fx55(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;

        for i in 0..Vx {
            self.memory[(self.index + i as u16) as usize] = self.register[i as usize];
        }
    }

    pub fn OP_Fx56(&mut self) {
        let Vx = ((self.current_op & 0x0F00) >> 8) as u8;

        for i in 0..Vx {
            self.register[i as usize] = self.memory[(self.index + i as u16) as usize]
        }
    } 
    
}

fn update_keys(down: bool, inputs: Vec<Key>, built_keys: &mut [u8; 16]) {
    inputs.iter().for_each(|k| {
        match k {
            Key::Key1 => built_keys[0] = if down { 1 } else { 0 },
            Key::Key2 => built_keys[1] = if down { 1 } else { 0 },
            Key::Key3 => built_keys[2] = if down { 1 } else { 0 },
            Key::Key4 => built_keys[3] = if down { 1 } else { 0 },
            Key::Q => built_keys[4] = if down { 1 } else { 0 },
            Key::W => built_keys[5] = if down { 1 } else { 0 },
            Key::E => built_keys[6] = if down { 1 } else { 0 },
            Key::R => built_keys[7] = if down { 1 } else { 0 },
            Key::A => built_keys[8] = if down { 1 } else { 0 },
            Key::S => built_keys[9] = if down { 1 } else { 0 },
            Key::D => built_keys[10] = if down { 1 } else { 0 },
            Key::F => built_keys[11] = if down { 1 } else { 0 },
            Key::Z => built_keys[12] = if down { 1 } else { 0 },
            Key::X => built_keys[13] = if down { 1 } else { 0 },
            Key::C => built_keys[14] = if down { 1 } else { 0 },
            Key::V => built_keys[15] = if down { 1 } else { 0 },
            _ => {}
        }
    });
}

fn main() {
    println!("CHIP-8 emu");
    stdout().flush().unwrap();
    let mut file_name = String::new();
    stdin().read_line(&mut file_name).unwrap();

    let mut chip8 = Chip8::init();
    let scale = 10; // Scale factor
    let width = 64 * scale;
    let height = 32 * scale;

    let mut window = Window::new(
        "Chip8-Emulator",
        width,
        height,
        WindowOptions::default(),
    )
    .unwrap();
    window.set_target_fps(60);
    chip8.load_rom(file_name);


    let mut frame_buffer = vec![0u32; width * height];
    let mut key_pressed = Vec::new();
    let mut key_released = Vec::new();

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        chip8.cycle();

        key_released = window.get_keys_released();
        update_keys(false, key_released, &mut chip8.keypad);

        // Scale up the video memory
        for (y, row) in chip8.video_memory.iter().enumerate() {
            for (x, &pixel) in row.iter().enumerate() {
                let color = if pixel == 1 { 0xFFFFFFFF } else { 0xFF000000 }; // White for ON, Black for OFF
                for dy in 0..scale {
                    for dx in 0..scale {
                        let scaled_x = x * scale + dx;
                        let scaled_y = y * scale + dy;
                        frame_buffer[scaled_y * width + scaled_x] = color;
                    }
                }
            }
        }

        //chip 8 keys

       key_pressed = window.get_keys_pressed(minifb::KeyRepeat::No);
       update_keys(true, key_pressed, &mut chip8.keypad);

        window
            .update_with_buffer(&frame_buffer, width, height)
            .unwrap();

        frame_buffer.fill(0); // Clear the buffer
    }
}
