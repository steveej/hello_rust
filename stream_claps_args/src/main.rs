#[macro_use]
extern crate lazy_static;

use futures::Stream;
use tokio;

lazy_static! {}

fn main() -> Result<(), Box<std::error::Error>> {
    let matches = clap::App::new("foo")
        .arg(clap::Arg::with_name("FILE").multiple(true).required(true))
        .get_matches_from(vec!["foo", "a", "b", "c"]);

    let files: Vec<String> = matches
        .values_of("FILE")
        .unwrap()
        .map(|x| x.into())
        .collect();

    let stream = futures::stream::iter_ok::<_, ()>(files).for_each(|x| {
        println!("x: {}", x);
        futures::future::ok(())
    });

    tokio::run(stream);

    Ok(())
}
