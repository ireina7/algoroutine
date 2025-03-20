#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]

use std::{
    marker::PhantomData,
    ops::{Coroutine, CoroutineState},
};

use algoroutine::{go, handler::Handler};

fn main() {
    let div = #[coroutine]
    |ctx: Context| {
        let (a, b) = ctx.unwrap();
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

    let mut handler = ExceptionHandler::new();
    let ans = handler.handle(Context::None, logic);
    dbg!(ans);
}

#[derive(Debug)]
pub enum Exception {
    Raise(String),
}

struct ExceptionHandler<I> {
    _data: PhantomData<I>,
}

impl<I> ExceptionHandler<I> {
    fn new() -> Self {
        Self { _data: PhantomData }
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

impl From<(i32, i32)> for Context {
    fn from(value: (i32, i32)) -> Self {
        Context::Some(value)
    }
}

impl<I, F> algoroutine::handler::Handler<I, F> for ExceptionHandler<I>
where
    F: Coroutine<I, Yield = Exception>,
    F::Return: From<i32>,
    I: From<Option<(i32, i32)>>,
{
    fn handle(&mut self, _: I, continuation: F) -> F::Return {
        let mut state: Option<CoroutineState<F::Yield, F::Return>> = None;
        let mut pinned = Box::pin(continuation);
        loop {
            match state {
                None => {
                    state = Some(pinned.as_mut().resume(I::from(None)));
                    continue;
                }
                Some(CoroutineState::Complete(n)) => break n,
                Some(CoroutineState::Yielded(Exception::Raise(msg))) => {
                    eprintln!("Exception: {}", msg);
                    break F::Return::from(-1); // early stop
                }
            }
        }
    }
}
