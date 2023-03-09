//! Three-valued NoGoods

use std::borrow::Borrow;

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
}

/// Representation of an Interpretation with Hints about truth-values.
#[derive(Debug, Default, Clone)]
pub struct NoGood {
    can_be_true: RoaringBitmap,
    can_be_false: RoaringBitmap,
    can_be_und: RoaringBitmap,
}

impl Eq for NoGood {}
impl PartialEq for NoGood {
    fn eq(&self, other: &Self) -> bool {
        (self.can_be_true.borrow() ^ other.can_be_true.borrow()).is_empty()
            && (self.can_be_false.borrow() ^ other.can_be_false.borrow()).is_empty()
            && (self.can_be_und.borrow() ^ other.can_be_und.borrow()).is_empty()
    }
}

impl From<Interpretation> for NoGood {
    fn from(interpretation: Interpretation) -> Self {
        interpretation.0.invert()
    }
}

impl NoGood {
    fn invert(mut self) -> Self {
        self.can_be_true ^= RoaringBitmap::full();
        self.can_be_false ^= RoaringBitmap::full();
        self.can_be_und ^= RoaringBitmap::full();
        self
    }

    /// Creates a [NoGood], based on the given Vector of [Terms][Term]
    pub fn from_term_vec(term_vec: &[Term]) -> Self {
        let mut result = Self {
            can_be_true: RoaringBitmap::full(),
            can_be_false: RoaringBitmap::full(),
            can_be_und: RoaringBitmap::full(),
        };
        term_vec.iter().enumerate().for_each(|(idx, val)| {
	    let idx:u32 = idx.try_into().expect("no-good learner implementation is based on the assumption that only u32::MAX-many variables are in place");
            if val.is_truth_value() {
                if val.is_true() {
		    result.can_be_false.remove(idx);
		    result.can_be_und.remove(idx);
                }else{
		    result.can_be_true.remove(idx);
		    result.can_be_und.remove(idx);
		}
            }
        });
        result
    }
}
