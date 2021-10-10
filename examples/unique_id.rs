// cooper-rs/examples/unique_id.rs
//
// This is an example app for the Rust "cooper" library.
//
// Copyright (c) 2021, Frank Pagliughi <fpagliughi@mindspring.com>
// All Rights Reserved
//
// Licensed under the MIT license:
//   <LICENSE or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according
// to those terms.
//

use cooper::Actor;
use smol::block_on;

/// An actor that can create unique integer values from a counting integer.
#[derive(Default, Clone)]
pub struct UniqueId {
    actor: Actor<u32>,
}

impl UniqueId {
    /// Create a new UniqueId actor
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets a unique ID as the next integer value in the sequence.
    pub async fn get_unique_id(&self) -> u32 {
        self.actor
            .call(|_, state| {
                Box::pin(async move {
                    *state += 1;
                    Some(*state)
                })
            })
            .await
    }
}

// --------------------------------------------------------------------------

fn main() {
    block_on(async {
        let actor = UniqueId::new();

        let n = actor.get_unique_id().await;
        println!("ID: {}", n);
        assert_eq!(n, 1);

        let n = actor.get_unique_id().await;
        println!("ID: {}", n);
        assert_eq!(n, 2);

        let n = actor.get_unique_id().await;
        println!("ID: {}", n);
        assert_eq!(n, 3);
    });
}
