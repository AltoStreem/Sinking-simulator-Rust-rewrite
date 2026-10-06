//! ExecutorKt.java: immediate GLFW wrappers and shared single-thread GL dispatcher.
#![allow(dead_code)]
use std::{
    any::Any,
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Condvar, Mutex, OnceLock, mpsc},
    time::{Duration, Instant},
};
type Job = Box<dyn FnOnce() + Send + 'static>;
struct Task {
    due: Instant,
    sequence: u64,
    job: Job,
}
#[derive(Default)]
struct Queue {
    tasks: Vec<Task>,
    closed: bool,
    started: bool,
    sequence: u64,
}
struct State {
    queue: Mutex<Queue>,
    wake: Condvar,
}
pub(crate) struct SingleThreadDispatcher {
    name: String,
    state: Arc<State>,
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct RejectedExecution;
pub(crate) struct TaskResult<T>(mpsc::Receiver<Result<T, Box<dyn Any + Send>>>);
impl<T> TaskResult<T> {
    pub fn join(self) -> T {
        match self.0.recv().expect("dispatcher task result disconnected") {
            Ok(value) => value,
            Err(error) => std::panic::resume_unwind(error),
        }
    }
}
impl SingleThreadDispatcher {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: Arc::new(State {
                queue: Mutex::new(Queue::default()),
                wake: Condvar::new(),
            }),
        }
    }
    pub fn execute<T: Send + 'static>(
        &self,
        job: impl FnOnce() -> T + Send + 'static,
    ) -> Result<TaskResult<T>, RejectedExecution> {
        self.schedule(Duration::ZERO, job)
    }
    pub fn schedule<T: Send + 'static>(
        &self,
        delay: Duration,
        job: impl FnOnce() -> T + Send + 'static,
    ) -> Result<TaskResult<T>, RejectedExecution> {
        let (send, receive) = mpsc::channel();
        let mut queue = self.state.queue.lock().unwrap();
        if queue.closed {
            return Err(RejectedExecution);
        }
        let sequence = queue.sequence;
        queue.sequence = queue.sequence.wrapping_add(1);
        queue.tasks.push(Task {
            due: Instant::now() + delay,
            sequence,
            job: Box::new(move || {
                let result = catch_unwind(AssertUnwindSafe(job));
                let _ = send.send(result);
            }),
        });
        if !queue.started {
            queue.started = true;
            let state = self.state.clone();
            std::thread::Builder::new()
                .name(self.name.clone())
                .spawn(move || run(state))
                .expect("unable to create dispatcher worker");
        }
        self.state.wake.notify_one();
        Ok(TaskResult(receive))
    }
    /// ScheduledThreadPoolExecutor.shutdown retains already accepted delayed jobs.
    pub fn close(&self) {
        self.state.queue.lock().unwrap().closed = true;
        self.state.wake.notify_one();
    }
    pub fn description(&self) -> String {
        format!("ThreadPoolDispatcher[1, {}]", self.name)
    }
}
fn run(state: Arc<State>) {
    loop {
        let mut queue = state.queue.lock().unwrap();
        let job = loop {
            if queue.tasks.is_empty() {
                if queue.closed {
                    return;
                }
                queue = state.wake.wait(queue).unwrap();
                continue;
            }
            let index = queue
                .tasks
                .iter()
                .enumerate()
                .min_by_key(|(_, task)| (task.due, task.sequence))
                .unwrap()
                .0;
            let due = queue.tasks[index].due;
            if due > Instant::now() {
                let wait = due.saturating_duration_since(Instant::now());
                queue = state.wake.wait_timeout(queue, wait).unwrap().0;
                continue;
            }
            break queue.tasks.remove(index).job;
        };
        drop(queue);
        job();
    }
}
static GL_CONTEXT: OnceLock<Arc<SingleThreadDispatcher>> = OnceLock::new();
pub(crate) fn get_gl_context() -> Arc<SingleThreadDispatcher> {
    GL_CONTEXT
        .get_or_init(|| Arc::new(SingleThreadDispatcher::new("GL")))
        .clone()
}
pub(crate) fn glfw_safe<T>(job: impl FnOnce() -> T) -> T {
    let _ = get_gl_context();
    job()
}
pub(crate) fn block_glfw_safe<T>(job: impl FnOnce() -> T) -> T {
    let _ = get_gl_context();
    job()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wrappers_run_immediately_on_caller_with_borrowed_values_and_panics() {
        let caller = std::thread::current().id();
        let mut value = 0;
        assert_eq!(
            glfw_safe(|| {
                value = 1;
                std::thread::current().id()
            }),
            caller
        );
        assert_eq!(
            block_glfw_safe(|| {
                value += 1;
                value
            }),
            2
        );
        assert!(std::panic::catch_unwind(|| glfw_safe(|| panic!("source callback"))).is_err());
    }
    #[test]
    fn singleton_dispatcher_preserves_name_single_thread_and_task_order() {
        let dispatcher = get_gl_context();
        assert!(Arc::ptr_eq(&dispatcher, &get_gl_context()));
        assert_eq!(dispatcher.description(), "ThreadPoolDispatcher[1, GL]");
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut results = Vec::new();
        for index in 0..8 {
            let log = log.clone();
            results.push(
                dispatcher
                    .execute(move || {
                        log.lock().unwrap().push(index);
                        (
                            std::thread::current().id(),
                            std::thread::current().name().unwrap().to_owned(),
                        )
                    })
                    .unwrap(),
            );
        }
        let threads: Vec<_> = results.into_iter().map(TaskResult::join).collect();
        assert!(threads.iter().all(|thread| thread == &threads[0]));
        assert_eq!(threads[0].1, "GL");
        assert_ne!(threads[0].0, std::thread::current().id());
        assert_eq!(*log.lock().unwrap(), (0..8).collect::<Vec<_>>());
    }
    #[test]
    fn delayed_jobs_survive_shutdown_and_task_panic_does_not_kill_worker() {
        let dispatcher = SingleThreadDispatcher::new("test-GL");
        let failed = dispatcher.execute(|| panic!("task")).unwrap();
        assert!(std::panic::catch_unwind(AssertUnwindSafe(|| failed.join())).is_err());
        let delayed = dispatcher
            .schedule(Duration::from_millis(10), || 42)
            .unwrap();
        let immediate = dispatcher.execute(|| 7).unwrap();
        dispatcher.close();
        assert!(dispatcher.execute(|| 0).is_err());
        assert_eq!(immediate.join(), 7);
        assert_eq!(delayed.join(), 42);
    }
}
