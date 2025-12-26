//! This module provides abstract implementation of stateful "computations" and "generators".
//! These are generally similar to asynchronous operations but allow us to optimize certain
//! behavior for CPU-bound tasks and resumable computations.
//!
//! The core idea is that every [`Computation`] has:
//!  - `CONTEXT`; some immutable object that provides configuration for the computation.
//!  - `STATE`; a mutable object that is changed by repeatedly invoking [`ComputationStep`].
//!  - `OUTPUT`; the type of data that `STATE` is converted to once the computation is done.
//!
//! The [`Generator`] is very similar, except it behaves like an iterator. The `OUTPUT` items
//! are produced gradually as a byproduct of invoking [`GeneratorStep`]. Each [`Generator`]
//! can be also converted into a [`Computation`] that collects the values into a container
//! (similar to `collect` on iterators).
//!
//! ## Basic computation
//!
//! ```rust
//! # use cancel_this::Cancellable;
//! # use biodivine_lib_algo_scc::algorithm_trait::{Completable, ComputationStep, Incomplete, Computation, Derived};
//! struct Counter;
//!
//! impl ComputationStep<usize, usize, usize> for Counter {
//!     fn step(context: &usize, state: &mut usize) -> Completable<()> {
//!         if *state >= *context {
//!             Ok(())
//!         } else {
//!             *state = *state + 1;
//!             Err(Incomplete::Working)
//!         }
//!     }
//! }
//!
//! // Type arguments of `Computation`:
//! // 1: The `CONTEXT`, here the target counter value.
//! // 2: The `STATE`, here the current counter value.
//! // 3: The `OUTPUT`, here the final count.
//! // 4: The actual "step operator" that performs state mutation.
//! type CounterComputation = Computation<usize, usize, usize, Counter>;
//!
//! // We can "run" the computation as a cancellable function, initialized with context and state:
//! assert_eq!(CounterComputation::run(10usize, 0usize).unwrap(), 10);
//!
//! // We can also create the computation object that we can gradually poll until completion:
//! let mut computation = CounterComputation::configure(6usize, 3usize);
//! assert_eq!(computation.try_compute(), Err(Incomplete::Working));  // 4
//! assert_eq!(computation.try_compute(), Err(Incomplete::Working));  // 5
//! assert_eq!(computation.try_compute(), Err(Incomplete::Working));  // 6
//! assert_eq!(computation.try_compute(), Ok(&6));  // done
//! ```
//!
//! ## Output type conversion
//!
//! In the example above,
//!
//! ## Implementation
//!
//! The operation is split into two methods: First, `step` is repeatedly called with `CONTEXT`
//! and mutable `STATE` until "completion" (`Ok(())` is returned). Then, `output` is called
//! once to convert owned `STATE` into `OUTPUT` (also with access to `CONTEXT`).
//!
//!  > Implementations should be robust towards calling `step` even after `()` was already
//!  > returned. It is still allowed to return any number of `Working` values as long as the
//!  > implementation eventually returns `()` again (assuming the operation is not canceled).
//!
//!  > Implementations are allowed to panic if `output` is called before `step` returned `()`.
//!  > However, if at all possible, it is preferred that in such a situation, a partial "incomplete"
//!  > result is returned instead.
//!
//! For example, here is a simple implementation of a "counter":
//!
//!
//! ## Background
//!
//! There is a wide range of reasons for using this design:
//!
//!  - The `step` function can't directly take ownership of `state`, because it could be "lost"
//!    if the operation is canceled (i.e., it would not be possible to resume computation).
//!    Furthermore, design where `step` owns `state` is in general quite hard to work with.
//!  - The `output` function cannot be cancellable for exactly this reason (state would be lost
//!    if canceled during conversion). If you have any complex data conversions you need to
//!    perform, these should be done by the `step` function.
//!  - Using `From`/`Into` instead of `output` causes issues with blanket vs. custom
//!    conversions, plus pollutes the code with many trait bounds.
//!  - Sometimes, having access to `CONTEXT` during type conversion can be necessary.
//!  - If the conversion is trivial (i.e., `STATE` implements `Into<OUTPUT>`), the conversion
//!    can be done automatically by implementing [`ComputationStep`]. However, for complex
//!    conversions, there is always the option to implement [`ComputationStepAndConvert`].
//!  - This automatic/manual conversion is indicated by the `STRATEGY` type parameter, which
//!    otherwise does nothing except for allowing the blanket implementation of this trait.
//!  - Being able to take ownership of `state` for the `output` conversion allows performing
//!    "zero copy" conversions instead of requiring that `OUTPUT` is cloned out of the
//!    computation state reference.
//!  - `CONTEXT` provides a way to "configure" the algorithm. If your algorithm does not need
//!    any configuration, you can use `()` as `CONTEXT`.

