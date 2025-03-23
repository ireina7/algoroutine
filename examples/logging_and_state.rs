#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]

use std::{
    marker::PhantomData,
    ops::{Coroutine, CoroutineState},
    pin::Pin,
};

use algoroutine::{
    go,
    handler::{Consumer, OneStep, SyncConsumer},
};

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

    let handler: Handler<Option<i32>> = Handler::new();
    let mut consumer = SyncConsumer::from(handler);
    let ans = consumer.consume(logic, None);
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

impl<I> algoroutine::handler::Step<Effect<i32>, I, ResultCode> for Handler<I>
where
    I: From<Option<i32>> + Project<i32>,
{
    fn step<F>(&mut self, continuation: Pin<&mut F>, pre: I) -> OneStep<I, ResultCode>
    where
        F: Coroutine<I, Yield = Effect<i32>, Return = ResultCode>,
    {
        let mut pinned = Box::pin(continuation);
        let state = pinned.as_mut().resume(pre);
        match state {
            CoroutineState::Complete(n) => OneStep::Return(n),
            CoroutineState::Yielded(Effect::Log(Log(msg))) => {
                println!("{msg}?");
                OneStep::Yield(I::from(Some(0)))
            }
            CoroutineState::Yielded(Effect::State(s)) => match s {
                State::Get => OneStep::Yield(I::from(self.state.clone())),
                State::Set(s) => {
                    self.state = s;
                    OneStep::Yield(I::from(None))
                }
            },
        }
    }
}
