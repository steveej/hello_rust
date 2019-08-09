#![allow(dead_code)]

use failure::{Error, Fallible};
use futures::IntoFuture;
use futures::{Future, Stream};
use futures_locks::Mutex as FuturesMutex;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::MutexGuard;

type AsyncResult<T: Sized + Sync + Send> = Box<dyn Future<Item = T, Error = Error> + Sync + Send>;
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

#[cfg(test)]
mod tests {
    use super::*;

    struct CountingForwarder(pub usize);

    impl CountingForwarder {
        fn run_internal(self: &mut Self, input: String) -> Fallible<String> {
            self.0 += 1;
            println!("Processed {} requests", self.0);

            Ok(input)
        }
    }

    impl ChainedAsyncWork<String> for CountingForwarder {
        fn run(self: &mut Self, io: String) -> AsyncResult<String> {
            Box::new(futures::future::ok(self.run_internal(io).unwrap()))
        }
    }

    #[test]
    fn test_process() -> Fallible<()> {
        lazy_static! {
            static ref WORK_COLLECTION_MUTEX: WorkCollectionMutex =
                FuturesMutex::new(vec![Arc::new(FuturesMutex::new(Box::new(
                    CountingForwarder { 0: 0 }
                )))]);
            static ref IO_MUTEX: FuturesMutex<String> =
                FuturesMutex::new("please work".to_string());
        }

        let mut runtime = tokio::runtime::current_thread::Runtime::new().unwrap();

        for i in 0..10 {
            println!("[{}] starting iteration", i);
            let future_work_collection_locked = WORK_COLLECTION_MUTEX
                .lock()
                .map_err(|_| failure::err_msg("could not acquire the mutex lock"));

            let future_work = future_work_collection_locked.map(move |work_collection_guard| {
                println!(
                    "[{}] got the collection lock, leaking the guard for now...",
                    i
                );

                let work_collection_guard = Box::leak(Box::new(work_collection_guard));
                let work_collection_guard_leaked =
                    Box::into_raw(Box::new(work_collection_guard.clone()));

                futures::stream::iter_ok::<_, Error>(work_collection_guard.iter())
                    .fold(
                        futures::future::Either::A(Box::new(futures::future::ok(
                            "please work :-)".to_string(),
                        ))),
                        move |future_input, next_item_mutex| {
                            println!("[] getting work lock...");
                            let next_item_result = next_item_mutex
                                .lock()
                                .map_err(|_| failure::err_msg("could not acquire the mutex lock"))
                                .join(future_input)
                                .map(|(mut next_item, input)| {
                                    println!("[] got work lock!");
                                    futures::future::Either::B((*next_item).run(input))
                                });

                            drop(unsafe { Box::from_raw(work_collection_guard_leaked) });

                            next_item_result
                        },
                    )
                    .into_future()
            });

            let async_result = Box::new(future_work);

            let _ = runtime.block_on(async_result).expect("work failed");
        }

        Ok(())
    }
}
