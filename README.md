# goroutine
Light-weight algebraic goroutine in rust

Using `go!` macro just as Haskell's do notation while we use algebraic effects
instead of monads.

Only zero-shot algebraic effect based on coroutine is supported.
Therefore we need nightly compiler to transform code.

## Main macros and combinators
- `go!` macro to run effects and coroutines, just like `?` and `.await`.
- `Coroutine.map` to map results.
- `Coroutine.and_then` to chain operations.

## Example
```rust
let logic = #[coroutine] |_: Option<i32>| {
    go!(Log("start".into()) => Effect); // inject Log into Effect type

    go!(State::Set(Some(9)));
    let s = go!(State::Get);
    go!(Log(format!("Got {:?}", s)));

    let res = go!(log0, None);
    if !res.is_ok() {
        go!(Log(format!("error code: {}", res.0)));
    }
    return res;
};

// Now we can use different ways to interpret `Log` and `State` effects!
let mut handler: Handler<Option<i32>> = Handler::new();
let ans = handler.handle(None, logic);
dbg!(ans);
```