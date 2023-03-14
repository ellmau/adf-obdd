//! Three-valued NoGoods

use roaring::RoaringBitmap;

use crate::datatypes::Term;

/// An interpretation
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Interpretation(NoGood);

impl From<NoGood> for Interpretation {
    fn from(ng: NoGood) -> Self {
        Interpretation(ng)
    }
}

impl Interpretation {
    /// Creates an [Interpretation], based on the given Vector of [Terms][Term]
    pub fn from_term_vec(term_vec: &[Term]) -> Self {
        Interpretation(NoGood::from_term_vec(term_vec))
    }
    fn invert(mut self) -> Self {
        self.0 = self.0.invert();
        self
    }

    fn active(&self) -> RoaringBitmap {
        self.0.active()
    }

    fn hint(&self) -> RoaringBitmap {
        ((&self.0.can_be_true & &self.0.can_be_false)
            | (&self.0.can_be_true & &self.0.can_be_und)
            | (&self.0.can_be_false & &self.0.can_be_und))
            & &self.active()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    fn idx_as_triple(&self, idx: u32) -> (bool, bool, bool) {
        self.0.idx_as_triple(idx)
    }

    pub fn conclude(mut self, ng: &NoGood) -> Conclusion {
        log::debug!("conclude: {:?} with NoGood {:?}", self, ng);
        if self.len() + 1 < ng.len() { // not matching
        }
        let interpretation_cred_active = self.active();
        let interpretation_active = &interpretation_cred_active & (self.hint() ^ self.0.full());
        let nogood_active = ng.active();

        let implication = (&interpretation_active ^ &nogood_active) & &nogood_active;
        let scep_match = self.sceptical_matches(ng);

        match implication.len() {
            // learning might be possible
            1 => {
                if (scep_match & interpretation_active).len() == nogood_active.len() - 1 {
                    let idx: u32 = implication
                        .min()
                        .expect("Checked that implication has a minimal value to be found.");
                    let (ng_t, ng_f, ng_u) = ng.idx_as_triple(idx);
                    let mut changes: u8 = 0;
                    if ng_t && self.0.can_be_true.remove(idx) {
                        changes += 1;
                    }
                    if ng_f && self.0.can_be_false.remove(idx) {
                        changes += 1;
                    }
                    if ng_u && self.0.can_be_und.remove(idx) {
                        changes += 1;
                    }
                    if self.0.can_be_true.contains(idx)
                        || self.0.can_be_false.contains(idx)
                        || self.0.can_be_und.contains(idx)
                    {
                        if changes > 0 {
                            Conclusion::Update(self)
                        } else {
                            Conclusion::NoChange(self)
                        }
                    } else {
                        // inconsistency
                        Conclusion::Inconsistent(self)
                    }
                } else {
                    Conclusion::NoChange(self)
                }
            }
            // ng-consistency check might be possible
            _ => {
                if (scep_match & interpretation_active).len() == nogood_active.len() {
                    Conclusion::Inconsistent(self)
                } else {
                    Conclusion::NoChange(self)
                }
            }
        }
    }
    fn credulous_matches(&self, other: &NoGood) -> RoaringBitmap {
        let true_match = (&self.0.can_be_true ^ &other.can_be_true) ^ self.0.full();
        let false_match = (&self.0.can_be_false ^ &other.can_be_false) ^ self.0.full();
        let und_match = (&self.0.can_be_und ^ &other.can_be_und) ^ self.0.full();

        true_match | false_match | und_match
    }

    fn sceptical_matches(&self, other: &NoGood) -> RoaringBitmap {
        let true_match = (&self.0.can_be_true ^ &other.can_be_true) ^ self.0.full();
        let false_match = (&self.0.can_be_false ^ &other.can_be_false) ^ self.0.full();
        let und_match = (&self.0.can_be_und ^ &other.can_be_und) ^ self.0.full();

        true_match & false_match & und_match
    }

    /// Returns the credulous and sceptical matches of the [Interpretation] and a given [NoGood]
    fn matches(&self, other: &NoGood) -> (RoaringBitmap, RoaringBitmap) {
        let true_match = (&self.0.can_be_true ^ &other.can_be_true) ^ self.0.full();
        let false_match = (&self.0.can_be_false ^ &other.can_be_false) ^ self.0.full();
        let und_match = (&self.0.can_be_und ^ &other.can_be_und) ^ self.0.full();

        (
            &true_match | &false_match | &und_match,
            true_match & false_match & und_match,
        )
    }
}

#[derive(Debug)]
pub enum Conclusion {
    Update(Interpretation),
    NoChange(Interpretation),
    Inconsistent(Interpretation),
}

impl Conclusion {
    fn consistent(self) -> Option<Interpretation> {
        match self {
            Conclusion::Update(val) => Some(val),
            Conclusion::NoChange(val) => Some(val),
            Conclusion::Inconsistent(_) => None,
        }
    }
}

/// Representation of an Interpretation with Hints about truth-values.
#[derive(Debug, Clone)]
pub struct NoGood {
    can_be_true: RoaringBitmap,
    can_be_false: RoaringBitmap,
    can_be_und: RoaringBitmap,
    size: u32,
}

impl Default for NoGood {
    fn default() -> Self {
        Self {
            can_be_true: Default::default(),
            can_be_false: Default::default(),
            can_be_und: Default::default(),
            size: Default::default(),
        }
    }
}

impl Eq for NoGood {}
impl PartialEq for NoGood {
    fn eq(&self, other: &Self) -> bool {
        (&self.can_be_true ^ &other.can_be_true).is_empty()
            && (&self.can_be_false ^ &other.can_be_false).is_empty()
            && (&self.can_be_und ^ &other.can_be_und).is_empty()
    }
}

impl From<Interpretation> for NoGood {
    fn from(interpretation: Interpretation) -> Self {
        interpretation.0.invert()
    }
}

impl NoGood {
    fn invert(mut self) -> Self {
        self.can_be_true ^= self.full();
        self.can_be_false ^= self.full();
        self.can_be_und ^= self.full();
        self
    }

