
#[macro_export]
macro_rules! COULDNT_OPEN_ERR { () => { "couldn't open {}: {}" }; }

#[macro_export]
macro_rules! COULDNT_READ_ERR { () => { "couldn't read {:?}: {}" }; }

#[macro_export]
macro_rules! MISSING_ARGUMENTS_ERR { () => { "No arguments provided. Please provide files to parse." }; }

use std::env;

pub fn get_args() -> Result<Vec<String>, &'static str> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        0 | 1 => Err(MISSING_ARGUMENTS_ERR!()),
        _ => Ok(args),
    }
}

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
extern crate libc;

pub fn open_file(arg: &String) -> Result<File, (String, i32)> {
    let path = Path::new(arg);
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    match File::open(&path) {
        // The `description` method of `io::Error` returns a string that
        // describes the error
        Err(why) => Err((format!(COULDNT_OPEN_ERR!(), display, why.description()), ::libc::ENOENT)),
        Ok(file) => Ok(file),
    }
}

pub fn read_to_string(f: &mut File) -> Result<String, String> {
    let mut s = String::new();
    match f.read_to_string(&mut s) {
        Err(why) => panic!(COULDNT_READ_ERR!(), f, why.description()),
        Ok(_) => Ok(s),
    }
}
