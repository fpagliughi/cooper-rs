// cooper/src/threaded_actor.rs
//
// This file is part of the `cooper-rs` library.
//
// Copyright (c) 2021, Frank Pagliughi <fpagliughi@mindspring.com>
// All Rights Reserved
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.

use crossbeam_channel::{self as channel, Receiver, Sender};
use std::thread;

/// The type of function that can be sent to a `ThreadedActor<T>`.
type Task<T, R> = dyn FnOnce(&mut T) -> R + Send;

/// The boxed verion of the function for a `ThreadedActor<T>`.
type BoxedTask<T, R> = Box<Task<T, R>>;

/// The type of task that can be queued to the `ThreadedActor<T>`.
///
/// This erases any return value from the user's function. A call() to the
/// actor must wrap the user's function and send the return value back to
/// the caller through a channel.
type QueueTask<T> = BoxedTask<T, ()>;

// --------------------------------------------------------------------------

/// An actor that uses an OS thread-per-instance.
///
/// This may be useful if the application only needs a few actors and doesn'
/// otherwise use an async runtime. Or it can be used in an async context if
/// an actor needs to block or is compute intensive and requires its own thread.
#[derive(Clone)]
pub struct ThreadedActor<T> {
    /// A transmit channel to send requests to the actor thread.
    tx: Sender<QueueTask<T>>,
}

impl<T> ThreadedActor<T>
where
    T: Send + 'static,
{
    /// Creates a threaded actor with the specified initial state.
    pub fn new(state: T) -> Self {
        let (tx, rx) = channel::unbounded();

        thread::spawn(move || {
            Self::thr_func(state, rx);
        });

        Self { tx }
    }

    /// The thread function for the actor.
    ///
    /// This runs in its own OS thread.
    fn thr_func(mut val: T, rx: Receiver<QueueTask<T>>) {
        for f in rx {
            f(&mut val);
        }
    }

    /// Sends an asynchronous request to the actor.
    ///
    /// This queues the request and returns immediately.
    pub fn cast<F>(&self, f: F)
    where
        F: FnOnce(&mut T) -> () + Send + 'static,
    {
        self.tx.send(Box::new(f)).unwrap();
    }

    /// Sends a synchronous request to the actor.
    ///
    /// This queues the request to the actor thread, then blocks waiting for
    /// a response.
    pub fn call<F, R>(&self, f: F) -> R
    where
        F: FnOnce(Sender<R>, &mut T) -> Option<R> + Send + 'static,
        R: Send + 'static,
    {
        let (tx, rx) = channel::unbounded();
        self.tx
            .send(Box::new(move |val: &mut T| {
                if let Some(res) = f(tx.clone(), val) {
                    tx.send(res).unwrap();
                }
            }))
            .unwrap();

        rx.recv().unwrap()
    }

    /// Blocks the calling task until all requests up to this point have
    /// been processed.
    ///
    /// Note that if there are clones of the actor, additional requests
    /// may get queued after this one, so the queue is not guaranteed to be
    /// empty when this returns; just that all the requests prior to this one
    /// have completed.
    pub fn flush(&self) {
        self.call(move |_, _| Some(()));
    }
}

impl<T> Default for ThreadedActor<T>
where
    T: Default + Send + 'static,
{
    /// Creates an actor with a default initial state.
    ///
    /// This requires the state type to implement the Default trait.
    fn default() -> Self {
        Self::new(T::default())
    }
}