    fn idx_as_triple(&self, idx: u32) -> (bool, bool, bool) {
        (
            self.can_be_true.contains(idx),
            self.can_be_false.contains(idx),
            self.can_be_und.contains(idx),
        )
    }

    fn full(&self) -> RoaringBitmap {
        let mut result = RoaringBitmap::default();
        result.insert_range(0..self.size);
        result
    }

    /// Creates a [NoGood], based on the given Vector of [Terms][Term]
    pub fn from_term_vec(term_vec: &[Term]) -> Self {
        let mut result = Self {
            can_be_true: RoaringBitmap::default(),
            can_be_false: RoaringBitmap::default(),
            can_be_und: RoaringBitmap::default(),
            size: TryInto::<u32>::try_into(term_vec.len()).expect("no-good learner implementation is based on the assumption that only u32::MAX-many variables are in place"),
        };
        result.can_be_true.insert_range(0..result.size);
        result.can_be_false.insert_range(0..result.size);
        result.can_be_und.insert_range(0..result.size);
        log::debug!("{:?}", result);
        term_vec.iter().enumerate().for_each(|(idx, val)| {
	    let idx:u32 = idx.try_into().expect("no-good learner implementation is based on the assumption that only u32::MAX-many variables are in place");
            if val.is_truth_value() {
		log::trace!("idx {idx} val: {val:?}");
                if val.is_true() {
		    result.can_be_false.remove(idx);
		    result.can_be_und.remove(idx);
                }else{
		    result.can_be_true.remove(idx);
		    result.can_be_und.remove(idx);
		}
            }
        });
        log::debug!("{:?}", result);
        result
    }

    /// Returns the number of set variables.
    pub fn len(&self) -> usize {
        self.active()
            .len()
            .try_into()
            .expect("Expecting to be on a 64 bit system")
    }

    fn active(&self) -> RoaringBitmap {
        (&self.can_be_true & &self.can_be_false & &self.can_be_und) ^ &self.full()
    }

