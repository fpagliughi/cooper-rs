# cooper

A simple in-process Actor library in Rust.

This is a minimal library to create Actors that run in an async execution context within a single process. It is not (nor will likely ever be) a way to communicate between different applications or different network hosts.

## Development Status

This is a very early proof-of-concept, and is currently not production-ready.

The initial implementation uses the `smol` library for channels and an executor, but by release time it will either be agnostic to the runtime or allow you to chose between `tokio`, `async-std`, or `smol`.

## Huge Thanks

This project sat idle on GitHub for three years while I waited for async/await to stabilize, and then tried to make heads or tails of it. After a few overly complicated failed attempts ...like writing an entire serial executor... it struck me that I just wanted a trivial async loop to execute closures sent to it. If only I could figure out how to queue the closures...

A huge "thank you" goes out to @Darksonn for showing me how to send async closures across a channel.

https://users.rust-lang.org/t/sending-futures-through-a-channel/57229

Check out her blog on the topic of simple Actors, which is very similar in spirit to what is contained in the core of this library.

[Actors with Tokio](https://ryhl.io/blog/actors-with-tokio/)

## The Basics

An Actor is an object containing an internal state and a private execucution context that can be used to update that state. The application and other actors communicate with it by sending it messages, which the actor processes in order, sequentially. Since the state is only available to the internal execution context, there are no data races or contention, and no need for locks.

This library borrows some ideas and nomenclature from the Erlang and Elixir languages:

 - A `cast` is an asynchronous message queued to the actor with no return value. It is pure fire-and-forget. The cast merely places the request into the Actor's queue, and does not wait for it to execute.

- A `call` is a synchronous operation that waits for the request to execute and return a result. The caller places a request into the actor's queue, then blocks the current task until the request is executed and returns a value.

Note that the nomenclature is a little confusing here in regard to "synchronous" and "blocking" calls. The entire library runs in a Rust `async` environment. But awaiting a `cast()` operation simply waits for the request to be queued, whereas awaiting a `call()` operation waits for the request to execute in the actor's task and get a result back.

The library's `Actor<S>;` is a struct that is generic around an instance the internal state type `S`. The `Actor` instance takes an initial state value, or uses `S::default()`, and sends it to a spawned task to await requests. The actual `Actor<S>` is just a wrapper around a channel `Sender` that can be used to queue requests to the internal task using the `cast()` and `call()` functions. The Actor can be cloned as needed and sent to other tasks and threads. When the last one goes out of scope, the internal task exits, dropping the state object.

Concrete actor types can be easily built around `Actor<>` by supplying a state type, value, and the functions/closures to manipulate it.

As an example, this is a key/value map that can be shared between different tasks:

```
/// The internal state type for the Actor
type State = HashMap<String, String>;

/// An actor that can act as a shared key/value store of strings.
#[derive(Clone)]
pub struct SharedMap {
    actor: Actor<State>,
}

impl SharedMap {
    /// Create a new actor to share a key/value map of string.
    pub fn new() -> Self {
        Self { actor: Actor::new() }
    }

    /// Insert a value into the shared map.
    pub async fn insert(&self, key: String, val: String) {
        self.actor.cast(|state: &mut State| Box::pin(async move {
            state.insert(key, val);
        })).await
    }


    /// Gets the value, if any, from the shared map that is
    /// associated with the key.
    pub async fn get(&self, key: String) -> Option<String> {
        self.actor.call(|state: &mut State| Box::pin(async move {
            state.get(&key).map(|v| v.to_string())
        })).await
    }
}
```

The shared map can then be used in an async block, like:

```

    async {
        let map = SharedMap::new();

        map.insert("city", "Boston").await;

        assert_eq!(
            map.get("city").await, 
            Some("Boston".to_string())
        );
    });
```




















