use std::cell::RefCell;
use std::future::Future;
use std::task::{Poll, Context};
use std::pin::Pin;
use futures::{
    channel::oneshot,
    task::noop_waker,
};

thread_local! {
    pub static EXECUTOR: RefCell<Executor> = RefCell::new(Executor::new());
}
pub struct Executor {
    tasks: Vec<Pin<Box<dyn Future<Output = ()>>>>,
}

impl Executor {
    pub fn new() -> Self {
        Self { tasks: vec![] }
    }

    pub fn spawn<F: Future<Output = ()> + 'static>(&mut self, future: F) {
        self.tasks.push(Box::pin(future));
        self.run();
    }

    pub fn spawn_with_result<F: Future<Output = T> + 'static, T: 'static>(&mut self, future: F) -> oneshot::Receiver<T> {
        let (tx, rx) = oneshot::channel();

        let wrapped_future = async {
            let result = future.await;
            let _ = tx.send(result);
        };

        self.spawn(wrapped_future);
        rx
    }

    pub fn run(&mut self) {
        let waker = noop_waker();
        let mut cx = Context::from_waker(&waker);

        let mut i = 0;
        while i < self.tasks.len() {
            if let Poll::Ready(()) = self.tasks[i].as_mut().poll(&mut cx) {
                // remove the task if it's done
                self.tasks.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }
}

pub fn spawn_local<F: Future<Output = ()> + 'static>(future: F) {
    EXECUTOR.with(|executor| executor.borrow_mut().spawn(future));
}

pub fn spawn_local_with_result<F: Future<Output = T> + 'static, T: 'static>(future: F) -> oneshot::Receiver<T> {
    EXECUTOR.with(|executor| executor.borrow_mut().spawn_with_result(future))
}
