//! Core collections — Vec, String, Box, HashMap, HashSet, VecDeque.

pub mod raw_vec;
pub mod vec;
pub mod string;
pub mod boxed;
pub mod hash;
pub mod hash_map;
pub mod hash_set;
pub mod json;
pub mod vec_deque;
pub mod xml;

pub use boxed::Box;
pub use hash_map::HashMap;
pub use hash_set::HashSet;
pub use string::String;
pub use string::ToString;
pub use vec::Vec;
pub use vec_deque::VecDeque;
