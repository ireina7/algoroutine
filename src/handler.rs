use std::{ops::Coroutine, pin::Pin};

pub enum OneStep<I, R> {
    Return(R),
    Yield(I),
}

/// Consume whe whole coroutine until finished
pub trait Consumer<E, I, R> {
    fn consume<F>(&mut self, continuation: F, arg: I) -> F::Return
    where
        F: Coroutine<I, Yield = E, Return = R> + 'static;
}

/// Step a coroutine until it yields or return
pub trait Step<E, I, R>: Sized {
    fn step<F>(&mut self, continuation: Pin<&mut F>, arg: I) -> OneStep<I, R>
    where
        F: Coroutine<I, Yield = E, Return = R> + 'static;
}

/// Build a sync consumer based on `Step`
pub struct SyncConsumer<S> {
    step: S,
}

impl<S> SyncConsumer<S> {
    pub fn from(step: S) -> Self {
        Self { step }
    }
}

impl<E, I, R, S> Consumer<E, I, R> for SyncConsumer<S>
where
    S: Step<E, I, R>,
{
    fn consume<F>(&mut self, continuation: F, mut arg: I) -> F::Return
    where
        F: Coroutine<I, Yield = E, Return = R> + 'static,
    {
        let mut f = Box::pin(continuation);
        loop {
            let step = self.step.step(f.as_mut(), arg);
            match step {
                OneStep::Return(ans) => break ans,
                OneStep::Yield(x) => arg = x,
            }
        }
    }
}
