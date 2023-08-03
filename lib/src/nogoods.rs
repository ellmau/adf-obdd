//! Collection of all nogood-related structures.

use std::{
    fmt::{Debug, Display},
    ops::{BitAnd, BitOr, BitXor, BitXorAssign},
};

use crate::datatypes::Term;
use roaring::RoaringBitmap;

/// A [NoGood] and an [Interpretation] can be represented by the same structure.
/// Moreover this duality (i.e. an [Interpretation] becomes a [NoGood] is reflected by this type alias.
pub type Interpretation = NoGood;

/// Representation of a nogood by a pair of [Bitmaps][RoaringBitmap]
#[derive(Debug, Default, Clone)]
pub struct NoGood {
    active: RoaringBitmap,
    value: RoaringBitmap,
}

impl Eq for NoGood {}
impl PartialEq for NoGood {
    fn eq(&self, other: &Self) -> bool {
        (&self.active).bitxor(&other.active).is_empty()
            && (&self.value).bitxor(&other.value).is_empty()
    }
}

impl NoGood {
    /// Creates an [Interpretation] from a given Vector of [Terms][Term].
    pub fn from_term_vec(term_vec: &[Term]) -> Interpretation {
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

    /// Creates a [NoGood] representing an atomic assignment.
    pub fn new_single_nogood(pos: usize, val: bool) -> NoGood {
        let mut result = Self::default();
        let pos:u32 = pos.try_into().expect("nog-good learner implementation is based on the assumption that only u32::MAX-many variables are in place");
        result.active.insert(pos);
        if val {
            result.value.insert(pos);
        }
        result
    }

    /// Returns [None] if the pair contains inconsistent pairs.
    /// Otherwise it returns an [Interpretation] which represents the set values.
    pub fn try_from_pair_iter(
        pair_iter: &mut impl Iterator<Item = (usize, bool)>,
    ) -> Option<Interpretation> {
        let mut result = Self::default();
        let mut visit = false;
        for (idx, val) in pair_iter {
            visit = true;
            let idx:u32 = idx.try_into().expect("no-good learner implementation is based on the assumption that only u32::MAX-many variables are in place");
            let is_new = result.active.insert(idx);
            let upd = if val {
                result.value.insert(idx)
            } else {
                result.value.remove(idx)
            };
            // if the state is not new and the value is changed
            if !is_new && upd {
                return None;
            }
        }
        visit.then_some(result)
    }

    /// Creates an updated [`Vec<Term>`], based on the given [&[Term]] and the [NoGood].
    /// The parameter _update_ is set to [`true`] if there has been an update and to [`false`] otherwise
    pub fn update_term_vec(&self, term_vec: &[Term], update: &mut bool) -> Vec<Term> {
        *update = false;
        term_vec
            .iter()
            .enumerate()
            .map(|(idx, val)| {
                let idx: u32 = idx.try_into().expect(
                    "no-good learner implementation is based on the assumption \
                     that only u32::MAX-many variables are in place",
                );
                if self.active.contains(idx) {
                    if !val.is_truth_value() {
                        *update = true;
                    }
                    if self.value.contains(idx) {
                        Term::TOP
                    } else {
                        Term::BOT
                    }
                } else {
                    *val
                }
            })
            .collect()
    }

    /// Given a [NoGood] and another one, conclude a non-conflicting value which can be concluded on basis of the given one.
    pub fn conclude(&self, other: &NoGood) -> Option<(usize, bool)> {
        log::debug!("conclude: {:?} other {:?}", self, other);
        let implication = (&self.active).bitxor(&other.active).bitand(&self.active);

        let bothactive = (&self.active).bitand(&other.active);
        let mut no_matches = (&bothactive).bitand(&other.value);
        no_matches.bitxor_assign(bothactive.bitand(&self.value));

        if implication.len() == 1 && no_matches.is_empty() {
            let pos = implication
                .min()
                .expect("just checked that there is one element to be found");
            log::trace!(
                "Conclude {:?}",
                Some((pos as usize, !self.value.contains(pos)))
            );
            Some((pos as usize, !self.value.contains(pos)))
        } else {
            log::trace!("Nothing to Conclude");
            None
        }
    }

    /// Updates the [NoGood] and a second one in a disjunctive (bitor) manner.
    pub fn disjunction(&mut self, other: &NoGood) {
        self.active = (&self.active).bitor(&other.active);
        self.value = (&self.value).bitor(&other.value);
    }

    /// Returns [true] if the other [Interpretation] matches with all the assignments of the current [NoGood].
    pub fn is_violating(&self, other: &Interpretation) -> bool {
        let active = (&self.active).bitand(&other.active);
        if self.active.len() == active.len() {
            let lhs = (&active).bitand(&self.value);
            let rhs = (&active).bitand(&other.value);
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

impl From<&[Term]> for NoGood {
    fn from(term_vec: &[Term]) -> Self {
        Self::from_term_vec(term_vec)
    }
}

/// A structure to store [NoGoods][NoGood] and offer operations and deductions based on them.
#[derive(Debug)]
pub struct NoGoodStore {
    store: Vec<Vec<NoGood>>,
    duplicates: DuplicateElemination,
}

impl Display for NoGoodStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "NoGoodStats: [")?;
        for (arity, vec) in self.store.iter().enumerate() {
            writeln!(f, "{arity}: {}", vec.len())?;
            log::debug!("Nogoods:\n {:?}", vec);
        }
        write!(f, "]")
    }
}

impl NoGoodStore {
    /// Creates a new [NoGoodStore] and assumes a size compatible with the underlying [NoGood] implementation.
    pub fn new(size: u32) -> NoGoodStore {
        Self {
            store: vec![Vec::new(); size as usize],
            duplicates: DuplicateElemination::Equiv,
        }
    }

