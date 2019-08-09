#![allow(dead_code)]

use failure::Error;
use futures::IntoFuture;
use futures::{Future, Stream};
use futures_locks::Mutex as FuturesMutex;
use std::sync::Arc;

type AsyncResult<T> = Box<dyn Future<Item = T, Error = Error> + Send>;

type WorkIO = String;

trait AsyncWorker<T>
where
    Self: Sync + Send,
    T: Sync + Send + Clone,
    T: 'static,
{
    fn run(self: &mut Self, input: T) -> AsyncResult<T>;
}

trait AsyncWorkerInternal<T> {
    fn run_internal(self: &mut Self, input: WorkIO) -> AsyncResult<WorkIO>;
}

trait AsyncWorkerExternal<T> {
    fn run_external(self: &mut Self, input: WorkIO) -> AsyncResult<WorkIO>;
}

type Work = Arc<FuturesMutex<Box<dyn AsyncWorker<WorkIO>>>>;
type WorkCollection = Vec<Work>;

struct InternalWorkWrapper<T>(T);
struct ExternalWorkWrapper<T>(T);

impl<T> AsyncWorker<WorkIO> for InternalWorkWrapper<T>
where
    T: AsyncWorkerInternal<WorkIO>,
    T: Sync + Send + Clone,
{
    fn run(self: &mut Self, input: WorkIO) -> AsyncResult<WorkIO> {
        self.0.run_internal(input)
    }
}

impl<T> AsyncWorker<WorkIO> for ExternalWorkWrapper<T>
where
    T: AsyncWorkerExternal<WorkIO>,
    T: Sync + Send + Clone,
{
    fn run(self: &mut Self, input: WorkIO) -> AsyncResult<WorkIO> {
        self.0.run_external(input)
    }
}

fn process<T>(work_collection: T) -> AsyncResult<WorkIO>
where
    T: Iterator<Item = &'static Work>,
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

    #[derive(Clone)]
    struct InternalCountingForwarder(pub usize);
    impl AsyncWorkerInternal<WorkIO> for InternalCountingForwarder {
        fn run_internal(self: &mut Self, input: WorkIO) -> AsyncResult<WorkIO> {
            println!("Processing {}th run. Input {}", self.0, input);
            self.0 += 1;

            Box::new(futures::future::ok(format!("{}{}", input, self.0 % 10)))
        }
    }

    #[derive(Clone)]
    struct ExternalCountingForwarder(pub usize);
    impl AsyncWorkerExternal<WorkIO> for ExternalCountingForwarder {
        fn run_external(self: &mut Self, input: WorkIO) -> AsyncResult<WorkIO> {
            println!("Processing {}th run. Input {}", self.0, input);
            self.0 += 1;

            Box::new(futures::future::ok(format!("{}{}", input, self.0 % 10)))
        }
    }

    #[test]
    fn test_process() -> Fallible<()> {
        lazy_static! {
            static ref WORK_COLLECTION: WorkCollection = vec![
                Arc::new(FuturesMutex::new(Box::new(InternalWorkWrapper(
                    InternalCountingForwarder(0)
                )))),
                Arc::new(FuturesMutex::new(Box::new(ExternalWorkWrapper(
                    ExternalCountingForwarder(0)
                )))),
                Arc::new(FuturesMutex::new(Box::new(InternalWorkWrapper(
                    InternalCountingForwarder(0)
                )))),
                Arc::new(FuturesMutex::new(Box::new(ExternalWorkWrapper(
                    ExternalCountingForwarder(0)
                )))),
            ];
        }

        let mut runtime = tokio::runtime::Runtime::new().unwrap();

        for _ in 0..10 {
            let async_result = process(WORK_COLLECTION.iter());

            let result: WorkIO = runtime.block_on(async_result).expect("work failed");

            assert_eq!(result.len(), WORK_COLLECTION.len());
        }

        Ok(())
    }
}
