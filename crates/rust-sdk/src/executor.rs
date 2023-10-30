use std::{
    cell::RefCell,
    future::Future,
    pin::Pin,
    task::{Context, Waker},
};
use once_cell::sync::OnceCell;

thread_local! {
    pub static EXECUTOR: RefCell<Executor> = RefCell::new(Executor::default());
}

#[derive(Default)]
pub struct Executor {
    task: OnceCell<Pin<Box<dyn Future<Output = ()>>>>,
}
impl Executor {
    pub fn run(&mut self) {
        if let Some(ref mut task) = self.task.get_mut() {
            let waker = Waker::noop();
            let mut cx = Context::from_waker(&waker);
            // Task is done, but we can't unset a OnceCell
            // Do any cleanup or finalization here if needed
            let _ = task.as_mut().poll(&mut cx);
        }
    }
}

pub fn spawn_local<F: Future<Output = ()> + 'static>(future: F) {
    EXECUTOR.with(|executor| {
        let executor = &mut executor.borrow_mut();
        let _ = executor.task.set(Box::pin(future));
        executor.run();
    });
}

// TODO: macro to wrap start/main function
