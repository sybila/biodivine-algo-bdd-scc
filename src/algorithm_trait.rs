use cancel_this::{Cancellable, is_cancelled};

/// The [`Algorithm`] trait is implemented by types that are intended to represent a computation
/// of a single "result".
///
/// In that sense, they are similar to futures or other asynchronous computation mechanisms.
/// However, using a custom definition allows us to include several niceties that are quite
/// hard to do well in generic `async` code. This specifically includes *serialization of
/// algorithm state*, *low-overhead cooperative cancellation* and *explicit interleaving
/// of multiple computations*.
///
/// ## Implementation comments
///  - Each algorithm object should be relatively simple and *do a single thing*. Prefer
///    composition of algorithms instead of building complicated state machines.
///  - Do not overuse the pattern for trivial computations. The main "reason" for using an
///    [`Algorithm`] instead of a simple cancellable function is either (a) several computations
///    need to be interleaved, or (b), the intermediate computation state needs to be serializable.
///  - All configuration must come as part of the initial state object. While it is advised to
///    define custom algorithm state types, it is also preferred to avoid extensive configuration
///    options to avoid overly complicated algorithm initialization. If you must provide complex
///    configuration, it is advised to use a `Builder` pattern which terminates with creation
///    of the algorithm object.
pub trait Algorithm {
    type State;
    type Output;

    /// Create a new instance of [`Algorithm`] from an initial state object.
    fn create(initial_state: Self::State) -> Self
    where
        Self: Sized;

    /// Advance this instance of [`Algorithm`] by "one computation step".
    ///
    /// If the algorithm has finished computing, this method should always return a fresh instance
    /// of the final result. However, it is also allowed to perform some computation to verify
    /// this is indeed the final result, so don't treat algorithm objects as storage.
    ///
    /// Note that algorithm can be also canceled while computing a single step. As such, this
    /// is not intended to define cancellation points. Instead, it defines serialization points
    /// where the state of the algorithm can be saved and restored.
    fn advance(&mut self) -> Cancellable<Option<Self::Output>>;

    /// Run this instance of [`Algorithm`] until completion by repeatedly
    /// calling [`Self::advance`].
    fn run(&mut self) -> Cancellable<Self::Output> {
        loop {
            is_cancelled!()?;
            if let Some(output) = self.advance()? {
                return Ok(output);
            }
        }
    }

    /// Run this algorithm as a single ongoing (but cancellable) computation.
    fn compute(initial_state: Self::State) -> Cancellable<Self::Output>
    where
        Self: Sized,
    {
        Self::create(initial_state).run()
    }
}
