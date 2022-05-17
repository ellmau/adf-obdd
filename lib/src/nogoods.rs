//! Collection of all nogood-related structures.

use std::{
    borrow::Borrow,
    ops::{BitAnd, BitXor},
};

use crate::datatypes::Term;
use roaring::RoaringBitmap;

/// Representation of a nogood by a pair of [Bitmaps][RoaringBitmap]
#[derive(Debug, Default, Clone)]
pub struct NoGood {
    active: RoaringBitmap,
    value: RoaringBitmap,
}

impl Eq for NoGood {}
impl PartialEq for NoGood {
    fn eq(&self, other: &Self) -> bool {
        self.active
            .borrow()
            .bitxor(other.active.borrow())
            .is_empty()
            && self.value.borrow().bitxor(other.value.borrow()).is_empty()
    }
}

impl NoGood {
    /// Creates a [NoGood] from a given Vector of [Terms][Term].
    pub fn from_term_vec(term_vec: &[Term]) -> NoGood {
        let mut result = Self::default();
        term_vec.iter().enumerate().for_each(|(idx, val)| {
	    let idx:u32 = idx.try_into().expect("no-good learner implementation is based on the assumption that only u32::MAX-many variables are in place");
            if val.is_truth_value() {
                result.active.insert(idx);
                if val.is_true() {
                    result.value.insert(idx);
                }
            }
        });
        result
    }

    /// Given a [NoGood] and another one, conclude a non-conflicting value which can be concluded on basis of the given one.
    pub fn conclude(&self, other: &NoGood) -> Option<(usize, bool)> {
        let implication = self
            .active
            .borrow()
            .bitxor(other.active.borrow())
            .bitand(self.active.borrow());
        log::debug!("{:?}", implication);
        if implication.len() == 1 {
            let pos = implication
                .min()
                .expect("just checked that there is one element to be found");
            Some((pos as usize, !self.value.contains(pos)))
        } else {
            None
        }
    }

    /// Returns [true] if the other [NoGood] matches with all the assignments of the current [NoGood].
    pub fn is_violating(&self, other: &NoGood) -> bool {
        let active = self.active.borrow().bitand(other.active.borrow());
        if self.active.len() == active.len() {
            let lhs = active.borrow().bitand(self.value.borrow());
            let rhs = active.borrow().bitand(other.value.borrow());
            if lhs.bitxor(rhs).is_empty() {
                return true;
            }
        }
        false
    }

    /// Returns the number of set (i.e. active) bits.
    pub fn len(&self) -> usize {
        self.active
            .len()
            .try_into()
            .expect("expecting to be on a 64 bit system")
    }

    #[must_use]
    /// Returns [true] if the [NoGood] does not set any value.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A structure to store [NoGoods][NoGood] and offer operations and deductions based on them.
#[derive(Debug)]
pub struct NoGoodStore {
    store: Vec<Vec<NoGood>>,
    duplicates: DuplicateElemination,
}

impl NoGoodStore {
    /// Creates a new [NoGoodStore] and assumes a size compatible with the underlying [NoGood] implementation.
    pub fn new(size: u32) -> NoGoodStore {
        Self {
            store: vec![Vec::new(); size as usize],
            duplicates: DuplicateElemination::None,
        }
    }

    /// Tries to create a new [NoGoodStore].
    /// Does not succeed if the size is too big for the underlying [NoGood] implementation
    pub fn try_new(size: usize) -> Option<NoGoodStore> {
        match TryInto::<u32>::try_into(size) {
            Ok(val) => Some(Self::new(val)),
            Err(_) => None,
        }
    }

    /// Sets the behaviour when managing duplicates
    pub fn set_dup_elem(&mut self, mode: DuplicateElemination) {
        self.duplicates = mode;
    }

    /// Adds a given [NoGood]
    pub fn add_ng(&mut self, nogood: NoGood) {
        let mut idx = nogood.len();
        if idx > 0 {
            idx -= 1;
            if match self.duplicates {
                DuplicateElemination::None => true,
                DuplicateElemination::Equiv => !self.store[idx].contains(&nogood),
                DuplicateElemination::Subsume => {
                    self.store
                        .iter_mut()
                        .enumerate()
                        .for_each(|(cur_idx, ng_vec)| {
                            if idx <= cur_idx {
                                ng_vec.retain(|ng| !nogood.is_violating(ng));
                            }
                        });
                    true
                }
            } {
                self.store[idx].push(nogood);
            }
        }
    }
}

/// Allows to define how costly the DuplicateElemination is done.
#[derive(Debug, Copy, Clone)]
pub enum DuplicateElemination {
    /// No Duplicate Detection
    None,
    /// Only check weak equivalence
    Equiv,
    /// Check for subsumptions
    Subsume,
}

#[cfg(test)]
mod test {
    use super::*;
    use test_log::test;

