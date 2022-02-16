use std::fs::File;
use std::io::{BufRead, BufReader};
use std::{process, path, env};


fn main() {

    // read the command line argument
    let args: Vec<String> = env::args().collect();

    // check that we have at least one argument (the filename)
    let mut filename = "";
    if args.len() > 1 {
        filename = &args[1];
    }

    // check if file exists
    if !path::Path::new(filename).exists() {
        println!("Error: could not find the file [{}].", filename);
        process::exit(1);
    }

    println!("filename = [{}]", filename);

    // open the file
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    for line in reader.lines() {
        for c in line.expect("Nope").chars() {
            println!("{}", c);
        }
    }
} 