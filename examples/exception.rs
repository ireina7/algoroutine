#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]

use std::{
    ops::{Coroutine, CoroutineState},
    pin::Pin,
};

use algoroutine::{
    go,
    handler::{Consumer, OneStep, SyncConsumer},
};

fn main() {
    let div = #[coroutine]
    |(a, b): (i32, i32)| {
        if b == 0 {
            go!(Exception::Raise("divided by 0".into()) => Exception);
        }
        return a / b;
    };

    let logic = #[coroutine]
    |_: Context| {
        println!("Start!");
        let ans = go!(div, (4, 0) => Exception);
        println!("div result: {}", ans);
        println!("end.");
        return 0;
    };

    let mut handler = SyncConsumer::from(ExceptionHandler::new());
    let ans = handler.consume(logic, Context::None);
    dbg!(ans);
}

#[derive(Debug)]
pub enum Exception {
    Raise(String),
}

struct ExceptionHandler {}

impl ExceptionHandler {
    fn new() -> Self {
        Self {}
    }
}

enum Context {
    None,
    Some((i32, i32)),
}

impl Context {
    fn unwrap(self) -> (i32, i32) {
        match self {
            Context::None => unreachable!(),
            Context::Some(v) => v,
        }
    }
}

impl From<Option<(i32, i32)>> for Context {
    fn from(value: Option<(i32, i32)>) -> Self {
        match value {
            Some(v) => Context::Some(v),
            None => Context::None,
        }
    }
}

impl From<Context> for (i32, i32) {
    fn from(value: Context) -> Self {
        value.unwrap()
    }
}

impl<R> algoroutine::handler::Step<Exception, R, i32> for ExceptionHandler
where
    R: From<Option<(i32, i32)>>,
{
    fn step<F>(&mut self, continuation: Pin<&mut F>, _: R) -> OneStep<R, i32>
    where
        F: Coroutine<R, Yield = Exception, Return = i32> + 'static,
    {
        let mut pinned = Box::pin(continuation);
        let state = pinned.as_mut().resume(R::from(None));
        match state {
            CoroutineState::Complete(n) => OneStep::Return(n),
            CoroutineState::Yielded(Exception::Raise(msg)) => {
                eprintln!("Exception: {}", msg);
                OneStep::Return(-1) // early stop
            }
        }
    }
}