    fn no_matches(&self, other: &NoGood) -> RoaringBitmap {
        let no_true_match = &self.can_be_true ^ &other.can_be_true;
        let no_false_match = &self.can_be_false ^ &other.can_be_false;
        let no_und_match = &self.can_be_und ^ &other.can_be_und;

        no_true_match | no_false_match | no_und_match
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

        assert_eq!(terms.len(), 5);
        assert_eq!(ng.size, 5);
        log::debug!("{:?}", ng.active());
        assert_eq!(ng.active().len(), 3);
        assert_eq!(ng.idx_as_triple(0), (true, false, false));
        assert_eq!(ng.idx_as_triple(1), (true, true, true));
        assert_eq!(ng.idx_as_triple(2), (true, true, true));
        assert_eq!(ng.idx_as_triple(3), (false, true, false));
        assert_eq!(ng.idx_as_triple(4), (true, false, false));
    }

    #[test]
    fn create_interpretation() {
        let terms = vec![Term::TOP, Term(22), Term(13232), Term::BOT, Term::TOP];
        let interpretation = Interpretation::from_term_vec(&terms);

        assert_eq!(interpretation.active().len(), 3);
        assert_eq!(interpretation.idx_as_triple(0), (true, false, false));
        assert_eq!(interpretation.idx_as_triple(1), (true, true, true));
        assert_eq!(interpretation.idx_as_triple(2), (true, true, true));
        assert_eq!(interpretation.idx_as_triple(3), (false, true, false));
        assert_eq!(interpretation.idx_as_triple(4), (true, false, false));
    }

    #[test]
    fn conclude() {
        let ng1 = Interpretation::from_term_vec(&[
            Term::TOP,
            Term(22),
            Term::TOP,
            Term::BOT,
            Term::TOP,
            Term(22),
        ]);
        let ng2 = Interpretation::from_term_vec(&[
            Term::TOP,
            Term(22),
            Term(13232),
            Term::BOT,
            Term::TOP,
            Term(22),
        ]);
        let ng3 = Interpretation::from_term_vec(&[
            Term::TOP,
            Term(22),
            Term::BOT,
            Term::BOT,
            Term::TOP,
            Term(22),
        ]);

        let mut ng4 = ng3.clone();
        ng4.0.can_be_und.insert(2);
        ng4.0.can_be_false.remove(2);

        let ng_too_big = Interpretation::from_term_vec(&[
            Term::TOP,
            Term(22),
            Term::TOP,
            Term::BOT,
            Term::TOP,
            Term::BOT,
        ]);

        assert!(ng1.clone().conclude(&ng2.0).consistent().is_none());
        let result = ng2.clone().conclude(&ng1.0).consistent().unwrap();
        assert_eq!(result.idx_as_triple(0), (true, false, false));
        assert_eq!(result.idx_as_triple(1), (true, true, true));
        assert_eq!(result.idx_as_triple(2), (false, true, true));
        assert_eq!(result.idx_as_triple(3), (false, true, false));
        assert_eq!(result.idx_as_triple(4), (true, false, false));

        let result = result.conclude(&ng3.0).consistent().unwrap();
        assert_eq!(result.idx_as_triple(0), (true, false, false));
        assert_eq!(result.idx_as_triple(1), (true, true, true));
        assert_eq!(result.idx_as_triple(2), (false, false, true));
        assert_eq!(result.idx_as_triple(3), (false, true, false));
        assert_eq!(result.idx_as_triple(4), (true, false, false));

        assert!(result.conclude(&ng4.0).consistent().is_none());

        let result = ng2
            .clone()
            .conclude(&ng1.0)
            .consistent()
            .and_then(|i| i.conclude(&ng4.0).consistent())
            .unwrap();

        assert_eq!(result.idx_as_triple(0), (true, false, false));
        assert_eq!(result.idx_as_triple(1), (true, true, true));
        assert_eq!(result.idx_as_triple(2), (false, true, false));
        assert_eq!(result.idx_as_triple(3), (false, true, false));
        assert_eq!(result.idx_as_triple(4), (true, false, false));

        log::debug!("{:?}", ng2.clone().conclude(&ng_too_big.0));
        if let Conclusion::NoChange(interpretation) = ng2.clone().conclude(&ng_too_big.0) {
            assert_eq!(interpretation, ng2);
        } else {
            panic!("test failed");
        }
    }
}
