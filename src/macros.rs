use std::ops::Coroutine;

#[macro_export]
macro_rules! go {
    ($e:expr => $t:tt) => {
        yield $t::from($e)
    };
    ($e:expr) => {
        yield $e.into()
    };
    ($f:expr, $arg:expr) => {{
        let mut pinned = Box::pin($f);
        let mut injs = $arg;
        loop {
            let res = pinned.as_mut().resume(injs);
            match res {
                CoroutineState::Yielded(eff) => {
                    injs = yield eff.into();
                }
                CoroutineState::Complete(v) => break v,
            }
        }
    }};
    ($eff:tt, $f:expr, $arg:expr) => {{
        let mut pinned = Box::pin($f);
        let mut injs = $arg;
        loop {
            let res = pinned.as_mut().resume(injs);
            match res {
                CoroutineState::Yielded(eff) => {
                    injs = yield $eff::from(eff);
                }
                CoroutineState::Complete(v) => break v,
            }
        }
    }};
}

#[inline]
pub fn assert_effect<Eff, R, O>(
    f: impl Coroutine<R, Yield = Eff, Return = O>,
) -> impl Coroutine<R, Yield = Eff, Return = O> {
    f
}

#[macro_export]
macro_rules! effectful {
    (($arg:tt => $r:ty) -> $o:ty | $t:ty, $b:block) => {
        assert_effect::<$t, $r, $o>(
            #[coroutine]
            move |$arg: $r| $b,
        )
    };

    ($t:ty, $e:expr) => {
        assert_effect::<$t, _, _>(
            #[coroutine]
            $e,
        )
    };
}
