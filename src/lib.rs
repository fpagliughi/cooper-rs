// cooper/src/lib/rs
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
//
//! cooper

mod actor;
pub use actor::*;

#[cfg(feature = "threaded")]
mod threaded_actor;

#[cfg(feature = "threaded")]
pub use threaded_actor::*;
