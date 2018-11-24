// cooper/src/lib/rs
//
// Copyright (c) 2018, Frank Pagliughi
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//
//! cooper

extern crate futures;
extern crate futures_cpupool;

//use futures::future::Future;
use futures::future::IntoFuture;
use futures_cpupool::{CpuPool,CpuFuture};

pub struct Actor {
    thr: CpuPool,
}

pub type ActorFuture<R,E> = CpuFuture<R,E>;

impl Actor {
    pub fn new() -> Actor {
        Actor {
            thr: CpuPool::new(1),
        }
    }

    pub fn call<F, R>(&self, f: F) -> ActorFuture<R::Item, R::Error>
        where F: FnOnce() -> R + Send + 'static,
        R: IntoFuture + 'static,
        R::Future: Send + 'static,
        R::Item: Send + 'static,
        R::Error: Send + 'static,
    {
        self.thr.spawn_fn(f)
    }

    pub fn cast<F,R>(&self, f: F)
        where F: FnOnce() + Send + 'static,
    {
        let _ = self.thr.spawn_fn(|| {
            f();
            let res: Result<_, ()> = Ok(());
            res
        });
    }
}

// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
