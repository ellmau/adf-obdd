//! Representation of various NoGood implementations.

pub mod threevalued;
pub mod twovalued;
use std::fmt::Display;

pub use twovalued::*;

// pub trait NoGood: std::fmt::Debug + Eq {
//     /// Returns the number of set (i.e. active) variables.
//     fn len(&self) -> usize;
// }

// #[derive(Debug)]
// pub struct NoGoodStore<N>
// where
//     N: NoGood + Clone,
// {
//     store: Vec<Vec<N>>,
// }

// impl<T> NoGoodStore<T>
// where
//     T: NoGood + Clone,
// {
//     /// Creates a new [NoGoodStore] and assumes a size compatible with the underlying [NoGood] implementations.
//     pub fn new(size: u32) -> Self {
//         Self {
//             store: vec![Vec::new(); size as usize],
//         }
//     }

//     /// Tries to create a new [NoGoodStore].
//     /// Does not succeed if the size is too big for the underlying [NoGood] implementations.
//     pub fn try_new(size: usize) -> Option<Self> {
//         Some(Self::new(size.try_into().ok()?))
//     }

//     pub fn add_ng(&mut self, nogood: T) {
//         let mut idx = nogood.len();
//         if idx > 0 {
//             idx -= 1;
//             if !self.store[idx].contains(&nogood) {
//                 self.store[idx].push(nogood);
//             }
//         }
//     }
// }

// impl<T> Display for NoGoodStore<T>
// where
//     T: NoGood + Clone,
// {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         writeln!(f, "NoGoodStats: [")?;
//         for (arity, vec) in self.store.iter().enumerate() {
//             writeln!(f, "{arity}: {}", vec.len())?;
//             log::debug!("Nogoods:\n {:?}", vec);
//         }
//         write!(f, "]")
//     }
// }
