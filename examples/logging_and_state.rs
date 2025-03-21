#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]

use std::{
    marker::PhantomData,
    ops::{Coroutine, CoroutineState},
};

use algoroutine::{go, handler::Handler as _};

fn main() {
    let prepare = #[coroutine]
    |_: Option<i32>| {
        go!(Log("preparing".into()) => Effect);
        return ResultCode(0);
    };

    let logic = #[coroutine]
    |_: Option<i32>| {
        go!(Log("start".into()) => Effect);

        go!(State::Set(Some(9)));
        let s = go!(State::Get);
        go!(Log(format!("Got {:?}", s)));

        let res = go!(prepare, None);
        if !res.is_ok() {
            go!(Log(format!("error code: {}", res.0)));
        }
        return res;
    };

    let mut handler: Handler<Option<i32>> = Handler::new();
    let ans = handler.handle(None, logic);
    dbg!(ans);
}

struct Log(String);
enum State<S> {
    Get,
    Set(Option<S>),
}

enum Effect<S> {
    Log(Log),
    State(State<S>),
}

impl<S> From<Log> for Effect<S> {
    fn from(value: Log) -> Self {
        Effect::Log(value)
    }
}

impl<S> From<State<S>> for Effect<S> {
    fn from(value: State<S>) -> Self {
        Effect::State(value)
    }
}

#[derive(Debug)]
struct ResultCode(i32);

impl ResultCode {
    pub fn is_ok(&self) -> bool {
        self.0 == 0
    }
}

impl From<Option<i32>> for ResultCode {
    fn from(value: Option<i32>) -> Self {
        ResultCode(value.unwrap_or(0))
    }
}

struct Handler<I> {
    state: Option<i32>,
    _marker: PhantomData<I>,
}

impl<I> Handler<I> {
    fn new() -> Self {
        Self {
            state: None,
            _marker: PhantomData,
        }
    }
}

pub trait Project<E> {
    fn project(self) -> Option<E>;
}

impl<S> Project<S> for Option<S> {
    fn project(self) -> Option<S> {
        self
    }
}

impl<F, I> algoroutine::handler::Handler<I, F> for Handler<I>
where
    F: Coroutine<I, Yield = Effect<i32>>,
    F::Return: From<Option<i32>>,
    I: From<Option<i32>> + Project<i32>,
{
    fn handle(&mut self, mut pre: I, continuation: F) -> F::Return {
        let mut state: Option<CoroutineState<F::Yield, F::Return>> = None;
        let mut pinned = Box::pin(continuation);
        loop {
            match state {
                None => {
                    state = Some(pinned.as_mut().resume(I::from(None)));
                    continue;
                }
                Some(CoroutineState::Complete(n)) => break n,
                Some(CoroutineState::Yielded(Effect::Log(Log(msg)))) => match pre.project() {
                    Some(0) | None => {
                        println!("{msg}?");
                        let step = pinned.as_mut().resume(Some(0).into());
                        state = Some(step);
                        pre = Some(0).into();
                        continue;
                    }
                    Some(n) => break F::Return::from(Some(n)),
                },
                Some(CoroutineState::Yielded(Effect::State(s))) => match s {
                    State::Get => {
                        let step = pinned.as_mut().resume(self.state.clone().into());
                        state = Some(step);
                        continue;
                    }
                    State::Set(s) => {
                        self.state = s;
                        let step = pinned.as_mut().resume(None.into());
                        state = Some(step);
                        continue;
                    }
                },
            }
        }
    }
}
