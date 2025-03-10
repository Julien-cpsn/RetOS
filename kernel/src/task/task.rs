use alloc::boxed::Box;
use alloc::string::String;
use alloc::sync::Arc;
use alloc::task::Wake;
use core::fmt::{Debug, Formatter};
use core::sync::atomic::{AtomicU64, Ordering};
use core::task::{Context, Poll, Waker};
use core::{future::Future, pin::Pin};
use crossbeam_queue::ArrayQueue;
use spin::Mutex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct TaskId(pub u64);

pub struct Task {
    pub id: TaskId,
    pub name: String,
    future: Mutex<Pin<Box<dyn Future<Output = ()> + Send + Sync>>>,
}

pub struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Task {
    pub fn new(name: String, future: impl Future<Output = ()> + Send + Sync + 'static) -> Task {
        Task {
            id: TaskId::new(),
            name,
            future: Mutex::new(Box::pin(future)),
        }
    }

    pub fn poll(&self, context: &mut Context) -> Poll<()> {
        self.future.lock().as_mut().poll(context)
    }
}

impl Debug for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Task")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}

impl TaskWaker {
    pub fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

