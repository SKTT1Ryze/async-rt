//! A Simple Implementation of Rust `Async` Runtime
//! 
//! This crate is mainly for `Rust embedded` project  
//! 
//! Make sure initializing the `Heap Allocator` before use the `Executor`
#![no_std]


extern crate alloc;

use {
    lazy_static::*,
    alloc::{
        boxed::Box,
        vec::Vec,
        collections::vec_deque::VecDeque,
        sync::Arc,
    },
    core::{
        future::Future,
        pin::Pin,
        task::{Context, Poll},
        usize,
    },
    spin::Mutex,
    woke::{waker_ref, Woke},
};

/// Executor holds a queue of tasks that ready to make more progress
#[derive(Default)]
pub struct Executor {
    tasks: Mutex<VecDeque<Box<dyn IsTask + core::marker::Send + core::marker::Sync>>>
}

/// Reactor holds a queue of tasks that pending
#[derive(Default)]
pub struct Reactor {
    tasks: Mutex<Vec<Box<dyn IsTask + core::marker::Sync + core::marker::Send>>>
}

trait IsTask: HasID + TaskPoll {}

trait HasID {
    fn id(&self) -> usize;
}

trait TaskPoll {
    /// Every task poll here  
    /// Poll::Pending return false
    /// Poll::Ready(v) return true 
    fn task_poll(&self) -> bool;
}

/// Task is our unit of execution and holds a future are waiting on
struct Task<T> {
    pub future: Mutex<Pin<Box<dyn Future<Output = T> + Send + 'static>>>,
    pub id: usize,
}

impl<T> HasID for Arc<Task<T>> {
    fn id(&self) -> usize {
        self.id
    }
}

impl<T> TaskPoll for Arc<Task<T>> {
    fn task_poll(&self) -> bool {
        let mut future = self.future.lock();
        // create a waker for the task
        let waker = waker_ref(&self);
        // poll the future and give it a waker
        let context = &mut Context::from_waker(&*waker);
        matches!(future.as_mut().poll(context), Poll::Ready(_))
    }
}

impl<T> IsTask for Arc<Task<T>> {}

impl<T> Woke for Task<T> {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // move the task from Reactor queue to Executor queue
        while let Some(task) = DEFAULT_REACTOR.tasks.lock().pop() {
            if task.id() == arc_self.id() {
                DEFAULT_EXECUTOR.tasks.lock().push_back(task);
            } else {
                DEFAULT_REACTOR.tasks.lock().push(task);
            }
        }
    }
}

impl Executor {
    // Block on task
    fn block<T>(&mut self, future: Box<dyn Future<Output = T> + 'static + Send + Unpin>) -> T
    where
        T: Send + 'static
    {
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            id: unsafe { let id = COUNTER; COUNTER += 1; id}
        });
        let mut future = task.future.lock();
        // create a waker for the task
        let waker = waker_ref(&task);
        let context = &mut Context::from_waker(&*waker);
        loop {
            if let Poll::Ready(val) = future.as_mut().poll(context) {
                return val;
            }
        }
    }

    /// Add a future as Task to the queue of tasks for the first poll
    fn append_task<T>(
        &mut self,
        future: Box<dyn Future<Output = T> + 'static + Send + Unpin>
    ) -> Arc<Task<T>>
    where
        T: Send + 'static
    {
        // wrap the future as Task
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            id: unsafe { let id = COUNTER; COUNTER += 1; id}
        });
        self.tasks.lock().push_back(Box::new(task.clone()));
        task
    }

    // Poll all the tasks in executor queue
    fn poll_tasks(&mut self) {
        while let Some(task) = self.tasks.lock().pop_front() {
            if !task.task_poll() {
                // If task not ready, push it to Reactor queue
                DEFAULT_REACTOR.tasks.lock().push(task);
            }
        }
    }
}

impl Reactor {
    fn append_task<T>(
        &mut self,
        future: Box<dyn Future<Output = T> + 'static + Send + Unpin>
    ) -> Arc<Task<T>>
    where
        T: Send + 'static
    {
        // wrap the future as Task
        let task = Arc::new(Task {
            future: Mutex::new(Box::pin(future)),
            id: unsafe { let id = COUNTER; COUNTER += 1; id}
        });
        self.tasks.lock().push(Box::new(task.clone()));
        task
    }
}

lazy_static! {
    static ref DEFAULT_EXECUTOR: Box<Executor> = {
        let e = Executor::default();
        Box::new(e)
    };
    static ref DEFAULT_REACTOR: Box<Reactor> = {
        let r = Reactor::default();
        Box::new(r)
    };
}

static mut COUNTER: usize = 0;