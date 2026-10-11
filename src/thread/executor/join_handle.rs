use std::{
    error::Error,
    fmt::Display,
    io,
    marker::PhantomData,
    mem::ManuallyDrop,
    panic::resume_unwind,
    pin::Pin,
    ptr,
    task::{Context, Poll},
};

use super::{Panic, task::Task};

#[must_use = "Drop `JoinHandle` will cancel the task. Use `detach` to run it in background."]
#[repr(transparent)]
pub struct JoinHandle<T> {
    task: Option<Task>,
    _marker: PhantomData<T>,
}

unsafe impl<T: Send> Send for JoinHandle<T> {}

unsafe impl<T> Sync for JoinHandle<T> {}

impl<T> Unpin for JoinHandle<T> {}

impl<T> JoinHandle<T> {
    pub(crate) fn new(task: Task) -> Self {
        Self {
            task: Some(task),
            _marker: PhantomData,
        }
    }

    pub async fn cancel(self) -> Option<T> {
        self.task.as_ref()?.cancel(false);
        self.await.ok()
    }

    pub fn detach(self) {
        unsafe { ptr::drop_in_place(&raw mut ManuallyDrop::new(self).task) };
    }

    pub fn is_finished(&self) -> bool {
        match &self.task {
            Some(task) => task.is_finished(),
            None => true,
        }
    }
}

#[derive(Debug)]
pub enum JoinError {
    /// The task was cancelled.
    Cancelled,
    /// The task panicked.
    Panicked(Panic),
}

pub trait ResumeUnwind {
    type Output;

    fn resume_unwind(self) -> Self::Output;
}

impl<T> ResumeUnwind for Result<T, JoinError> {
    type Output = Option<T>;

    fn resume_unwind(self) -> Self::Output {
        match self {
            Ok(res) => Some(res),
            Err(JoinError::Cancelled) => None,
            Err(JoinError::Panicked(e)) => resume_unwind(e),
        }
    }
}

impl JoinError {
    pub fn resume_unwind(self) {
        if let JoinError::Panicked(e) = self {
            resume_unwind(e)
        }
    }
}

impl Display for JoinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JoinError::Cancelled => write!(f, "Task was cancelled"),
            JoinError::Panicked(_) => write!(f, "Task has panicked"),
        }
    }
}

impl Error for JoinError {}

impl From<JoinError> for io::Error {
    fn from(e: JoinError) -> Self {
        match e {
            JoinError::Cancelled => io::Error::other("Task was cancelled"),
            JoinError::Panicked(_) => io::Error::other("Task has panicked"),
        }
    }
}

impl<T> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let task = self.task.as_ref().expect("Cannot poll after completion");

        unsafe { task.poll(cx) }.map(|res| {
            self.task = None;

            match res {
                Some(Ok(res)) => Ok(res),
                Some(Err(e)) => Err(JoinError::Panicked(e)),
                None => Err(JoinError::Cancelled),
            }
        })
    }
}

impl<T> Drop for JoinHandle<T> {
    fn drop(&mut self) {
        if let Some(task) = self.task.as_ref() {
            task.cancel(true);
        }
    }
}
