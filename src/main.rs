// #![feature(rust_2018_preview)]
// #![feature(crate_in_paths)]
//#![allow(dead_code)]
// #![feature(nll)]
#[macro_use]
extern crate clap;

mod cpu65;
mod disasm;
mod prascii;

use clap::{App, Arg};
use crate::prascii::print_ascii;

use std::collections::HashMap;
use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let matches = App::new("fe02")
        .version(crate_version!())
        .about("NMOS 6502 emulator")
        .author(crate_authors!())
        .arg(
            Arg::with_name("ifile")
                // .short("i")
                // .long("input")
                .value_name("INFILE")
                .takes_value(true)
                .required(true)
                .index(1)
                .help("Sets the binary file to read"),
        ).arg(
            Arg::with_name("address")
                .short("a")
                .long("address")
                .takes_value(true)
                .conflicts_with("haddress")
                .help("Positive integer: load address for raw binary"),
        ).arg(
            Arg::with_name("haddress")
                .short("h")
                .long("haddress")
                .takes_value(true)
                .help("Positive hexidecimal integer: load address for raw binary"),
        ).arg(
            Arg::with_name("disassemble")
                .short("d")
                .multiple(false)
                // .conflicts_with("trace")
                // .required(true)
                .help("Disassemble the binary"),
        ).arg(
            Arg::with_name("trace")
                .short("t")
                .multiple(false)
                // .required(true)
                .help("Trace execution of the binary"),
        ).arg(
            Arg::with_name("steps")
                .short("s")
                .long("steps")
                .takes_value(true)
                .requires("trace")
                .help("Positive integer: number of steps to trace"),
        ).get_matches();

    let steps = matches
        .value_of("steps")
        .unwrap_or("5")
        .parse::<u32>()
        .unwrap_or(5);

    let load_add = if matches.is_present("address") {
        match matches.value_of("address") {
            Some(add) => match usize::from_str_radix(add, 10) {
                Ok(x) => Some(x),
                Err(_) => None,
            },
            None => None,
        }
    } else {
        match matches.value_of("haddress") {
            Some(add) => match usize::from_str_radix(add, 16) {
                Ok(x) => Some(x),
                Err(_) => None,
            },
            None => None,
        }
    };

    let fname = matches.value_of("ifile").unwrap();
    let buf = read_program(fname)?;
    let mut cpu = cpu65::CPU::new();
    let segs = cpu.load(&buf, load_add)?;

    // count_implemented();
    // println!();

    if matches.is_present("disassemble") {
        print_ascii(&";;; begin disassembley ;;;\n");
        print_ascii(&"    PROCESSOR 6502");
        print_ascii(&"    LIST ON\n\n");
        print_ascii(&"START\n");

        let mut map: HashMap<usize, String> = HashMap::new();
        // blindly assuming all segments are code
        for seg in &segs {
            // first_pass finds jump/branch addresses and creates labels
            disasm::first_pass(&cpu, seg.start, seg.end, &mut map);
            disasm::disasm(&cpu, seg.start, seg.end, Some(&map));
            // disasm::disasm(&cpu, seg.start, seg.end, None);
        }
    }
    if matches.is_present("trace") {
        let start = segs[0].start as u16;
        disasm::trace(&mut cpu, start, steps, None);
    }

    // let fname = "mem.bin";
    // fs::write(fname, &cpu.get_mem()[..])?;

    Ok(())
}

// fn count_implemented() {
//     // count implemented
//     let n = CPU::emu_err as *const fn(&mut CPU);
//     let k = INSTRUCTIONS
//         .iter()
//         .filter(|f| f.ef as *const fn(&mut CPU) != n)
//         .count();
//     println!("{} instructions implememnted!", k);
// }

fn read_program(fname: &str) -> Result<Vec<u8>, io::Error> {
    use std::io::{Error, ErrorKind};

    if fs::metadata(&fname)?.is_dir() {
        return Err(Error::new(ErrorKind::Other, "Input file is a directory!"));
    }
    let buf = fs::read(&fname)?;

    Ok(buf)
}
