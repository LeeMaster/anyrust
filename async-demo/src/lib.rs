use std::{
    collections::HashMap,
    future::Future,
    mem,
    pin::Pin,
    sync::{
        mpsc::{channel, Sender},
        Arc, Condvar, Mutex,
    },
    task::{Context, Poll, RawWaker, RawWakerVTable},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

// Parker
// this parker is really sample used in the thread environment
// and use the condvar
#[derive(Default)]
struct Parker(Mutex<bool>, Condvar);

impl Parker {
    fn park(&self) {
        let mut resumable = self.0.lock().unwrap();
        while !*resumable {
            resumable = self.1.wait(resumable).unwrap();
        }
        *resumable = false;
    }

    fn unpark(&self) {
        *self.0.lock().unwrap() = true;
        self.1.notify_one();
    }
}

#[derive(Default)]
struct Waker {
    parker: Arc<Parker>,
}

const VTABLE: RawWakerVTable = unsafe {
    RawWakerVTable::new(
        |s| custom_clone(&*(s as *const Waker)),
        |s| custom_wake(&*(s as *const Waker)),
        |s| (*(s as *const Waker)).parker.unpark(),
        |s| drop(Arc::from_raw(s as *const Waker)),
    )
};

fn custom_wake(s: &Waker) {
    let arc = unsafe { Arc::from_raw(s) };
    arc.parker.unpark();
}

fn custom_clone(s: &Waker) -> RawWaker {
    let arc = unsafe { Arc::from_raw(s) };
    // TODO: why this point we need forget the arc pointer ?
    std::mem::forget(arc.clone());
    RawWaker::new(Arc::into_raw(arc) as *const (), &VTABLE)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
