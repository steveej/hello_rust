extern crate invsearch;
extern crate libc;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate hyper;
extern crate futures;
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
        info!("{:?}", &req);
        match (req.method(), req.path()) {
            (&Method::Get, "/") => {
                Box::new(futures::future::ok(Response::new().with_body(
                    "Try POSTing data to /invoice/parse",
                )))
            }
            (&Method::Post, "/invoice/parse") => Box::new(
                req.body().concat2().map(|chunk: Chunk| {
                    let collected = chunk.iter().cloned().collect::<Vec<u8>>();
                    match String::from_utf8(collected) {
                        Err(e) => {
                            Response::new()
                                .with_status(StatusCode::BadRequest)
                                .with_body(format!("{}", e))
                        }
                        Ok(s) => {
                            info!("{}", s);
                            match ::invsearch::parse_invoice(s) {
                                Ok(inv) => Response::new().with_body(format!("{:?}", inv)),
                                Err(e) => {
                                    Response::new()
                                        .with_status(StatusCode::BadRequest)
                                        .with_body(format!("{}", e))
                                }
                            }
                        }
                    }
                }),
            ),
            _ => {
                Box::new(futures::future::ok(
                    Response::new().with_status(StatusCode::NotFound),
                ))
            }
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