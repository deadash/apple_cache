use anyhow::Result;
use bincode::config;
use std::fs::File;
use std::io::BufRead;
use std::{fs, io};
use std::path::Path;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn main() -> Result<()>{
    let mut values: Vec<usize> = Vec::new();
    if let Ok(lines) = read_lines("../blob/reloc.txt") {
        // Consumes the iterator, returns an (Optional) String
        for line in lines {
            if let Ok(value) = line {
                let value = value.trim();
                let value = value.trim_start_matches("0x");
                let a: u64 = u64::from_str_radix(value, 16)?;
                values.push(a as usize);
            }
        }
    }
    let config = config::standard();
    let encoded: Vec<u8> = bincode::encode_to_vec(&values, config)?;

    fs::write("../blob/reloc.bin", encoded)?;
    Ok(())
}