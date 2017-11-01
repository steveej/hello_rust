#![recursion_limit = "4096"]

pub mod db;
pub mod schema;
pub mod models;

use std::env;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[macro_use]
extern crate diesel_codegen;

#[macro_use]
extern crate diesel;

extern crate libc;

#[macro_use]
extern crate nom;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate edit_distance;

#[macro_export]
macro_rules! COULDNT_OPEN_ERR { () => { "couldn't open {}: {}" }; }

#[macro_export]
macro_rules! COULDNT_READ_ERR { () => { "couldn't read {:?}: {}" }; }

#[macro_export]
macro_rules! MISSING_ARGUMENTS_ERR { () => { "No arguments provided. Please provide files to parse." }; }

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
        Err(why) => Err((
            format!(COULDNT_OPEN_ERR!(), display, why.description()),
            ::libc::ENOENT,
        )),
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

use std::fmt::Debug;
fn extract_result<T: Debug>(res: ::nom::IResult<&str, T>) -> Result<T, String> {
    match res {
        ::nom::IResult::Done(rest, value) => {
            // FIXME: doesn't show up
            info!(
                "Done\n--- Rest ---\n{}\n--- Value ---\n{:?}\n---",
                rest,
                value
            );
            Ok(value)
        }
        ::nom::IResult::Error(err) => {
            // FIXME: doesn't show up
            warn!("Err {:?}", err);
            Err(format!("Err {:?}", err))
        }
        ::nom::IResult::Incomplete(needed) => {
            // FIXME: doesn't show up
            warn!("Needed {:?}", needed);
            Err(format!("Needed {:?}", needed))
        }
    }
}

pub fn parse_invoice(buf: String) -> Result<::models::invoice::Invoice, String> {

    named!(format1<&str, ::models::invoice::Invoice>, do_parse!(
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
                (::models::invoice::Invoice::new(sirname, name, date, "", total))
            )
        );

    named!(format2<&str, ::models::invoice::Invoice>, do_parse!(
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
                (::models::invoice::Invoice::new(sirname, name, date, number, total))
            )
        );

    named!(format3<&str, ::models::invoice::Invoice>, do_parse!(
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
                (::models::invoice::Invoice::new(sirname, name, date, number, total))
            )
        );

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

pub fn parse_payments(buf: String) -> Result<Vec<::models::payment::Payment>, String> {
    let mut results: Vec<::models::payment::Payment> = Vec::new();

    for line in buf.split("\n") {
        let line_items: Vec<String> = line.split(";")
            .map(|cell: &str| cell.trim().to_string())
            .collect();

        if line_items.len() < 6 {
            continue;
        }

        let date = &line_items[0];
        let number = &line_items[1];
        let unused = &line_items[2];
        let subject = &line_items[3];
        let invoice_number = &line_items[4];
        let amount = &line_items[5];

        results.push(::models::payment::Payment::new(
            &date,
            &number,
            &unused,
            &subject,
            &invoice_number,
            &amount,
        ));
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    #[test]
    fn parse_sample_invoices() {
        let samples =
            [
                (
                    "tests/assets/invoice_1.txt",
                    &::models::invoice::Invoice::new("Doe", "John", "99.99.9999", "", "999.00"),
                ),
                (
                    "tests/assets/invoice_2.txt",
                    &::models::invoice::Invoice::new("Doe", "John", "99.99.9999", "999", "999.00"),
                ),
                (
                    "tests/assets/invoice_3.txt",
                    &::models::invoice::Invoice::new("Doe", "John", "99.99.9999", "999", "999.00"),
                ),
                (
                    "tests/assets/invoice_4.txt",
                    &::models::invoice::Invoice::new("Döe", "John", "99.99.9999", "", "999.00"),
                ),
                (
                    "tests/assets/invoice_5.txt",
                    &::models::invoice::Invoice::new("Doe", "John", "99.99.9999", "999", "999.00"),
                ),
                (
                    "tests/assets/invoice_6.txt",
                    &::models::invoice::Invoice::new("Doe", "John", "99.99.9999", "", "9'999.00"),
                ),
            ];

        for &(path, invoice_expected) in samples.iter() {
            println!("parsing {}", path);
            let mut f = ::open_file(&path.to_string()).unwrap();
            let buf = ::read_to_string(&mut f).unwrap();
            let invoice = &::parse_invoice(buf).unwrap();
            assert_eq!(invoice, invoice_expected);
        }
    }

    #[test]
    fn parse_sample_payments() {
        let samples = [
            (
                "tests/assets/payments_1.txt",
                [
                    ::models::payment::Payment::new(
                        "98.98.9988",
                        "99998",
                        "",
                        "John Döe Nowhere",
                        "998/9999",
                        "9099.90",
                    ),
                    ::models::payment::Payment::new(
                        "99.99.2016",
                        "99999",
                        "",
                        "John & Lara Doe Nü Mexico",
                        "",
                        "9090.00",
                    ),
                ],
            ),
        ];

        for &(path, ref payments_expected) in samples.iter() {
            println!("parsing {}", &path);
            let mut f = ::open_file(&path.to_string()).unwrap();
            let buf = ::read_to_string(&mut f).unwrap();
            let payments = &::parse_payments(buf).unwrap();

            assert_eq!(payments.len(), payments_expected.len());

            for (payment, payment_expected) in payments.iter().zip(payments_expected.iter()) {
                assert_eq!(payment, payment_expected);
            }
        }
    }
}