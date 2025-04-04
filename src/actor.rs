// cooper/src/actor.rs
//
// Copyright (c) 2021, Frank Pagliughi <fpagliughi@mindspring.com>
// All Rights Reserved
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//
//! cooper

use async_channel::{self as channel, Receiver, Sender};
use futures::future::BoxFuture;
use std::fmt::Debug;
use std::future::Future;

/// The actor function signature
pub type BoxedActorFn<S> = Box<dyn for<'a> FnOnce(&'a mut S) -> BoxFuture<'a, ()> + Send>;

/// Message type for the Actor.
///
/// This wraps an async function type that takes a mutable reference to a
/// state object. Implementations of actor objects can queue functions and
/// closures to process the state.
/// `S` is the internal state type for the actor to manage
struct Message<S> {
    func: BoxedActorFn<S>,
}

/// The Actor.
///
/// This is an async command processor that serializes requests around an
/// internal state. Each request runs to completion, atomically, in the
/// order received, and thus tasks do not need to lock or protect the state
/// for access.
#[derive(Clone)]
pub struct Actor<S>
where
    S: Send + 'static,
{
    /// The channel to send requests to the actor's processor task.
    tx: Sender<Message<S>>,
}

#[cfg(not(feature = "tokio"))]
fn spawn<F>(future: F)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    smol::spawn(future).detach();
}

#[cfg(feature = "tokio")]
fn spawn<F>(future: F)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    tokio::spawn(future);
}

impl<S> Actor<S>
where
    S: Send + 'static,
{
    /// Creates a new actor from an initial state.
    pub fn new(state: S) -> Self {
        let (tx, rx) = channel::unbounded();

        // TODO: Stash the handle somewhere?
        //  Perhaps make a registry of running actors?
        spawn(async move { Self::run(state, rx).await });

        Self { tx }
    }

    /// The actor's command processor.
    ///
    /// This runs each request for the actor to completion before
    /// running the next one.
    async fn run(mut state: S, rx: Receiver<Message<S>>) {
        while let Ok(msg) = rx.recv().await {
            (msg.func)(&mut state).await;
        }
    }

    /// This is a totally asynchronous operation. Awaiting the returned
    /// future only waits for the operation to be placed in the queue.
    /// It does not wait for the operation to be executed.
    pub fn cast<F>(&self, f: F)
    where
        F: for<'a> FnOnce(&'a mut S) -> BoxFuture<'a, ()>,
        F: 'static + Send,
    {
        let msg = Message {
            func: Box::new(move |state| {
                Box::pin(async move {
                    f(state).await;
                })
            }),
        };

        // TODO: Should we at least log the error?
        let _ = self.tx.try_send(msg);
    }

    /// A call is a synchronous operation within the async task.
    /// It will queue the request, wait for it to execute, and
    /// return the result.
    pub async fn call<F, R>(&self, f: F) -> R
    where
        F: for<'a> FnOnce(Sender<R>, &'a mut S) -> BoxFuture<'a, Option<R>>,
        F: 'static + Send,
        R: 'static + Send + Debug,
    {
        let (tx, rx) = channel::bounded(1);
        let msg = Message {
            func: Box::new(move |state| {
                Box::pin(async move {
                    if let Some(res) = f(tx.clone(), state).await {
                        let _ = tx.send(res).await;
                    }
                })
            }),
        };

        let _ = self.tx.send(msg).await;
        // TODO: Return an error instead of panicking
        rx.recv().await.expect("Actor is gone")
    }

    /// Blocks the calling task until all requests up to this point have
    /// been processed.
    ///
    /// Note that if there are clones of the actor, additional requests
    /// may get queued after this one, so the queue is not guaranteed to be
    /// empty when this returns; just that all the requests prior to this one
    /// have completed.
    pub async fn flush(&self) {
        self.call(|_, _| Box::pin(async move { Some(()) })).await
    }
}

impl<S> Default for Actor<S>
where
    S: Default + Send + 'static,
{
    /// Creates a new actor with a default state.
    fn default() -> Self {
        Self::new(S::default())
    }
}
