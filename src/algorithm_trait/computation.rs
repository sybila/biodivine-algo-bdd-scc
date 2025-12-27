use crate::algorithm_trait::{Algorithm, Completable, Computable, Incomplete};
use cancel_this::{Cancellable, is_cancelled};
use std::marker::PhantomData;

/// Implemented by stateless objects that define "completable computations", i.e., operations that
/// repeatedly update a `STATE`, eventually producing an `OUTPUT` value (while having
/// access to some immutable `CONTEXT` object).
pub trait ComputationStepAndConvert<CONTEXT, STATE, OUTPUT, STRATEGY = Manual> {
    /// Advance the computation by one step by mutating `state`. If the computation is
    /// completed, return `()`.
    fn step(context: &CONTEXT, state: &mut STATE) -> Completable<()>;

    /// An uninterruptible conversion from `STATE` to `OUTPUT`.
    ///
    /// Any long-running cancellable operations should be performed within `step`.
    fn output(context: &CONTEXT, state: STATE) -> OUTPUT;
}

pub struct Manual;
pub struct Derived;

pub trait ComputationStep<CONTEXT, STATE: Into<OUTPUT>, OUTPUT> {
    fn step(context: &CONTEXT, state: &mut STATE) -> Completable<()>;
}

impl<CONTEXT, STATE: Into<OUTPUT>, OUTPUT, T: ComputationStep<CONTEXT, STATE, OUTPUT>>
    ComputationStepAndConvert<CONTEXT, STATE, OUTPUT, Derived> for T
{
    fn step(context: &CONTEXT, state: &mut STATE) -> Completable<()> {
        T::step(context, state)
    }

    fn output(_context: &CONTEXT, state: STATE) -> OUTPUT {
        state.into()
    }
}

/// An object that uses [`ComputationStepAndConvert`] to repeatedly mutate some `STATE` until an `OUTPUT`
/// is produced, at which point it stores the `OUTPUT` value for future use.
///
/// The computation can be configured via a `CONTEXT` object that is available during every
/// `STATE` update.
pub struct Computation<
    CONTEXT,
    STATE,
    OUTPUT,
    STEP: ComputationStepAndConvert<CONTEXT, STATE, OUTPUT, STRATEGY>,
    STRATEGY = Derived,
> {
    context: CONTEXT,
    state: Option<STATE>,
    output: Option<OUTPUT>,
    _marker: PhantomData<(STRATEGY, STEP)>,
}

impl<
    CONTEXT,
    STATE,
    OUTPUT,
    STEP: ComputationStepAndConvert<CONTEXT, STATE, OUTPUT, STRATEGY>,
    STRATEGY,
> Computation<CONTEXT, STATE, OUTPUT, STEP, STRATEGY>
{
    /// Initialize a [` Computation `] object by providing values that convert to
    /// configuration `CONTEXT` and the initial `STATE`.
    pub fn configure<I1: Into<CONTEXT>, I2: Into<STATE>>(context: I1, initial_state: I2) -> Self {
        Computation {
            context: context.into(),
            state: Some(initial_state.into()),
            output: None,
            _marker: PhantomData,
        }
    }

    pub fn try_compute(&mut self) -> Completable<&OUTPUT> {
        if let Some(state) = self.state.as_mut() {
            STEP::step(&self.context, state)?;
        }

        // At this point, `step` must have returned `()`.

        if let Some(state) = self.state.take() {
            self.output = Some(STEP::output(&self.context, state));
        }

        // At this point, output should be computed. The only way to get here is if `step`
        // returns a value, at which point the state is converted into output.
        Ok(self
            .output_ref()
            .expect("Correctness violation: When computation is done, output must be available."))
    }

    pub fn compute(mut self) -> Cancellable<OUTPUT> {
        loop {
            is_cancelled!()?;
            let advance = self.try_compute();
            match advance {
                Err(Incomplete::Working) => continue,
                Err(Incomplete::Cancelled(x)) => return Err(x),
                Ok(_) => break,
            }
        }

        // The only way to get here is when `try_compute` returns a value, at which point we know
        // the computation is complete.
        Ok(self
            .output()
            .expect("Correctness violation: When computation is done, output must be available."))
    }

    pub fn run<I1: Into<CONTEXT>, I2: Into<STATE>>(
        context: I1,
        initial_state: I2,
    ) -> Cancellable<OUTPUT> {
        Self::configure(context, initial_state).compute()
    }

    pub fn output_ref(&self) -> Option<&OUTPUT> {
        self.output.as_ref()
    }

    pub fn output(self) -> Option<OUTPUT> {
        self.output
    }

    pub fn state_ref(&self) -> Option<&STATE> {
        self.state.as_ref()
    }

    pub fn state(self) -> Option<STATE> {
        self.state
    }

    pub fn context_ref(&self) -> &CONTEXT {
        &self.context
    }
}

impl<
    CONTEXT,
    STATE,
    OUTPUT,
    STEP: ComputationStepAndConvert<CONTEXT, STATE, OUTPUT, STRATEGY>,
    STRATEGY,
> Computable<OUTPUT> for Computation<CONTEXT, STATE, OUTPUT, STEP, STRATEGY>
{
    fn try_compute(&mut self) -> Completable<&OUTPUT> {
        Computation::try_compute(self)
    }

    fn compute(self) -> Cancellable<OUTPUT> {
        Computation::compute(self)
    }
}

impl<
    CONTEXT,
    STATE,
    OUTPUT,
    STEP: ComputationStepAndConvert<CONTEXT, STATE, OUTPUT, STRATEGY>,
    STRATEGY,
> Algorithm<CONTEXT, STATE, OUTPUT> for Computation<CONTEXT, STATE, OUTPUT, STEP, STRATEGY>
{
    fn configure<I1: Into<CONTEXT>, I2: Into<STATE>>(context: I1, initial_state: I2) -> Self
    where
        Self: Sized,
    {
        Computation::configure(context, initial_state)
    }
}
