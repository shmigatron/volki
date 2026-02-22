//! Threading — spawn, sleep.

pub mod sleep;
pub mod spawn;

pub use sleep::sleep;
pub use spawn::{spawn, JoinHandle};
