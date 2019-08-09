#![allow(dead_code)]

use failure::Error;
use futures::IntoFuture;
use futures::{Future, Stream};
use futures_locks::Mutex as FuturesMutex;
use std::sync::Arc;

type AsyncResult<T> = Box<dyn Future<Item = T, Error = Error> + Sync + Send>;
trait ChainedAsyncWork<T>
where
    Self: Sync + Send,
    T: Sync + Send + Clone,
    T: 'static,
{
    fn run(self: &mut Self, input: T) -> AsyncResult<T>;
}

type Work = Box<dyn ChainedAsyncWork<String>>;
type WorkMutex = Arc<FuturesMutex<Work>>;
type WorkCollection = Vec<WorkMutex>;
type WorkCollectionMutex = FuturesMutex<WorkCollection>;

fn process<T>(work_collection: T) -> AsyncResult<String>
where
    T: Iterator<Item = &'static WorkMutex>,
    T: Sync + Send,
    T: 'static,
{
    let future_work = futures::stream::iter_ok::<_, Error>(work_collection)
        .fold(
            futures::future::Either::A(Box::new(futures::future::ok("".to_string()))),
            |future_input, next_item_mutex| {
                println!("[] getting work lock...");
                next_item_mutex
                    .lock()
                    .map_err(|_| failure::err_msg("could not acquire the mutex lock"))
                    .join(future_input)
                    .map(|(mut next_item, input)| {
                        println!("[] got work lock!");
                        println!("[] input: {}", input);
                        futures::future::Either::B((*next_item).run(input))
                    })
            },
        )
        .into_future()
        .flatten();

    Box::new(future_work)
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::Fallible;

    struct CountingForwarder(pub usize);

    impl CountingForwarder {
        fn run_internal(self: &mut Self, input: String) -> AsyncResult<String> {
            println!("Processing {}th run. Input {}", self.0, input);
            self.0 += 1;

            Box::new(futures::future::ok(format!("{}{}", input, self.0 % 10)))
        }
    }

    impl ChainedAsyncWork<String> for CountingForwarder {
        fn run(self: &mut Self, io: String) -> AsyncResult<String> {
            self.run_internal(io)
        }
    }

    #[test]
    fn test_process() -> Fallible<()> {
        lazy_static! {
            static ref WORK_COLLECTION: WorkCollection = vec![
                Arc::new(FuturesMutex::new(Box::new(CountingForwarder { 0: 0 }))),
                Arc::new(FuturesMutex::new(Box::new(CountingForwarder { 0: 0 }))),
            ];
        }

        let mut runtime = tokio::runtime::Runtime::new().unwrap();

        for _ in 0..10 {
            let async_result = process(WORK_COLLECTION.iter());

            let result: String = runtime.block_on(async_result).expect("work failed");

            assert_eq!(result.len(), WORK_COLLECTION.len());
        }

        Ok(())
    }
}
