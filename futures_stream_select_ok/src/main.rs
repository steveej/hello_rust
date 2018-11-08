extern crate futures;
extern crate tokio;

use futures::prelude::*;

fn delay<'a>(millis: u64) -> impl Future<Item = u64, Error = tokio::timer::Error> {
    let when = std::time::Instant::now() + std::time::Duration::from_millis(millis);
    tokio::timer::Delay::new(when).and_then(move |_| Ok(millis))
}

fn main() {
    let vec = vec![300u64, 200, 100];
    let vec_futures = futures::future::select_ok(vec.iter().map(|n| delay(*n)).collect::<Vec<_>>())
        .map(|(winner, _)| winner);

    let result = tokio::runtime::Runtime::new()
        .unwrap()
        .block_on_all(vec_futures)
        .unwrap();

    println!("{:?}", result);
}