use cancel_this::{Cancellable, Cancelled};
use std::fmt::{Display, Formatter};

mod computation;
mod generator;

pub use computation::{Computation, ComputationStep, ComputationStepAndConvert, Derived, Manual};
pub use generator::{CollectorStep, Generator, GeneratorStep};

pub trait Algorithm<CONTEXT, STATE, OUTPUT>: Computable<OUTPUT> {
    fn configure<I1: Into<CONTEXT>, I2: Into<STATE>>(context: I1, initial_state: I2) -> Self
    where
        Self: Sized;

    fn configure_dyn<I1: Into<CONTEXT>, I2: Into<STATE>>(
        context: I1,
        initial_state: I2,
    ) -> DynAlgorithm<CONTEXT, STATE, OUTPUT>
    where
        Self: Sized + 'static,
    {
        Box::new(Self::configure(context, initial_state))
    }
}

pub trait GenAlgorithm<CONTEXT, STATE, OUTPUT>: Generatable<OUTPUT> {
    fn configure<I1: Into<CONTEXT>, I2: Into<STATE>>(context: I1, initial_state: I2) -> Self
    where
        Self: Sized;

    fn configure_dyn<I1: Into<CONTEXT>, I2: Into<STATE>>(
        context: I1,
        initial_state: I2,
    ) -> DynGenAlgorithm<CONTEXT, STATE, OUTPUT>
    where
        Self: Sized + 'static,
    {
        Box::new(Self::configure(context, initial_state))
    }
}

pub trait Computable<T> {
    fn try_compute(&mut self) -> Completable<&T>;
    fn compute(self) -> Cancellable<T>;
}

pub trait Generatable<T>: Iterator<Item = Cancellable<T>> {
    fn try_next(&mut self) -> Option<Completable<T>>;
}

pub type DynComputable<T> = Box<dyn Computable<T>>;
pub type DynAlgorithm<CONTEXT, STATE, OUTPUT> = Box<dyn Algorithm<CONTEXT, STATE, OUTPUT>>;
pub type DynGenAlgorithm<CONTEXT, STATE, OUTPUT> = Box<dyn GenAlgorithm<CONTEXT, STATE, OUTPUT>>;

/// A [`Completable`] result is a value that is eventually computed by an algorithm, but
/// the computation can be incomplete when the value is polled.
pub type Completable<T> = Result<T, Incomplete>;

/// The error type returned by an algorithm when the result is not available.
///
/// The result can be unavailable because the computation was canceled or because the algorithm
/// has not finished yet but reached one of its pre-defined interruption points.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Incomplete {
    Working,
    Cancelled(Cancelled),
}

impl From<Cancelled> for Incomplete {
    fn from(value: Cancelled) -> Self {
        Incomplete::Cancelled(value)
    }
}

impl Display for Incomplete {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Incomplete::Working => write!(f, "Computation not finished"),
            Incomplete::Cancelled(c) => write!(f, "{}", c),
        }
    }
}

impl std::error::Error for Incomplete {}

#[cfg(test)]
mod tests {
    use crate::algorithm_trait::{
        Completable, Computation, ComputationStep, Generator, GeneratorStep, Incomplete,
    };

    #[test]
    fn simple_computation() {
        /// A simple "counter" that iterates until the limit given by `context` is reached.
        ///
        /// Note that the output state conversion is implemented automatically.
        struct CounterStep;
        impl ComputationStep<usize, usize, usize> for CounterStep {
            fn step(context: &usize, state: &mut usize) -> Completable<()> {
                if *state >= *context {
                    Ok(())
                } else {
                    *state = *state + 1;
                    Err(Incomplete::Working)
                }
            }
        }

        type CounterComputation = Computation<usize, usize, usize, CounterStep>;
        let result = CounterComputation::run(10usize, 0usize).unwrap();
        assert_eq!(result, 10);
    }

    #[test]
    fn foo() {
        struct TestStep;
        impl GeneratorStep<(), usize, usize> for TestStep {
            fn step(_context: &(), state: &mut usize) -> Completable<Option<usize>> {
                if *state >= 100 {
                    Ok(None)
                } else {
                    *state += 1;
                    Ok(Some(*state))
                }
            }
        }

        type NumberGenerator = Generator<(), usize, usize, TestStep>;

        let generator = NumberGenerator::configure((), 10usize);
        let computation = generator.computation::<Vec<_>>();
        let result = computation.compute().unwrap();
        println!("Computation result: {:?}", result);

        let generator = NumberGenerator::configure((), 20usize);

        for item in generator {
            println!("{:?}", item.unwrap());
        }
    }
}
