#![allow(dead_code)]

use core::fmt::Display;
use core::fmt::Formatter;
use failure::Error;
use futures::IntoFuture;
use futures::{Future, Stream};
use futures_locks::Mutex as FuturesMutex;
use std::sync::Arc;

type AsyncWorkIO<T> = Box<dyn Future<Item = T, Error = Error> + Send>;

struct InternalWorkIO(pub String);
struct ExternalWorkIO(pub String);
enum WorkIO {
    InternalWorkIO(InternalWorkIO),
    ExternalWorkIO(ExternalWorkIO),
}

impl WorkIO {
    fn get_string(self: &Self) -> String {
        match self {
            WorkIO::InternalWorkIO(io) => io.0.clone(),
            WorkIO::ExternalWorkIO(io) => io.0.clone(),
        }
    }
}

impl Display for InternalWorkIO {
    fn fmt(self: &Self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{}", self.0)
    }
}
impl Display for ExternalWorkIO {
    fn fmt(self: &Self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{}", self.0)
    }
}
impl Display for WorkIO {
    fn fmt(self: &Self, formatter: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(formatter, "{}", self.get_string())
    }
}

trait AsyncWorker<T>
where
    Self: Sync + Send,
    T: Sync + Send,
{
    fn run(self: &mut Self, input: T) -> AsyncWorkIO<T>;
}

trait AsyncWorkerInternal {
    fn run_internal(self: &mut Self, input: InternalWorkIO) -> AsyncWorkIO<InternalWorkIO>;
}

trait AsyncWorkerExternal {
    fn run_external(self: &mut Self, input: ExternalWorkIO) -> AsyncWorkIO<ExternalWorkIO>;
}

type Work = Arc<FuturesMutex<Box<dyn AsyncWorker<WorkIO>>>>;
type WorkCollection = Vec<Work>;

struct InternalWorkWrapper<T>(T);
struct ExternalWorkWrapper<T>(T);

impl<T> AsyncWorker<WorkIO> for InternalWorkWrapper<T>
where
    T: AsyncWorkerInternal,
    T: Sync + Send,
{
    fn run(self: &mut Self, input: WorkIO) -> AsyncWorkIO<WorkIO> {
        Box::new(
            self.0
                .run_internal(match input {
                    WorkIO::InternalWorkIO(io) => io,
                    WorkIO::ExternalWorkIO(io) => InternalWorkIO(io.0),
                })
                .map(WorkIO::InternalWorkIO),
        )
    }
}

impl<T> AsyncWorker<WorkIO> for ExternalWorkWrapper<T>
where
    T: AsyncWorkerExternal,
    T: Sync + Send,
{
    fn run(self: &mut Self, input: WorkIO) -> AsyncWorkIO<WorkIO> {
        Box::new(
            self.0
                .run_external(match input {
                    WorkIO::InternalWorkIO(io) => ExternalWorkIO(io.0),
                    WorkIO::ExternalWorkIO(io) => io,
                })
                .map(WorkIO::ExternalWorkIO),
        )
    }
}

fn process<T>(work_collection: T, initial_io: WorkIO) -> AsyncWorkIO<String>
where
    T: Iterator<Item = &'static Work>,
    T: Sync + Send,
    T: 'static,
{
    let future_work = futures::stream::iter_ok::<_, Error>(work_collection)
        .fold(
            futures::future::Either::A(futures::future::ok(initial_io)),
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
        .flatten()
        .map(|io| io.get_string());

    Box::new(future_work)
}

#[cfg(test)]
mod tests {
    use super::*;
    use failure::Fallible;

    struct InternalCountingForwarder(pub usize);

    impl AsyncWorkerInternal for InternalCountingForwarder {
        fn run_internal(self: &mut Self, input: InternalWorkIO) -> AsyncWorkIO<InternalWorkIO> {
            println!("Processing {}th run. Input {}", self.0, input);
            self.0 += 1;

            Box::new(futures::future::ok(InternalWorkIO(format!(
                "{}{}",
                input,
                self.0 % 10
            ))))
        }
    }

    struct ExternalCountingForwarder(pub usize);
    impl AsyncWorkerExternal for ExternalCountingForwarder {
        fn run_external(self: &mut Self, input: ExternalWorkIO) -> AsyncWorkIO<ExternalWorkIO> {
            println!("Processing {}th run. Input {}", self.0, input);
            self.0 += 1;

            Box::new(futures::future::ok(ExternalWorkIO(format!(
                "{}{}",
                input,
                self.0 % 10
            ))))
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
            let async_result = process(
                WORK_COLLECTION.iter(),
                WorkIO::InternalWorkIO(InternalWorkIO("".to_string())),
            );

            let result: String = runtime.block_on(async_result).expect("work failed");

            assert_eq!(result.len(), WORK_COLLECTION.len());
        }

        Ok(())
    }
}
