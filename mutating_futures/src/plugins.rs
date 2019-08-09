use failure::Fallible;
use std::collections::HashMap;
use futures::stream::Stream;
use std::sync::{Arc, Mutex};
use core::fmt::Debug;
use futures::Future;
use futures::future::IntoFuture;
use failure::Error;
use futures_locks::Mutex as FuturesMutex;

/// Convenience type to wrap other types in a Future
pub type FutureIO<'a, T> = Box<dyn Future<Item = T, Error = Error> + Send + 'a>;

// /// Convenience type for the thread-safe storage of plugins
pub type PluginReference = Box<dyn Plugin<String> + Sync + Send>;

/// Trait which fronts InternalPlugin and ExternalPlugin, allowing their trait objects to live in the same collection
pub trait Plugin<T>
where
    Self: Debug,
    T: Sync + Send,
{
    fn run(self: &Self, t: T) -> FutureIO<'static, T>;
}

/// Trait to be implemented by internal plugins with their native IO type
pub trait InternalPlugin
where
    Self: Debug,
{
    fn run_internal(self: &mut Self, input: String) -> FutureIO<String>;
}

/// Wrapper struct for a universal implementation of Plugin<PluginIO> for all InternalPlugin implementors
#[derive(Debug, Clone)]
pub struct InternalPluginWrapper<T>(Arc<FuturesMutex<T>>);

/// This implementation allows the process function to run ipmlementors of
/// InternalPlugin
impl<T> Plugin<String> for Arc<InternalPluginWrapper<T>>
where
    Self: Sync + Send + Clone + 'static,
    T: InternalPlugin,
    T: Sync + Send + Clone + 'static,
{
    fn run(self: &Self, plugin_io: String) -> FutureIO<'static, String> {
        // Box::new(futures::future::ok(plugin_io))

        Box::new(
            self.0.clone().lock()
                .map_err(|_| failure::err_msg("could not acquire the plugin mutex"))
                .map(|mut guard| {
                    futures::future::ok::<String, Error>(plugin_io)
                        .and_then(|internal_io| guard.run_internal(internal_io))
                        .and_then(|internal_io| {
                            let plugin_io: Fallible<String> = Ok(internal_io.to_owned());
                            plugin_io
                        })
                })
                .flatten(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct PluginProcessor
{
    plugins: Arc<Vec<Arc<PluginReference>>>,
}

impl PluginProcessor {
    /// Processes all given Plugins sequentially.
    ///
    /// This function automatically converts between the different IO representations
    /// if necessary.
    pub fn process<'a: 'b, 'b>(
        self: &'a Self,
        initial_io: String,
    ) -> FutureIO<'b, String> {
        let future_result = futures::stream::iter_ok(0..self.plugins.len())
            .fold(
                String::new(),
                move |last_io, next_plugin_index| match self.plugins.clone().get(next_plugin_index).clone() {
                    Some(next_plugin) => next_plugin.clone().run(last_io),
                    None => Box::new(futures::future::err(failure::err_msg(format!(
                        "could not find plugin at index {}",
                        next_plugin_index
                    )))),
                }
            )
            .into_future()
            .and_then(|final_io| {
                Ok(final_io)
            });

        Box::new(future_result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestInternalPlugin {
        counter: usize,
        dict: HashMap<usize, bool>,
    }

    impl InternalPlugin for TestInternalPlugin {
        fn run_internal(self: &mut Self, io: String) -> FutureIO<String> {
            self.counter += 1;
            self.dict.insert(self.counter, true);

            Box::new(futures::future::ok(io))
        }
    }

    impl Plugin<String> for TestInternalPlugin {
        fn run(self: Box<Self>, io: String) -> FutureIO<'static, String> {
            Box::new(futures::future::ok(io))
        }
    }

    #[test]
    fn process_plugins_with_state() {
        let initial_io = String::new();

        let plugins: Arc<Vec<PluginReference>> = Arc::new(vec![
            Arc::new(InternalPluginWrapper(
                Arc::new(FuturesMutex::new(
                    TestInternalPlugin {
                        counter: Default::default(),
                        dict: Default::default(),
                    },
                ))
            )),
            // Arc::new(FuturesMutex::new(Box::new(InternalPluginWrapper(
            //     TestInternalPlugin {
            //         counter: Default::default(),
            //         dict: Default::default(),
            //     },
            // )))),
        ]);

        let plugin_processor = PluginProcessor { plugins };

        let runs: usize = 10;
        for _ in 0..runs {
            let initial_io = initial_io.clone();

            let plugins_future: FutureIO<String> =
                plugin_processor.clone().process(initial_io.clone());

            let _ = tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(plugins_future)
                .expect("plugin processing failed");
        }

        // assert_eq!(runs, counter.load(Ordering::SeqCst));
        // assert!(dict.read().unwrap().get(&runs).unwrap());
    }
}