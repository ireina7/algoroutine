use std::{
    ops::{Coroutine, CoroutineState},
    pin::pin,
};

pub trait MapCoroutine<R>: Coroutine<R> {
    fn map<U>(
        self,
        f: impl FnOnce(Self::Return) -> U,
    ) -> impl Coroutine<R, Yield = Self::Yield, Return = U>;
}

pub trait BindCoroutine<R, F, Ef, T>: Coroutine<R> {
    fn and_then<E>(
        self,
        f: impl FnOnce(Self::Return) -> F,
    ) -> impl Coroutine<R, Yield = E, Return = T>
    where
        R: Clone,
        F: Coroutine<R, Yield = Ef, Return = T>,
        E: From<Self::Yield> + From<Ef>;
}

impl<G: Coroutine<R>, R> MapCoroutine<R> for G {
    fn map<U>(
        self,
        f: impl FnOnce(Self::Return) -> U,
    ) -> impl Coroutine<R, Yield = Self::Yield, Return = U> {
        fmap(self, f)
    }
}

impl<G: Coroutine<R>, R: Clone, F, Ef, T> BindCoroutine<R, F, Ef, T> for G {
    fn and_then<E>(
        self,
        f: impl FnOnce(Self::Return) -> F,
    ) -> impl Coroutine<R, Yield = E, Return = T>
    where
        R: Clone,
        F: Coroutine<R, Yield = Ef, Return = T>,
        E: From<Self::Yield> + From<Ef>,
    {
        bind(self, f)
    }
}

pub fn fmap<E, I, T, U>(
    g: impl Coroutine<I, Yield = E, Return = T>,
    f: impl FnOnce(T) -> U,
) -> impl Coroutine<I, Yield = E, Return = U> {
    #[coroutine]
    static move |mut injs: I| {
        let mut pinned = pin!(g);
        loop {
            match pinned.as_mut().resume(injs) {
                CoroutineState::Yielded(effs) => injs = yield effs,
                CoroutineState::Complete(ret) => return f(ret),
            }
        }
    }
}

pub fn map_effect<E1, E2, I, R, F: Coroutine<I, Yield = E1, Return = R>>(
    f: F,
) -> impl Coroutine<I, Yield = E2, Return = R>
where
    E2: From<E1>,
{
    #[coroutine]
    static move |mut arg: I| {
        let mut pinned = pin!(f);
        loop {
            match pinned.as_mut().resume(arg) {
                CoroutineState::Yielded(effs) => arg = yield E2::from(effs),
                CoroutineState::Complete(ret) => return ret,
            }
        }
    }
}

pub fn map_input<E, I1, I2, R, F: Coroutine<I1, Yield = E, Return = R>>(
    f: F,
) -> impl Coroutine<I2, Yield = E, Return = R>
where
    I1: From<I2>,
{
    #[coroutine]
    static move |arg: I2| {
        let mut arg = I1::from(arg);
        let mut pinned = pin!(f);
        loop {
            match pinned.as_mut().resume(arg) {
                CoroutineState::Yielded(effs) => arg = I1::from(yield effs),
                CoroutineState::Complete(ret) => return ret,
            }
        }
    }
}

fn bind<E1, E2, E3, F, G, I, T, U>(
    g: G,
    f: impl FnOnce(U) -> F,
) -> impl Coroutine<I, Yield = E3, Return = T>
where
    I: Clone,
    G: Coroutine<I, Yield = E1, Return = U>,
    F: Coroutine<I, Yield = E2, Return = T>,
    E3: From<E1> + From<E2>,
{
    #[coroutine]
    static move |mut injs: I| {
        let mut pinned = pin!(g);
        loop {
            match pinned.as_mut().resume(injs.clone()) {
                CoroutineState::Yielded(effs) => injs = yield E3::from(effs),
                CoroutineState::Complete(ret) => {
                    let mut pinned = pin!(f(ret));
                    loop {
                        match pinned.as_mut().resume(injs) {
                            CoroutineState::Yielded(effs) => injs = yield E3::from(effs),
                            CoroutineState::Complete(ret) => return ret,
                        }
                    }
                }
            }
        }
    }
}
