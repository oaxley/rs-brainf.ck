use std::io::prelude::*;
use std::fs::File;
use std::{process, path, env};
use std::collections::HashMap;
use console::Term;

//----- constants
const CODE_SIZE: usize = 32168;
const DATA_SIZE: usize = 32168;

//----- functions
// read the code from the disk
fn read_code(filename: &str, code: &mut [u8]) -> Result<usize, String> {

    // unable to find the file
    if !path::Path::new(&filename).exists() {
        return Err("Unable to find the file".to_string());
    }

    // read the code
    let mut file = File::open(filename).unwrap();
    let n = file.read(&mut code[..]).unwrap();

    Ok(n)
}

// compute the hashmap for the jumps
fn compute_jumps(code: &[u8], nbytes: usize) -> HashMap<usize, usize> {
    let mut jumps: HashMap<usize, usize> = HashMap::new();
    let mut stack: Vec<usize> = Vec::new();
    let mut counter: usize = 0;

    // parse the code and look for '[' and ']'
    while counter < nbytes {
        // retrieve the opcode
        let opcode = code[counter as usize];

        // push the current counter if we found '['
        if opcode == 91 {
            stack.push(counter)
        }

        // retrieve the last value if we found ']'
        if opcode == 93 {
            if stack.len() == 0 {
                println!("Error: unbalanced number of '[' and ']' in the source code.");
                process::exit(1);
            }

            // retrieve the position of the corresponding '['
            let value = stack.pop().unwrap();

            // insert the two value inside the hashmap
            jumps.insert(value, counter + 1);           // '[' is map to the position after the ']'
            jumps.insert(counter, value + 1);           // ']' is map to the position after the '['

        }

        // next opcode
        counter += 1;
    };

    return jumps;
}


// run the code


//----- variables

fn main() {

    let mut code: [u8; CODE_SIZE] = [0; CODE_SIZE];
    let mut data: [u8; DATA_SIZE] = [0; DATA_SIZE];

    // read the command line argument
    let args: Vec<String> = env::args().collect();

    // check that we have at least one argument (the filename)
    if args.len() == 0 {
        println!("Please specify a source code file on the command line");
        process::exit(1);
    }

    // read the code
    let nbytes = match read_code(&args[1], &mut code) {
        Ok(n) => {
            n
        },
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };

    println!("{} bytes read.", nbytes);

    // compute the jumps locations
    let jumps = compute_jumps(&code, nbytes);
    // for (k,v) in &jumps {
    //     println!("JMP[{}] = {}",k+1, v+1);
    // }

    // run the code
    let mut pc: usize = 0;
    let mut dp: i32 = 0;

    while pc <= nbytes {
        let opcode = code[pc];
        pc = pc + 1;
        // println!("PC:{} | PC[]:{} | DP:{} | DP[]:{}",pc, opcode, dp, data[dp as usize]);

        // '[' (91)
        if opcode == 91 {
            if data[dp as usize] == 0 {
                let value = pc - 1;
                pc = jumps[&value];
            }
        }

        // ']' (93)
        if opcode == 93 {
            if data[dp as usize] != 0 {
                let value = pc  -1;
                pc = jumps[&value];
            }
        }

        // '>' (62)
        if opcode == 62 {
            dp = dp + 1;
        }

        // '<' (60)
        if opcode == 60 {
            dp = dp - 1;
        }

        // '+' (43)
        if opcode == 43 {
            let mut value: u16 = data[dp as usize] as u16;
            value = (value + 1) & 255;
            data[dp as usize] = value as u8;
        }

        // '-' (45)
        if opcode == 45 {
            let mut value: i16 = data[dp as usize] as i16;
            value = (value - 1) & 255;
            data[dp as usize] = value as u8;
        }

        // '.' (46)
        if opcode == 46 {
            print!("{}",data[dp as usize] as char);
        }

        // ',' (44)
        if opcode == 44 {
            let t = Term::stdout();
            match t.read_char() {
                Ok(value) => {
                    data[dp as usize] = (value.to_digit(10).unwrap() & 255) as u8;
                },
                _ => continue
            }
        }
    }
}