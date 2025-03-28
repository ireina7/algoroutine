# algoroutine
Light-weight *algebraic effect* (algebraic goroutine) in Rust.

Using `go!` macro just as Haskell's do notation while we use algebraic effects
instead of monads to avoid monad's composition issue.
Only *one-shot (linear) algebraic effect* based on coroutine is supported.
Therefore we need nightly compiler to transform code.

## Main macros and combinators
- `go!` macro to run effects and coroutines, just like `?` and `.await`.
- `Coroutine.map` to map results.
- `Coroutine.and_then` to chain operations.

### Handler
Currently handlers can only be built by hand. There's no easy way to combine.

## Example
### Logging and mutable states
```rust
let prepare = #[coroutine] |_: Option<i32>| {
    go!(Log("preparing".into()) => Effect);
    return ResultCode(0);
};

let logic = #[coroutine] |_: Option<i32>| {
    go!(Log("start".into()) => Effect); // inject Log into Effect type

    go!(State::Set(Some(9)));
    let s = go!(State::Get);
    go!(Log(format!("Got {:?}", s)));

    let res = go!(prepare, None);
    if !res.is_ok() {
        go!(Log(format!("error code: {}", res.0)));
    }
    return res;
};

// Now we can use different ways to interpret `Log` and `State` effects!
let mut handler: Handler<Option<i32>> = Handler::new();
let ans = handler.handle(logic, None);
dbg!(ans);
```
To inject effects, we don't use `coproduct`, just `From`!
See [examples/logging_and_states](./examples/logging_and_state.rs) for details.

### Exception
```rust
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
    let ans = go!(div, (4, 0) => Exception); // will stop executing rest continuation
    println!("div result: {}", ans);
    println!("end.");
    return 0;
};

let mut handler = ExceptionHandler::new();
let ans = handler.handle(logic, Context::None);
dbg!(ans);
```
See [examples/exception](./examples/exception.rs) for details.

### Event loop
```rust
let logic = #[coroutine]
|| {
    println!("begin...");

    println!("first let's wait for 3 secs...");
    go!(Timeout::of(time::Duration::from_secs(3)) => Effect);

    println!("next we try to wait 5 secs...");
    go!(Timeout::of(time::Duration::from_secs(5)));

    println!("end.");
};
```
See [examples/event_loop](./examples/event_loop.rs) for details.


## Fun facts
- `goroutine` and `go` syntax is cool (but golang's type system is terrible)
- `algebraic effect` is cool
- `algebraic goroutine (algoroutine)` is probably bad ;)

## Limitation
- Current API only permits `'static` coroutines. 
    Once cannot borrow data outside coroutine. If you have to, consider using `move` and more data structures.

## TODO
- [ ] Macros for declare effect types
- [ ] More auto-injective traits
- [ ] Remove boxes for better performance