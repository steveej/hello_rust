use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[macro_use]
extern crate libc;

#[macro_use]
extern crate nom;

#[macro_use]
extern crate log;

#[macro_export]
macro_rules! COULDNT_OPEN_ERR { () => { "couldn't open {}: {}" }; }

#[macro_export]
macro_rules! COULDNT_READ_ERR { () => { "couldn't read {:?}: {}" }; }

#[macro_export]
macro_rules! MISSING_ARGUMENTS_ERR { () => { "No arguments provided. Please provide files to parse." }; }

#[derive(PartialEq, Debug)]
pub struct Invoice {
    sirname: String,
    name: String,
    date: String,
    number: String,
    total: String,
}

impl Invoice {
    pub fn new(sirname: &str, name: &str, date: &str, number: &str, total: &str) -> Invoice {
        ::Invoice {
            date: date.trim().to_string(),
            number: number.trim().to_string(),
            name: name.trim().to_string(),
            sirname: sirname.trim().to_string(),
            total: total.trim().to_string().replace("'", ""),
        }
    }
}

pub fn get_args() -> Result<Vec<String>, &'static str> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        0 | 1 => Err(MISSING_ARGUMENTS_ERR!()),
        _ => Ok(args),
    }
}

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

pub fn parse_invoice(f: &mut File) -> Result<Invoice, String> {
    let sirname: String;
    let name: String;
    let date: String;
    let number: String;
    let total: String;

    sirname = String::new();
    name = String::new();
    date = String::new();
    number = String::new();
    total = String::new();
    {
        use std::fmt::Display;
        use std::fmt::Debug;

        let buf = ::read_to_string(f).unwrap();

        named!(format1<&str, Invoice>, do_parse!(
                take_until_and_consume!("Name/Vorname") >>
                take_until_and_consume!("Name/Vorname") >>
                take_until_and_consume!("\n") >>
                take_until_and_consume!("\n") >>
                sirname: take_until!(" ") >>
                take_while_s!(|s:char| { s == ' '}) >>
                name: take_until!("\n") >>
                take_until_and_consume!("Rechnungsdatum:") >>
                take_until_and_consume!("\n") >>
                date: take_until_and_consume!("\n") >>
                take_until_and_consume!("otal") >>
                take_until_and_consume!("CHF") >>
                take_while_s!(|s:char| { s == ' ' || s == '\n'}) >>
                total: take_until_and_consume!("\n") >>
                (Invoice::new(sirname, name, date, "", total))
            )
        );

        named!(format2<&str, Invoice>, do_parse!(
                take_until_and_consume!("Name/Vorname") >>
                take_until_and_consume!("Name/Vorname") >>
                take_until_and_consume!("\n") >>
                take_until_and_consume!("\n") >>
                sirname: take_until!(" ") >>
                take_while_s!(|s:char| { s == ' '}) >>
                name: take_until!("\n") >>
                take_until_and_consume!("Rechnungsdatum:") >>
                take_until_and_consume!("\n") >>
                date: take_until_and_consume!("\n") >>
                take_until_and_consume!("Rechnungs-Nr") >>
                take_until_and_consume!("\n") >>
                number: take_until_and_consume!("\n") >>
                take_until_and_consume!("otal") >>
                take_until_and_consume!("CHF\n") >>
                total: take_until_and_consume!("\n") >>
                (Invoice::new(sirname, name, date, number, total))
            )
        );

        named!(format3<&str, Invoice>, do_parse!(
                take_until_and_consume!("Name/Vorname") >>
                take_until_and_consume!("Name/Vorname") >>
                take_until_and_consume!("\n") >>
                take_until_and_consume!("\n") >>
                sirname: take_until!(" ") >>
                take_while_s!(|s:char| { s == ' '}) >>
                name: take_until!("\n") >>
                take_until_and_consume!("Rechnungsdatum: Rechnungs-Nr") >>
                take_until_and_consume!("\n") >>
                date: take_until_and_consume!("\n") >>
                number: take_until_and_consume!("\n") >>
                take_until_and_consume!("otal") >>
                take_until_and_consume!("CHF\n") >>
                total: take_until_and_consume!("\n") >>
                (Invoice::new(sirname, name, date, number, total))
            )
        );

        fn extract_result<T: Debug>(res: ::nom::IResult<&str, T>) -> Result<T, String> {
            match res {
                ::nom::IResult::Done(rest, value) => {
                    debug!("Done\n--- Rest ---\n{}\n--- Value ---\n{:?}\n---",
                           rest,
                           value);
                    Ok(value)
                }
                ::nom::IResult::Error(err) => Err(format!("Err {:?}", err)),
                ::nom::IResult::Incomplete(needed) => Err(format!("Needed {:?}", needed)),
            }
        };

        match extract_result(format3(&buf)) {
            Ok(res3) => Ok(res3),
            _ => {
                match extract_result(format2(&buf)) {
                    Ok(res2) => Ok(res2),
                    _ => extract_result(format1(&buf)),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_sample_invoices() {
        let samples = [("tests/assets/invoice_1.txt",
                        &::Invoice::new("Doe", "John", "99.99.9999", "", "999.00")),
                       ("tests/assets/invoice_2.txt",
                        &::Invoice::new("Doe", "John", "99.99.9999", "999", "999.00")),
                       ("tests/assets/invoice_3.txt",
                        &::Invoice::new("Doe", "John", "99.99.9999", "999", "999.00")),
                       ("tests/assets/invoice_4.txt",
                        &::Invoice::new("Döe", "John", "99.99.9999", "", "999.00")),
                       ("tests/assets/invoice_5.txt",
                        &::Invoice::new("Doe", "John", "99.99.9999", "999", "999.00")),
                       ("tests/assets/invoice_6.txt",
                        &::Invoice::new("Doe", "John", "99.99.9999", "", "9'999.00"))];

        for &(path, invoice_expected) in samples.iter() {
            println!("parsing {}", path);
            let mut f = ::open_file(&path.to_string()).unwrap();
            let invoice = &::parse_invoice(&mut f).unwrap();
            assert_eq!(invoice, invoice_expected);
        }
    }
}
