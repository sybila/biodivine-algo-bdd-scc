use crate::algorithm_trait::{
    Completable, Computation, ComputationStepAndConvert, GenAlgorithm, Generatable, Incomplete,
    Manual,
};
use cancel_this::Cancellable;
use std::marker::PhantomData;

/// Same as [`ComputationStepAndConvert`], but can produce additional `OUTPUT` items
/// repeatedly.
///
/// After the final value is returned, it should consistently return `None`,
/// possibly interleaved with [`Incomplete::Working`].
pub trait GeneratorStep<CONTEXT, STATE, OUTPUT> {
    fn step(context: &CONTEXT, state: &mut STATE) -> Completable<Option<OUTPUT>>;
}

/// Same as [`Computation`], except that it can produce multiple items
/// in a sequence, similar to asynchronous iterators.
///
/// Compared to [`Computation`], it does not provide an option to store the
/// result for future use. Instead, the caller is responsible for collecting all computed values
/// if they are meant to be retained.
pub struct Generator<CONTEXT, STATE, OUTPUT, STEP: GeneratorStep<CONTEXT, STATE, OUTPUT>> {
    context: CONTEXT,
    state: STATE,
    _output: PhantomData<OUTPUT>,
    _step: PhantomData<STEP>,
}

impl<CONTEXT, STATE, OUTPUT, STEP: GeneratorStep<CONTEXT, STATE, OUTPUT>>
    Generator<CONTEXT, STATE, OUTPUT, STEP>
{
    pub fn configure<I1: Into<CONTEXT>, I2: Into<STATE>>(context: I1, initial_state: I2) -> Self {
        Generator {
            context: context.into(),
            state: initial_state.into(),
            _output: PhantomData,
            _step: PhantomData,
        }
    }

    pub fn try_next(&mut self) -> Option<Completable<OUTPUT>> {
        STEP::step(&self.context, &mut self.state).transpose()
    }

    pub fn computation<COLLECTION: Default + Extend<OUTPUT>>(
        self,
    ) -> CollectorComputation<CONTEXT, STATE, OUTPUT, STEP, COLLECTION> {
        Computation::configure(
            (),
            CollectorState {
                collector: COLLECTION::default(),
                generator: self,
            },
        )
    }
}

impl<CONTEXT, STATE, OUTPUT, STEP: GeneratorStep<CONTEXT, STATE, OUTPUT>> Iterator
    for Generator<CONTEXT, STATE, OUTPUT, STEP>
{
    type Item = Cancellable<OUTPUT>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.try_next() {
                None => return None,
                Some(Ok(output)) => return Some(Ok(output)),
                Some(Err(Incomplete::Cancelled(c))) => return Some(Err(c)),
                Some(Err(Incomplete::Working)) => continue,
            }
        }
    }
}

/// A type of [`Computation`] which uses [`CollectorState`] to gather all results
/// of a [`Generator`] into a single `COLLECTION`.
type CollectorComputation<CONTEXT, STATE, OUTPUT, STEP, COLLECTION> = Computation<
    (),
    CollectorState<CONTEXT, STATE, OUTPUT, STEP, COLLECTION>,
    COLLECTION,
    CollectorStep,
    Manual,
>;

pub struct CollectorStep();

pub struct CollectorState<
    CONTEXT,
    STATE,
    OUTPUT,
    STEP: GeneratorStep<CONTEXT, STATE, OUTPUT>,
    COLLECTION: Default + Extend<OUTPUT>,
> {
    collector: COLLECTION,
    generator: Generator<CONTEXT, STATE, OUTPUT, STEP>,
}

impl<
    CONTEXT,
    STATE,
    OUTPUT,
    STEP: GeneratorStep<CONTEXT, STATE, OUTPUT>,
    COLLECTION: Default + Extend<OUTPUT>,
>
    ComputationStepAndConvert<
        (),
        CollectorState<CONTEXT, STATE, OUTPUT, STEP, COLLECTION>,
        COLLECTION,
    > for CollectorStep
{
    fn step(
        _context: &(),
        state: &mut CollectorState<CONTEXT, STATE, OUTPUT, STEP, COLLECTION>,
    ) -> Completable<()> {
        // Advance the inner generator by one step, outputting the whole collection
        // whenever the computation is finally done.
        match state.generator.try_next() {
            None => Ok(()),
            Some(Ok(output)) => {
                state.collector.extend(std::iter::once(output));
                Err(Incomplete::Working)
            }
            Some(Err(Incomplete::Working)) => Err(Incomplete::Working),
            Some(Err(Incomplete::Cancelled(c))) => Err(Incomplete::Cancelled(c)),
        }
    }

    fn output(
        _context: &(),
        state: CollectorState<CONTEXT, STATE, OUTPUT, STEP, COLLECTION>,
    ) -> COLLECTION {
        state.collector
    }
}

impl<CONTEXT, STATE, OUTPUT, STEP: GeneratorStep<CONTEXT, STATE, OUTPUT>> Generatable<OUTPUT>
    for Generator<CONTEXT, STATE, OUTPUT, STEP>
{
    fn try_next(&mut self) -> Option<Completable<OUTPUT>> {
        Generator::try_next(self)
    }
}

impl<CONTEXT, STATE, OUTPUT, STEP: GeneratorStep<CONTEXT, STATE, OUTPUT>>
    GenAlgorithm<CONTEXT, STATE, OUTPUT> for Generator<CONTEXT, STATE, OUTPUT, STEP>
{
    fn configure<I1: Into<CONTEXT>, I2: Into<STATE>>(context: I1, initial_state: I2) -> Self
    where
        Self: Sized,
    {
        Generator::configure(context, initial_state)
    }
}
