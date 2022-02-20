/*
 * @file    main.rs
 * @author  Sebastien LEGRAND
 *
 * @brief   Brainfuck interpreter in Rust
 */

//----- crates
use std::{process, env};


//----- modules
mod brainfuck;


//----- main function
fn main() {

    // read the command line argument
    let args: Vec<String> = env::args().collect();

    // check that we have at least one argument (the filename)
    if args.len() == 1 {
        println!("Please specify a source code file on the command line");
        process::exit(1);
    }

    // create a new Brainfuck Core VM
    let mut vm_core: brainfuck::VMCore = brainfuck::VMCore::new();

    // read the code
    let nbytes = match vm_core.load(&args[1]) {
        Ok(n) => n,
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
    };

    println!("{} bytes read.", nbytes);

    // execute the code
    vm_core.execute();
}