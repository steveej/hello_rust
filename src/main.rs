extern crate invsearch;
extern crate libc;

extern crate env_logger;
#[macro_use]
extern crate log;

extern crate futures;
extern crate hyper;
use futures::future::Future;
use futures::Stream;
use hyper::server::{Http, Request, Response, Service};
use hyper::{Chunk, Method, StatusCode};

struct InvoiceParser;
impl Service for InvoiceParser {
    // boilerplate hooking up hyper's server types
    type Request = Request;
    type Response = Response;
    type Error = hyper::Error;

    // The future representing the eventual Response your call will
    // resolve to. This can change to whatever Future you need.
    type Future = Box<Future<Item = Self::Response, Error = Self::Error>>;


    fn call(&self, req: Request) -> Self::Future {
        debug!("{:?}", &req);

        match (req.method(), req.path()) {
            (&Method::Get, "/") => Box::new(futures::future::ok(Response::new().with_body(
                // TODO return a generated list of endpoints
                r#"
GET /invoice/index
POST /invoice/parse
GET /invoice/unpaid
GET /payment/index
POST /payment/parse
                "#,
            ))),
            (&Method::Get, "/invoice/index") => {
                match ::invsearch::db::get_invoices() {
                    Ok(invoices) => Box::new(futures::future::ok(
                        Response::new().with_body(format!("{:?}", invoices)),
                    )),
                    Err(e) => {
                        error!("{}", e);
                        Box::new(futures::future::ok(Response::new().with_body(e)))
                    }
                }
            }
            (&Method::Post, "/invoice/parse") => {
                Box::new(req.body().concat2().map(|chunk: Chunk| {
                    let collected = chunk.iter().cloned().collect::<Vec<u8>>();
                    match String::from_utf8(collected) {
                        Err(e) => {
                            let e = format!("Body must be valid UTF8: {}", e);
                            error!("{}", e);
                            Response::new()
                                .with_status(StatusCode::BadRequest)
                                .with_body(e)
                        }
                        Ok(s) => {
                            debug!("{}", s);
                            match ::invsearch::parse_invoice(s) {
                                Ok(inv) => {
                                    ::invsearch::db::insert_invoice(&inv).unwrap();
                                    Response::new().with_body("Invoice inserted.")
                                }
                                Err(e) => {
                                    let e = format!("Could not parse body as Invoice: {}", e);
                                    error!("{}", e);
                                    Response::new()
                                        .with_status(StatusCode::BadRequest)
                                        .with_body(e)
                                }
                            }
                        }
                    }
                }))
            }
            (&Method::Get, "/invoice/unpaid") => {
                match ::invsearch::db::get_unpaid_invoices() {
                    Ok(payments) => Box::new(futures::future::ok(
                        Response::new().with_body(format!("{:?}", payments)),
                    )),
                    Err(e) => {
                        error!("{}", e);
                        Box::new(futures::future::ok(Response::new().with_body(e)))
                    }
                }
            }
            (&Method::Get, "/payment/index") => {
                match ::invsearch::db::get_payments() {
                    Ok(payments) => Box::new(futures::future::ok(
                        Response::new().with_body(format!("{:?}", payments)),
                    )),
                    Err(e) => {
                        error!("{}", e);
                        Box::new(futures::future::ok(Response::new().with_body(e)))
                    }
                }
            }
            (&Method::Post, "/payment/parse") => {
                Box::new(req.body().concat2().map(|chunk: Chunk| {
                    let collected = chunk.iter().cloned().collect::<Vec<u8>>();
                    match String::from_utf8(collected) {
                        Err(e) => {
                            let e = format!("Body must be valid UTF8: {}", e);
                            error!("{}", e);
                            Response::new()
                                .with_status(StatusCode::BadRequest)
                                .with_body(e)
                        }
                        Ok(s) => {
                            debug!("{}", s);
                            match ::invsearch::parse_payments(s) {
                                Ok(payments) => {
                                    let db_payments = ::invsearch::db::insert_payments(payments)
                                        .unwrap();
                                    Response::new().with_body(format!(
                                        "Inserted {} payments",
                                        db_payments.len()
                                    ))
                                }
                                Err(e) => {
                                    let e = format!("Could not parse body as Payments: {}", e);
                                    error!("{}", e);
                                    Response::new()
                                        .with_status(StatusCode::BadRequest)
                                        .with_body(e)
                                }
                            }
                        }
                    }
                }))
            }
            _ => Box::new(futures::future::ok(
                Response::new()
                    .with_status(StatusCode::NotFound)
                    .with_body("404 NOT FOUND"),
            )),
        }
    }
}

fn main() {
    env_logger::init().unwrap();

    let addr = "127.0.0.1:3000".parse().unwrap();
    let server = Http::new().bind(&addr, || Ok(InvoiceParser)).unwrap();

    server.run().unwrap();

    /* TODO: migrate this to a client

    // Receive command line arguments
    let args = match ::invsearch::get_args() {
        Err(e) => {
            eprintln!("{}", e);
            ::std::process::exit(::libc::EINVAL)
        }
        Ok(a) => a,
    };

    for arg in args[1..].iter() {
        // Create a path to the desired file

        let mut f = match ::invsearch::open_file(arg) {
            Err((string, code)) => {
                eprintln!("{}", string);
                ::std::process::exit(code)
            }
            Ok(f) => f,
        };

        let buf = ::invsearch::read_to_string(&mut f).unwrap();

        match ::invsearch::parse_invoice(buf) {
            Ok(inv) => println!("{:?}", inv),
            Err(e) => eprintln!("'{}': {}", arg, e),
        };
    }
    */
}