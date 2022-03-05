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

    fn data_value_inc(&mut self) {
        let mut value: i16 = self.data[self.dp] as i16;
        value = (value + 1) & 255;
        self.data[self.dp] = value as u8;
    }

    fn data_value_dec(&mut self) {
        let mut value: i16 = self.data[self.dp] as i16;
        value = (value - 1) & 255;
        self.data[self.dp] = value as u8;
    }

    fn data_ptr_inc(&mut self) {
        self.dp = (self.dp + 1) & (DATA_SIZE - 1);
    }

    fn data_ptr_dec(&mut self) {
        let mut value: i32 = self.dp as i32;
        let max = DATA_SIZE as i32;
        value = (value - 1) & (max - 1) ;
        self.dp = value as usize;
    }

    fn jump_fwd(&mut self) {
        if self.data[self.dp] == 0 {
            let value = self.pc - 1;
            self.pc = self.jumps[&value];
        }
    }

    fn jump_bck(&mut self) {
        if self.data[self.dp] != 0 {
            let value = self.pc - 1;
            self.pc = self.jumps[&value];
        }
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

        for (k, v) in &self.jumps {
            println!("[{}] = {}", k, v);
        }

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

                Opcodes::DATA_VALUE_INC => self.data_value_inc(),
                Opcodes::DATA_VALUE_DEC => self.data_value_dec(),
                Opcodes::DATA_PTR_INC => self.data_ptr_inc(),
                Opcodes::DATA_PTR_DEC => self.data_ptr_dec(),
                Opcodes::JUMP_FWD => self.jump_fwd(),
                Opcodes::JUMP_BCK => self.jump_bck(),

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

#[cfg(test)]
mod tests {
    use super::*;

    fn insert_code(a: &mut VMCore) {
        a.code.push(Opcodes::DATA_VALUE_INC);
        a.code.push(Opcodes::DATA_VALUE_INC);
        a.code.push(Opcodes::DATA_VALUE_INC);
        a.code.push(Opcodes::DATA_VALUE_INC);
        a.code.push(Opcodes::DATA_VALUE_INC);
        a.code.push(Opcodes::JUMP_FWD);
        a.code.push(Opcodes::DATA_PTR_INC);
        a.code.push(Opcodes::DATA_VALUE_INC);
        a.code.push(Opcodes::DATA_PTR_DEC);
        a.code.push(Opcodes::DATA_VALUE_DEC);
        a.code.push(Opcodes::JUMP_BCK);
    }

    #[test]
    fn create_vm_core() {
        let a = VMCore::new();
        assert_eq!(a.pc, 0);
        assert_eq!(a.dp, 0);
        assert_eq!(a.code.capacity(), CODE_SIZE);
        assert_eq!(a.data.len(), DATA_SIZE);
        assert_eq!(a.jumps.len(), 0);
    }

    #[test]
    fn compute_jumps_unbalanced_one() {
        let mut a = VMCore::new();
        a.code.push(Opcodes::JUMP_FWD);
        a.code.push(Opcodes::DATA_VALUE_INC);
        a.code.push(Opcodes::JUMP_BCK);
        a.code.push(Opcodes::JUMP_BCK);

        assert_eq!(a.compute_jumps(4), Err("Error: unbalanced number of '[' and ']' in the source code!".to_string()))
    }

    #[test]
    fn compute_jumps_unbalanced_two() {
        let mut a = VMCore::new();
        a.code.push(Opcodes::JUMP_FWD);
        a.code.push(Opcodes::JUMP_FWD);
        a.code.push(Opcodes::DATA_VALUE_INC);
        a.code.push(Opcodes::JUMP_BCK);

        assert_eq!(a.compute_jumps(4), Err("Error: unbalanced number of '[' and ']' in the source code!".to_string()))
    }

    #[test]
    fn compute_jumps_one_loop_correct() {
        let mut a = VMCore::new();

        // insert the code and compute jumps
        insert_code(&mut a);
        a.compute_jumps(11).unwrap();

        // assess if the jumps are correctly computed
        assert_eq!(a.jumps[&5], 11);
        assert_eq!(a.jumps[&10], 6);
    }

    #[test]
    fn data_value_inc() {
        let mut a = VMCore::new();

        a.code.push(Opcodes::DATA_VALUE_INC);
        a.code.push(Opcodes::DATA_VALUE_INC);
        a.code.push(Opcodes::DATA_VALUE_INC);
        a.code.push(Opcodes::DATA_VALUE_INC);
        a.code.push(Opcodes::DATA_VALUE_INC);

        a.execute();

        assert_eq!(a.data[0], 5);
    }

    #[test]
    fn data_value_dec() {
        let mut a = VMCore::new();

        a.code.push(Opcodes::DATA_VALUE_DEC);
        a.code.push(Opcodes::DATA_VALUE_DEC);
        a.code.push(Opcodes::DATA_VALUE_DEC);
        a.code.push(Opcodes::DATA_VALUE_DEC);
        a.code.push(Opcodes::DATA_VALUE_DEC);

        a.data[0] = 7;
        a.execute();

        assert_eq!(a.data[0], 2);
    }

    #[test]
    fn data_value_inc_overflow() {
        let mut a = VMCore::new();

        a.data[0] = 253;
        a.code.push(Opcodes::DATA_VALUE_INC);
        a.code.push(Opcodes::DATA_VALUE_INC);
        a.code.push(Opcodes::DATA_VALUE_INC);
        a.code.push(Opcodes::DATA_VALUE_INC);
        a.code.push(Opcodes::DATA_VALUE_INC);
        a.execute();

        assert_eq!(a.data[0], 2);
    }

    #[test]
    fn data_value_dec_overflow() {
        let mut a = VMCore::new();

        a.data[0] = 2;
        a.code.push(Opcodes::DATA_VALUE_DEC);
        a.code.push(Opcodes::DATA_VALUE_DEC);
        a.code.push(Opcodes::DATA_VALUE_DEC);
        a.code.push(Opcodes::DATA_VALUE_DEC);
        a.code.push(Opcodes::DATA_VALUE_DEC);
        a.execute();

        assert_eq!(a.data[0], 253);
    }

    #[test]
    fn data_ptr_inc() {
        let mut a = VMCore::new();

        a.code.push(Opcodes::DATA_PTR_INC);
        a.execute();

        assert_eq!(a.dp, 1);
    }

    #[test]
    fn data_ptr_inc_overflow() {
        let mut a = VMCore::new();

        a.dp = DATA_SIZE - 2;
        a.code.push(Opcodes::DATA_PTR_INC);
        a.code.push(Opcodes::DATA_PTR_INC);
        a.code.push(Opcodes::DATA_PTR_INC);
        a.code.push(Opcodes::DATA_PTR_INC);
        a.execute();

        assert_eq!(a.dp, 2);
    }

    #[test]
    fn data_ptr_dec() {
        let mut a = VMCore::new();

        a.dp = 10;
        a.code.push(Opcodes::DATA_PTR_DEC);
        a.execute();

        assert_eq!(a.dp, 9);
    }

    #[test]
    fn data_ptr_dec_overflow() {
        let mut a = VMCore::new();

        a.dp = 2;
        a.code.push(Opcodes::DATA_PTR_DEC);
        a.code.push(Opcodes::DATA_PTR_DEC);
        a.code.push(Opcodes::DATA_PTR_DEC);
        a.code.push(Opcodes::DATA_PTR_DEC);
        a.execute();

        assert_eq!(a.dp, DATA_SIZE - 2);
    }

    #[test]
    fn jump_fwd_not_zero() {

        let mut a = VMCore::new();

        // insert the code and compute the jumps
        insert_code(&mut a);
        a.compute_jumps(11).unwrap();

        // change registers
        a.data[0] = 5;
        a.pc = 6;

        // compute forward jump
        a.jump_fwd();

        assert_eq!(a.pc, 6);
    }

    #[test]
    fn jmp_fwd_zero() {
        let mut a = VMCore::new();

        // insert the code and compute the jumps
        insert_code(&mut a);
        a.compute_jumps(11).unwrap();

        // jump
        a.pc = 6;
        a.jump_fwd();

        assert_eq!(a.pc, 11);
    }

    #[test]
    fn jump_bck_not_zero() {
        let mut a = VMCore::new();

        // insert the code and compute the jumps
        insert_code(&mut a);
        a.compute_jumps(11).unwrap();

        // jump
        a.data[0] = 5;
        a.pc = 11;
        a.jump_bck();

        assert_eq!(a.pc, 6);
    }

    #[test]
    fn jump_bck_zero() {
        let mut a = VMCore::new();

        // insert the code and compute the jumps
        insert_code(&mut a);
        a.compute_jumps(11).unwrap();

        // jump
        a.data[0] = 0;
        a.pc = 11;
        a.jump_bck();

        assert_eq!(a.pc, 11);
    }
}
