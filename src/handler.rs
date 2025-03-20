use std::ops::Coroutine;

pub trait Handler<R, F: Coroutine<R>> {
    fn handle(&mut self, pre: R, continuation: F) -> F::Return;
}