    /// Tries to create a new [NoGoodStore].
    /// Does not succeed if the size is too big for the underlying [NoGood] implementation.
    pub fn try_new(size: usize) -> Option<NoGoodStore> {
        Some(Self::new(size.try_into().ok()?))
    }

    /// Sets the behaviour when managing duplicates.
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
                            if idx >= cur_idx {
                                ng_vec.retain(|ng| !ng.is_violating(&nogood));
                            }
                        });
                    true
                }
            } {
                self.store[idx].push(nogood);
            }
        }
    }

    /// Draws a (Conclusion)[NoGood], based on the [NoGoodStore] and the given [NoGood].
    /// *Returns* [None] if there is a conflict
    pub fn conclusions(&self, nogood: &NoGood) -> Option<NoGood> {
        let mut result = nogood.clone();
        log::trace!("ng-store: {:?}", self.store);
        self.store
            .iter()
            .enumerate()
            .filter(|(len, _vec)| *len <= nogood.len())
            .filter_map(|(_len, val)| {
                NoGood::try_from_pair_iter(&mut val.iter().filter_map(|ng| ng.conclude(nogood)))
            })
            .try_fold(&mut result, |acc, ng| {
                if ng.is_violating(acc) {
                    log::trace!("ng conclusion violating");
                    None
                } else {
                    acc.disjunction(&ng);
                    Some(acc)
                }
            })?;
        if self
            .store
            .iter()
            .enumerate()
            .filter(|(len, _vec)| *len <= nogood.len())
            .any(|(_, vec)| {
                vec.iter()
                    .any(|elem| elem.is_violating(&result) || elem.is_violating(nogood))
            })
        {
            return None;
        }
        Some(result)
    }

    /// Constructs the Closure of the conclusions drawn by the nogoods with respect to the given `interpretation`
    pub(crate) fn conclusion_closure(&self, interpretation: &[Term]) -> ClosureResult {
        let mut update = true;
        let mut result = match self.conclusions(&interpretation.into()) {
            Some(val) => {
                log::trace!(
                    "conclusion-closure step 1: val:{:?} -> {:?}",
                    val,
                    val.update_term_vec(interpretation, &mut update)
                );
                val.update_term_vec(interpretation, &mut update)
            }

            None => return ClosureResult::Inconsistent,
        };
        if !update {
            return ClosureResult::NoUpdate;
        }
        while update {
            match self.conclusions(&result.as_slice().into()) {
                Some(val) => result = val.update_term_vec(&result, &mut update),
                None => return ClosureResult::Inconsistent,
            }
        }
        ClosureResult::Update(result)
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

/// If the closure had some issues, it is represented with this enum
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum ClosureResult {
    Update(Vec<Term>),
    NoUpdate,
    Inconsistent,
}

impl ClosureResult {
    /// Dead_code due to (currently) unused utility function for the [ClosureResult] enum.
    #[allow(dead_code)]
    pub fn is_update(&self) -> bool {
        matches!(self, Self::Update(_))
    }
    /// Dead_code due to (currently) unused utility function for the [ClosureResult] enum.
    #[allow(dead_code)]
    pub fn is_no_update(&self) -> bool {
        matches!(self, Self::NoUpdate)
    }
    /// Dead_code due to (currently) unused utility function for the [ClosureResult] enum.
    #[allow(dead_code)]
    pub fn is_inconsistent(&self) -> bool {
        matches!(self, Self::Inconsistent)
    }
}

impl TryInto<Vec<Term>> for ClosureResult {
    type Error = &'static str;

    fn try_into(self) -> Result<Vec<Term>, Self::Error> {
        match self {
            ClosureResult::Update(val) => Ok(val),
            ClosureResult::NoUpdate => Err("No update occurred, use the old value instead"),
            ClosureResult::Inconsistent => Err("Inconsistency occurred"),
        }
    }
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
        let ng2 = NoGood::from_term_vec(&[Term::TOP, Term(22), Term(13232), Term::BOT, Term::TOP]);
        let ng3 = NoGood::from_term_vec(&[
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

        // conclusions on empty knowledge
        let ng4 = NoGood::from_term_vec(&[Term::TOP]);
        let ng5 = NoGood::from_term_vec(&[Term::BOT]);
        let ng6 = NoGood::from_term_vec(&[]);

        assert_eq!(ng4.conclude(&ng6), Some((0, false)));
        assert_eq!(ng5.conclude(&ng6), Some((0, true)));
        assert_eq!(ng6.conclude(&ng5), None);
        assert_eq!(ng4.conclude(&ng5), None);

        let ng_a = NoGood::from_term_vec(&[Term::BOT, Term(22)]);
        let ng_b = NoGood::from_term_vec(&[Term(22), Term::TOP]);

        assert_eq!(ng_a.conclude(&ng_b), Some((0, true)));
    }

    #[test]
    fn violate() {
        let ng1 = NoGood::from_term_vec(&[Term::TOP, Term(22), Term::TOP, Term::BOT, Term::TOP]);
        let ng2 = NoGood::from_term_vec(&[Term::TOP, Term(22), Term(13232), Term::BOT, Term::TOP]);
        let ng3 = NoGood::from_term_vec(&[
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

        assert_eq!(ng4, NoGood::new_single_nogood(0, true));
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

        ngs.add_ng(NoGood::from_term_vec(&[Term(22), Term::BOT, Term(22)]));

        assert_eq!(
            ngs.store
                .iter()
                .fold(0, |acc, ng_vec| { acc + ng_vec.len() }),
            6
        );

        ngs.add_ng(NoGood::from_term_vec(&[Term(22), Term::BOT, Term::BOT]));

        assert_eq!(
            ngs.store
                .iter()
                .fold(0, |acc, ng_vec| { acc + ng_vec.len() }),
            6
        );

        assert!(NoGood::from_term_vec(&[Term(22), Term::BOT, Term(22)])
            .is_violating(&NoGood::from_term_vec(&[Term(22), Term::BOT, Term::BOT])));
    }

    #[test]
    fn ng_store_conclusions() {
        let mut ngs = NoGoodStore::new(5);

        let ng1 = NoGood::from_term_vec(&[Term::BOT]);

        ngs.add_ng(ng1.clone());
        assert_eq!(ng1.conclude(&ng1), None);
        assert_eq!(
            ng1.conclude(&NoGood::from_term_vec(&[Term(33)])),
            Some((0, true))
        );
        assert_eq!(ngs.conclusions(&ng1), None);
        assert_ne!(ngs.conclusions(&NoGood::from_term_vec(&[Term(33)])), None);
        assert_eq!(
            ngs.conclusions(&NoGood::from_term_vec(&[Term(33)]))
                .expect("just checked with prev assertion")
                .update_term_vec(&[Term(33)], &mut false),
            vec![Term::TOP]
        );

        let ng2 = NoGood::from_term_vec(&[Term(123), Term::TOP, Term(234), Term(345)]);
        let ng3 = NoGood::from_term_vec(&[Term::TOP, Term::BOT, Term::TOP, Term(345)]);

        ngs.add_ng(ng2);
        ngs.add_ng(ng3);

        log::debug!("issues start here");
        assert!(ngs
            .conclusions(&NoGood::from_term_vec(&[Term::TOP]))
            .is_some());
        assert_eq!(
            ngs.conclusions(&[Term::TOP].as_slice().into())
                .expect("just checked with prev assertion")
                .update_term_vec(&[Term::TOP, Term(4), Term(5), Term(6), Term(7)], &mut false),
            vec![Term::TOP, Term::BOT, Term(5), Term(6), Term(7)]
        );
        assert!(ngs
            .conclusions(&NoGood::from_term_vec(&[
                Term::TOP,
                Term::BOT,
                Term(5),
                Term(6),
                Term(7)
            ]))
            .is_some());

        ngs = NoGoodStore::new(10);
        ngs.add_ng([Term::BOT].as_slice().into());
        ngs.add_ng(
            [Term::TOP, Term::BOT, Term(33), Term::TOP]
                .as_slice()
                .into(),
        );
        ngs.add_ng(
            [Term::TOP, Term::BOT, Term(33), Term(33), Term::BOT]
                .as_slice()
                .into(),
        );
        ngs.add_ng([Term::TOP, Term::TOP].as_slice().into());

        let interpr: Vec<Term> = vec![
            Term(123),
            Term(233),
            Term(345),
            Term(456),
            Term(567),
            Term(678),
            Term(789),
            Term(899),
            Term(999),
            Term(1000),
        ];
        let concl = ngs.conclusions(&interpr.as_slice().into());
        assert_eq!(concl, Some(NoGood::from_term_vec(&[Term::TOP])));
        let mut update = false;
        let new_interpr = concl
            .expect("just tested in assert")
            .update_term_vec(&interpr, &mut update);
        assert_eq!(
            new_interpr,
            vec![
                Term::TOP,
                Term(233),
                Term(345),
                Term(456),
                Term(567),
                Term(678),
                Term(789),
                Term(899),
                Term(999),
                Term(1000)
            ]
        );
        assert!(update);

        let new_int_2 = ngs
            .conclusions(&new_interpr.as_slice().into())
            .map(|val| val.update_term_vec(&new_interpr, &mut update))
            .expect("Should return a value");
        assert_eq!(
            new_int_2,
            vec![
                Term::TOP,
                Term::BOT,
                Term(345),
                Term(456),
                Term(567),
                Term(678),
                Term(789),
                Term(899),
                Term(999),
                Term(1000)
            ]
        );
        assert!(update);

        let new_int_3 = ngs
            .conclusions(&new_int_2.as_slice().into())
            .map(|val| val.update_term_vec(&new_int_2, &mut update))
            .expect("Should return a value");

        assert_eq!(
            new_int_3,
            vec![
                Term::TOP,
                Term::BOT,
                Term(345),
                Term::BOT,
                Term::TOP,
                Term(678),
                Term(789),
                Term(899),
                Term(999),
                Term(1000)
            ]
        );
        assert!(update);

        let concl4 = ngs.conclusions(&new_int_3.as_slice().into());
        assert_ne!(concl4, None);

        let new_int_4 = ngs
            .conclusions(&new_int_3.as_slice().into())
            .map(|val| val.update_term_vec(&new_int_3, &mut update))
            .expect("Should return a value");

        assert_eq!(
            new_int_4,
            vec![
                Term::TOP,
                Term::BOT,
                Term(345),
                Term::BOT,
                Term::TOP,
                Term(678),
                Term(789),
                Term(899),
                Term(999),
                Term(1000)
            ]
        );
        assert!(!update);

        // inconsistence
        let interpr = vec![
            Term::TOP,
            Term::TOP,
            Term::BOT,
            Term::BOT,
            Term(111),
            Term(678),
            Term(789),
            Term(899),
            Term(999),
            Term(1000),
        ];

        assert_eq!(ngs.conclusions(&interpr.as_slice().into()), None);

        ngs = NoGoodStore::new(6);
        ngs.add_ng(
            [Term(1), Term(1), Term(1), Term(0), Term(0), Term(1)]
                .as_slice()
                .into(),
        );
        ngs.add_ng(
            [Term(1), Term(1), Term(8), Term(0), Term(0), Term(11)]
                .as_slice()
                .into(),
        );
        ngs.add_ng([Term(22), Term(1)].as_slice().into());

        assert_eq!(
            ngs.conclusions(
                &[Term(1), Term(3), Term(3), Term(9), Term(0), Term(1)]
                    .as_slice()
                    .into(),
            ),
            Some(NoGood::from_term_vec(&[
                Term(1),
                Term(0),
                Term(3),
                Term(9),
                Term(0),
                Term(1)
            ]))
        );
    }

    #[test]
    fn conclusion_closure() {
        let mut ngs = NoGoodStore::new(10);
        ngs.add_ng([Term::BOT].as_slice().into());
        ngs.add_ng(
            [Term::TOP, Term::BOT, Term(33), Term::TOP]
                .as_slice()
                .into(),
        );
        ngs.add_ng(
            [Term::TOP, Term::BOT, Term(33), Term(33), Term::BOT]
                .as_slice()
                .into(),
        );
        ngs.add_ng([Term::TOP, Term::TOP].as_slice().into());

        let interpr: Vec<Term> = vec![
            Term(123),
            Term(233),
            Term(345),
            Term(456),
            Term(567),
            Term(678),
            Term(789),
            Term(899),
            Term(999),
            Term(1000),
        ];

        let result = ngs.conclusion_closure(&interpr);
        assert!(result.is_update());
        let resultint: Vec<Term> = result.try_into().expect("just checked conversion");
        assert_eq!(
            resultint,
            vec![
                Term::TOP,
                Term::BOT,
                Term(345),
                Term::BOT,
                Term::TOP,
                Term(678),
                Term(789),
                Term(899),
                Term(999),
                Term(1000)
            ]
        );
        let result_no_upd = ngs.conclusion_closure(&resultint);

        assert!(result_no_upd.is_no_update());
        assert_eq!(
            <ClosureResult as TryInto<Vec<Term>>>::try_into(result_no_upd)
                .expect_err("just checked that it is an error"),
            "No update occurred, use the old value instead"
        );

        let inconsistent_interpr = vec![
            Term::TOP,
            Term::TOP,
            Term::BOT,
            Term::BOT,
            Term(111),
            Term(678),
            Term(789),
            Term(899),
            Term(999),
            Term(1000),
        ];
        let result_inconsistent = ngs.conclusion_closure(&inconsistent_interpr);

        assert!(result_inconsistent.is_inconsistent());
        assert_eq!(
            <ClosureResult as TryInto<Vec<Term>>>::try_into(result_inconsistent)
                .expect_err("just checked that it is an error"),
            "Inconsistency occurred"
        );

        ngs = NoGoodStore::new(6);
        ngs.add_ng(
            [Term(1), Term(1), Term(1), Term(0), Term(0), Term(1)]
                .as_slice()
                .into(),
        );
        ngs.add_ng(
            [Term(1), Term(1), Term(8), Term(0), Term(0), Term(11)]
                .as_slice()
                .into(),
        );
        ngs.add_ng([Term(22), Term(1)].as_slice().into());

        assert_eq!(
            ngs.conclusion_closure(&[Term(1), Term(3), Term(3), Term(9), Term(0), Term(1)]),
            ClosureResult::Update(vec![Term(1), Term(0), Term(3), Term(9), Term(0), Term(1)])
        );
    }
}
