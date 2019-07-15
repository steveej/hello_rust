use actix_web::{web, App, Error, HttpResponse, HttpServer, Responder};
use futures::future::Future;
use tokio;

static BLOCK_TIME: std::time::Duration = std::time::Duration::from_millis(1000);

fn native_blocking(workers: usize, port: usize) {
    fn index(info: web::Path<(String, u32)>) -> impl Responder {
        if info.1 > 0 {
            std::thread::sleep(BLOCK_TIME)
        }
        format!("Hello {}! \nid:{}", info.0, info.1)
    }

    std::thread::spawn(move || {
        HttpServer::new(|| App::new().service(web::resource("/{name}/{id}/index.html").to(index)))
            .workers(workers)
            .bind(format!("127.0.0.1:{}", port))
            .unwrap()
            .run()
            .unwrap();
    });
}

fn future_blocking(workers: usize, port: usize) {
    fn index(info: web::Path<(String, u32)>) -> impl Future<Item = HttpResponse, Error = Error> {
        futures::future::ok({
            if info.1 > 0 {
                std::thread::sleep(BLOCK_TIME)
            }
            HttpResponse::with_body(
                actix_web::http::StatusCode::OK,
                actix_web::body::Body::from(format!("Hello {}! \nid:{}", info.0, info.1)),
            )
        })
    }

    std::thread::spawn(move || {
        HttpServer::new(|| {
            App::new().service(
                web::resource("/{name}/{id}/index.html").route(actix_web::web::to_async(index)),
            )
        })
        .workers(workers)
        .bind(format!("127.0.0.1:{}", port))
        .unwrap()
        .run()
        .unwrap();
    });
}

fn future_nonblocking(workers: usize, port: usize) {
    fn index(info: web::Path<(String, u32)>) -> impl Future<Item = HttpResponse, Error = Error> {
        let delay = if info.1 > 0 {
            BLOCK_TIME
        } else {
            std::time::Duration::from_millis(0)
        };

        tokio::timer::Delay::new(std::time::Instant::now() + delay)
            .map_err(|e| actix_web::Error::from(e))
            .and_then(move |()| {
                HttpResponse::with_body(
                    actix_web::http::StatusCode::OK,
                    actix_web::body::Body::from(format!("Hello {}! \nid:{}", info.0, info.1)),
                )
            })
    }
    std::thread::spawn(move || {
        HttpServer::new(|| {
            App::new().service(
                web::resource("/{name}/{id}/index.html").route(actix_web::web::to_async(index)),
            )
        })
        .workers(workers)
        .bind(format!("127.0.0.1:{}", port))
        .unwrap()
        .run()
        .unwrap();
    });
}

fn main() -> std::result::Result<(), Box<std::error::Error>> {
    let mut results: std::collections::HashMap<&'static str, usize> = Default::default();

    struct Variant {
        name: &'static str,
        f: fn(workers: usize, port: usize),
        port: usize,
    }

    for variant in [
        Variant {
            name: "native blocking",
            f: native_blocking,
            port: 8120,
        },
        Variant {
            name: "future blocking",
            f: future_blocking,
            port: 8121,
        },
        Variant {
            name: "future non-blocking",
            f: future_nonblocking,
            port: 8122,
        },
    ]
    .iter()
    {
        let workers = 5;
        let max_threads = 128;

        println!(
            "{}: spawning up to {} threads against {} workers",
            variant.name, max_threads, workers
        );

        (variant.f)(workers, variant.port);

        let mut n = 2;
        let mut done = false;
        while !done {
            let client_threads: Vec<_> = (1..n)
                .into_iter()
                .map(|i| {
                    std::thread::spawn(move || {
                        let _ = reqwest::Client::builder()
                            .timeout(Some(BLOCK_TIME))
                            .build()
                            .unwrap()
                            .request(
                                reqwest::Method::GET,
                                &format!("http://localhost:{}/test/{}/index.html", variant.port, i),
                            )
                            .send();
                    })
                })
                .collect();

            match reqwest::Client::builder()
                .timeout(Some(BLOCK_TIME / 5))
                .build()
                .unwrap()
                .request(
                    reqwest::Method::GET,
                    &format!("http://localhost:{}/test/0/index.html", variant.port),
                )
                .send()
            {
                Ok(_) => {
                    ();
                }
                Err(e) => {
                    println!(
                        "{}: could handle {} threads. stopped with: {}",
                        variant.name, n, e
                    );
                    results.insert(variant.name, n);
                    done = true;
                }
            }

            if n >= max_threads {
                done = true;
                results.insert(variant.name, n);
            }

            client_threads.into_iter().for_each(|handle| {
                let _ = handle.join();
            });

            n += 1;
        }

        // enusre all threads have time to terminate
        std::thread::sleep(BLOCK_TIME * 2);
    }

    println!("{:#?}", results);

    Ok(())
}