    #[test]
    fn create_ng() {
        let terms = vec![Term::TOP, Term(22), Term(13232), Term::BOT, Term::TOP];
        let ng = NoGood::from_term_vec(&terms);

        assert_eq!(ng.active.len(), 3);
        assert_eq!(ng.value.len(), 2);
        assert!(ng.active.contains(0));
        assert!(!ng.active.contains(1));
        assert!(!ng.active.contains(2));
        assert!(ng.active.contains(3));
        assert!(ng.active.contains(4));

        assert!(ng.value.contains(0));
        assert!(!ng.value.contains(1));
        assert!(!ng.value.contains(2));
        assert!(!ng.value.contains(3));
        assert!(ng.value.contains(4));
    }

    #[test]
    fn conclude() {
        let ng1 = NoGood::from_term_vec(&[Term::TOP, Term(22), Term::TOP, Term::BOT, Term::TOP]);
        let ng2 = NoGood::from_term_vec(&vec![
            Term::TOP,
            Term(22),
            Term(13232),
            Term::BOT,
            Term::TOP,
        ]);
        let ng3 = NoGood::from_term_vec(&vec![
            Term::TOP,
            Term(22),
            Term(13232),
            Term::BOT,
            Term::TOP,
            Term::BOT,
        ]);

        assert_eq!(ng1.conclude(&ng2), Some((2, false)));
        assert_eq!(ng1.conclude(&ng1), None);
        assert_eq!(ng2.conclude(&ng1), None);
        assert_eq!(ng1.conclude(&ng3), Some((2, false)));
        assert_eq!(ng3.conclude(&ng1), Some((5, true)));
        assert_eq!(ng3.conclude(&ng2), Some((5, true)));
    }

    #[test]
    fn violate() {
        let ng1 = NoGood::from_term_vec(&[Term::TOP, Term(22), Term::TOP, Term::BOT, Term::TOP]);
        let ng2 = NoGood::from_term_vec(&vec![
            Term::TOP,
            Term(22),
            Term(13232),
            Term::BOT,
            Term::TOP,
        ]);
        let ng3 = NoGood::from_term_vec(&vec![
            Term::TOP,
            Term(22),
            Term(13232),
            Term::BOT,
            Term::TOP,
            Term::BOT,
        ]);
        let ng4 = NoGood::from_term_vec(&[Term::TOP]);

        assert!(ng4.is_violating(&ng1));
        assert!(!ng1.is_violating(&ng4));
        assert!(ng2.is_violating(&ng3));
        assert!(!ng3.is_violating(&ng2));
    }

    #[test]
    fn add_ng() {
        let mut ngs = NoGoodStore::new(5);
        let ng1 = NoGood::from_term_vec(&[Term::TOP]);
        let ng2 = NoGood::from_term_vec(&[Term(22), Term::TOP]);
        let ng3 = NoGood::from_term_vec(&[Term(22), Term(22), Term::TOP]);
        let ng4 = NoGood::from_term_vec(&[Term(22), Term(22), Term(22), Term::TOP]);
        let ng5 = NoGood::from_term_vec(&[Term::BOT]);

        assert!(!ng1.is_violating(&ng5));
        assert!(ng1.is_violating(&ng1));

        ngs.add_ng(ng1.clone());
        ngs.add_ng(ng2.clone());
        ngs.add_ng(ng3.clone());
        ngs.add_ng(ng4.clone());
        ngs.add_ng(ng5.clone());

        assert_eq!(
            ngs.store
                .iter()
                .fold(0, |acc, ng_vec| { acc + ng_vec.len() }),
            5
        );

        ngs.set_dup_elem(DuplicateElemination::Equiv);

        ngs.add_ng(ng1.clone());
        ngs.add_ng(ng2.clone());
        ngs.add_ng(ng3.clone());
        ngs.add_ng(ng4.clone());
        ngs.add_ng(ng5.clone());

        assert_eq!(
            ngs.store
                .iter()
                .fold(0, |acc, ng_vec| { acc + ng_vec.len() }),
            5
        );
        ngs.set_dup_elem(DuplicateElemination::Subsume);
        ngs.add_ng(ng1);
        ngs.add_ng(ng2);
        ngs.add_ng(ng3);
        ngs.add_ng(ng4);
        ngs.add_ng(ng5);

        assert_eq!(
            ngs.store
                .iter()
                .fold(0, |acc, ng_vec| { acc + ng_vec.len() }),
            5
        );
        ngs.add_ng(NoGood::from_term_vec(&[Term(22), Term::BOT, Term::BOT]));

        assert_eq!(
            ngs.store
                .iter()
                .fold(0, |acc, ng_vec| { acc + ng_vec.len() }),
            6
        );

        ngs.add_ng(NoGood::from_term_vec(&[Term(22), Term::BOT, Term(22)]));

        assert_eq!(
            ngs.store
                .iter()
                .fold(0, |acc, ng_vec| { acc + ng_vec.len() }),
            6
        );

        assert!(NoGood::from_term_vec(&[Term(22), Term::BOT, Term(22)])
            .is_violating(&NoGood::from_term_vec(&[Term(22), Term::BOT, Term::BOT])));
    }
}
