#![feature(coroutines)]
#![feature(coroutine_trait)]
#![feature(stmt_expr_attributes)]

use std::{
    collections::VecDeque,
    marker::PhantomData,
    ops::{Coroutine, CoroutineState},
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicU8, Ordering},
    },
    time,
};

use algoroutine::{go, handler::Handler as _};

fn main() {
    let logic = #[coroutine]
    || {
        println!("begin...");

        for _ in 0..5 {
            println!("let's wait for 1 secs...");
            go!(Timeout::of(time::Duration::from_secs(1)) => Effect);
        }

        println!("next we try to wait 5 secs...");
        go!(Timeout::of(time::Duration::from_secs(5)));

        println!("end.");
    };

    let mut handler = Handler::new();
    let ans = handler.handle((), logic);
    dbg!(ans);
}

#[derive(Debug, Clone)]
enum State {
    Begin,
    Finished,
    Waiting,
    End,
}

impl From<u8> for State {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Begin,
            1 => Self::Waiting,
            2 => Self::Finished,
            3 => Self::End,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
struct Timeout {
    delay: time::Duration,
}

impl Timeout {
    pub fn of(delay: time::Duration) -> Self {
        Self { delay }
    }
}

enum Effect {
    Timer(Timeout),
}

impl From<Timeout> for Effect {
    fn from(value: Timeout) -> Self {
        Self::Timer(value)
    }
}

struct Task<I, E> {
    finished: Arc<AtomicU8>,
    continuation: Pin<Box<dyn Coroutine<I, Yield = E, Return = ()>>>,
}

impl<I, E> Task<I, E> {
    pub fn new(continuation: Pin<Box<dyn Coroutine<I, Yield = E, Return = ()>>>) -> Self {
        Self {
            finished: Arc::new(AtomicU8::new(0)),
            continuation,
        }
    }
}

struct Handler<I, E> {
    queue: VecDeque<Task<I, E>>,
    _data: PhantomData<I>,
}

impl<I, E> Handler<I, E> {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            _data: PhantomData,
        }
    }
}

impl<I> Handler<I, Effect> {
    fn handle_task<F>(&mut self, mut task: Task<I, Effect>)
    where
        F: Coroutine<I, Yield = Effect, Return = ()> + 'static,
        I: From<()>,
    {
        let state = task.continuation.as_mut().resume(I::from(()));
        let pinned = task.continuation;
        match state {
            CoroutineState::Complete(_) => {
                task.finished.store(3, Ordering::SeqCst);
                self.queue.push_back(Task {
                    finished: task.finished,
                    continuation: pinned,
                });
            }
            CoroutineState::Yielded(Effect::Timer(ref timeout)) => {
                let time_state = State::from(task.finished.load(Ordering::SeqCst));
                match time_state {
                    State::Begin => {
                        let delay = timeout.delay.clone();
                        task.finished.fetch_add(1, Ordering::SeqCst);
                        let signal = Arc::clone(&task.finished);
                        std::thread::spawn(move || {
                            std::thread::sleep(delay);
                            signal.fetch_add(1, Ordering::SeqCst);
                        });
                        self.queue.push_back(Task {
                            finished: task.finished,
                            continuation: pinned,
                        });
                    }
                    _ => {
                        self.queue.push_back(Task {
                            finished: task.finished,
                            continuation: pinned,
                        });
                    }
                }
            }
        }
    }
}

impl<I, F> algoroutine::handler::Handler<I, F> for Handler<I, Effect>
where
    F: Coroutine<I, Yield = Effect, Return = ()> + 'static,
    F::Return: From<()>,
    I: From<()>,
{
    fn handle(&mut self, _: I, continuation: F) -> F::Return {
        let task = Task {
            finished: Arc::new(AtomicU8::new(0)),
            continuation: Box::pin(continuation),
        };
        self.queue.push_back(task);
        while self.queue.len() > 0 {
            let task = self.queue.pop_front().unwrap();
            let state = State::from(task.finished.load(Ordering::SeqCst));
            match state {
                State::Begin => self.handle_task::<F>(task),
                State::Finished => self.handle_task::<F>(Task::new(task.continuation)),
                State::Waiting => self.queue.push_back(task),
                State::End => return,
            }
        }
    }
}
