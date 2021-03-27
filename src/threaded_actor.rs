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

use std::thread;
use crossbeam_channel::{
    self as channel,
    Receiver,
    Sender,
};

/// The type of function that can be sent to a `ThreadedActor<T>`.
type Task<T,R> = dyn FnOnce(&mut T) -> R + Send;

/// The boxed verion of the function for a `ThreadedActor<T>`.
type BoxedTask<T,R> = Box<Task<T,R>>;

/// The type of task that can be queued to the `ThreadedActor<T>`.
/// This erases any return value from the user's function. A call() to the
/// actor must wrap the user's function and send the return value back to
/// the caller through a channel.
type QueueTask<T> = BoxedTask<T,()>;

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
    T: Default + Send + 'static,
{
    /// Creates an actor with a default initial state.
    ///
    /// This requires the state type to implement the Default trait.
    pub fn new() -> Self {
        Self::from_state(T::default())
    }
}

impl<T> ThreadedActor<T>
where
    T: Send + 'static,
{
    /// Creates a threaded actor with the specified initial state.
    pub fn from_state(state: T) -> Self {
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

    /// Send an asynchronous request to the actor.
    ///
    /// This queues the request and returns immediately.
    pub fn cast<F>(&self, f: F)
    where
        F: FnOnce(&mut T) -> () + Send + 'static,
    {
        self.tx.send(Box::new(f)).unwrap();
    }

    pub fn call<F,R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R + Send + 'static,
        R: Send + 'static
    {
        let (tx, rx) = channel::unbounded();
        self.tx.send(Box::new(move |val: &mut T| {
            let res = f(val);
            tx.send(res).unwrap();
        })).unwrap();

        rx.recv().unwrap()
    }
}

// --------------------------------------------------------------------------

/*
fn main() {
    println!("Initializing...");

    println!();
    println!("size_of(ptr): {}", mem::size_of::<&u32>());
    println!("size_of(Task): {}", mem::size_of::<BoxedTask::<u32,()>>());
    println!();

    let actor = ThreadedActor::<u32>::new();

    actor.cast(|val| { *val += 1; });
    actor.cast(|val| { *val += 2; });

    let v = actor.call(|val| { *val });
    println!("Value: {}", v);

    println!("\nCleaning up...");
    drop(actor);

    println!("Done");
}
*/
