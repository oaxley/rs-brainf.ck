/*
 * @file    brainfuck.rs
 * @author  Sebastien LEGRAND
 *
 * @brief   Brainfuck interpreter main module
*/

//----- crates
use std::io::prelude::*;
use std::fs::File;
use std::path;
use std::collections::HashMap;
use console::Term;


//----- globals
const CODE_SIZE: usize = 1 << 15;       // max code size is 32,768 bytes
const DATA_SIZE: usize = 1 << 15;       // max data size is 32,768 bytes


//----- structures

// Opcodes definition
struct Opcodes;

impl Opcodes {
    pub const DATA_VALUE_INC: u8 = 43;       // '+' increase value at the data pointer
    pub const READ_CHAR: u8      = 44;       // ',' read a char from the user
    pub const DATA_VALUE_DEC: u8 = 45;       // '-' decrease value at the data pointer
    pub const WRITE_CHAR: u8     = 46;       // '.' write a char to the screen
    pub const DATA_PTR_DEC: u8   = 60;       // '<' move data pointer to the left
    pub const DATA_PTR_INC: u8   = 62;       // '>' move data pointer to the right
    pub const JUMP_FWD: u8       = 91;       // '[' jump forward if data value is 0
    pub const JUMP_BCK: u8       = 93;       // ']' jump backward if data value is not 0
}

// Brainfuck Virtual Machine Core
pub struct VMCore {
    pc: usize,                          // program counter
    dp: usize,                          // data pointer
    code: Vec<u8>,                      // array holding the code
    data: Vec<u8>,                      // array holding the data

    jumps: HashMap<usize, usize>,       // jumps hashmap
}

// implementation
impl VMCore {
    //----- private functions

    // compute the jumps hashmap
    fn compute_jumps(&mut self, nbytes: usize) -> Result<(), String> {
        // temporary stack to hold jumps location
        let mut stack: Vec<usize> = Vec::new();
        let mut counter: usize = 0;

        // parse the code for '[' and ']'
        while counter < nbytes {
            let opcode = self.code[counter];

            // jump forward
            if opcode == Opcodes::JUMP_FWD {
                stack.push(counter);
            }

            // jump backward
            if opcode == Opcodes::JUMP_BCK {
                // empty stack => Error
                if stack.len() == 0 {
                    return Err("Error: unbalanced number of '[' and ']' in the source code!".to_string());
                }

                // retrieve the last value found for JumpFwd
                let value = stack.pop().unwrap();

                // insert the two values in the HashMap
                self.jumps.insert(value, counter + 1);      // '[' is map to the position after the matching ']'
                self.jumps.insert(counter, value + 1);      // ']' is map to the position after the matching '['
            }

            // next opcode
            counter += 1;
        }

        // last check for missing closing jump
        if stack.len() > 0 {
            return Err("Error: unbalanced number of '[' and ']' in the source code!".to_string());
        }

        Ok(())
    }


    //----- public functions

    // constructor
    pub fn new() -> Self {
        VMCore {
            pc: 0,
            dp: 0,
            code: Vec::with_capacity(CODE_SIZE),
            data: vec![0; DATA_SIZE],
            jumps: HashMap::new()
        }
    }

    // read the code from a file
    pub fn load(&mut self, filename: &str) -> Result<usize, String> {
        // check for the file
        if !path::Path::new(&filename).exists() {
            return Err("Unable to find the file!".to_string());
        }

        // read the code
        let mut program: [u8; CODE_SIZE] = [0; CODE_SIZE];
        let mut file = File::open(filename).unwrap();
        let n = file.read(&mut program[..]).unwrap();

        // insert the code into the structure
        for i in program {
            self.code.push(i);
        }

        // compute the jumps
        self.compute_jumps(n).unwrap();

        // return the number of bytes read
        Ok(n)
    }

    // execute the code
    pub fn execute(&mut self) {
        while self.pc < self.code.len() {
            // read the next opcode and increment the program counter
            let opcode = self.code[self.pc];
            self.pc = self.pc + 1;

            // opcode lookup
            match opcode {

                Opcodes::DATA_VALUE_INC => {
                    let mut value: i16 = self.data[self.dp] as i16;
                    value = (value + 1) & 255;
                    self.data[self.dp] = value as u8;
                },

                Opcodes::DATA_VALUE_DEC => {
                    let mut value: i16 = self.data[self.dp] as i16;
                    value = (value - 1) & 255;
                    self.data[self.dp] = value as u8;
                },

                Opcodes::DATA_PTR_INC => {
                    self.dp = (self.dp + 1) & (DATA_SIZE - 1);
                },

                Opcodes::DATA_PTR_DEC => {
                    self.dp = (self.dp - 1) & (DATA_SIZE - 1);
                },

                Opcodes::JUMP_FWD => {
                    if self.data[self.dp] == 0 {
                        let value = self.pc - 1;
                        self.pc = self.jumps[&value];
                    }
                },

                Opcodes::JUMP_BCK => {
                    if self.data[self.dp] != 0 {
                        let value = self.pc - 1;
                        self.pc = self.jumps[&value];
                    }
                },

                Opcodes::WRITE_CHAR => {
                    print!("{}", self.data[self.dp] as char);
                },

                Opcodes::READ_CHAR => {
                    let t = Term::stdout();
                    match t.read_char() {
                        Ok(value) => {
                            self.data[self.dp] = (value.to_digit(10).unwrap() & 255) as u8;
                        },
                        _ => continue
                    }
                }

                // unknown opcode
                _ => continue
            }
        }
    }
}
