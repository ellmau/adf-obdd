//! To represent a BDD, a couple of datatypes is needed.
//! This module consists of all internally and externally used datatypes, such as
//! [Term], [Var], and [BddNode]
use serde::{Deserialize, Serialize};
use std::{fmt::Display, ops::Deref};

use crate::adfbiodivine::AdfOperations;

/// Representation of a Term
/// Each Term is represented in a number ([usize]) and relates to a
/// Node in the decision diagram
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct Term(pub usize);

impl Deref for Term {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<usize> for Term {
    fn from(val: usize) -> Self {
        Self(val)
    }
}

impl From<&biodivine_lib_bdd::Bdd> for Term {
    fn from(val: &biodivine_lib_bdd::Bdd) -> Self {
        if val.is_true() {
            Term::TOP
        } else if val.is_false() {
            Term::BOT
        } else {
            Term::UND
        }
    }
}

impl Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Term({})", self.0)
    }
}

impl Term {
    /// Represents the truth-value bottom, i.e. false
    pub const BOT: Term = Term(0);
    /// Represents the truth-value top, i.e. true
    pub const TOP: Term = Term(1);
    /// Represents the truth-value undecided, i.e. sat, but not valid
    pub const UND: Term = Term(2);

    /// Get the value of the Term, i.e. the corresponding [usize]
    pub fn value(self) -> usize {
        self.0
    }

    /// Checks if the [Term] represents a truth-value ([Term::TOP] or [Term::BOT]), or
    /// another compound formula.
    pub fn is_truth_value(&self) -> bool {
        self.0 <= Term::TOP.0
    }

    /// Returns true, if the Term is true, i.e. [Term::TOP]
    pub fn is_true(&self) -> bool {
        *self == Self::TOP
    }

    /// Returns true, if the Terms have the same information-value
    pub fn compare_inf(&self, other: &Self) -> bool {
        self.is_truth_value() == other.is_truth_value() && self.is_true() == other.is_true()
    }

    /// Returns true, if the Term and the BDD have the same information-value
    pub fn cmp_information(&self, other: &biodivine_lib_bdd::Bdd) -> bool {
        self.is_truth_value() == other.is_truth_value() && self.is_true() == other.is_true()
    }
}

/// Representation of Variables
/// Note that the algorithm only uses [usize] values to identify variables.
/// The order of these values will be defining for the Variable order of the decision diagram.
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub struct Var(pub usize);

impl Deref for Var {
    type Target = usize;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<usize> for Var {
    fn from(val: usize) -> Self {
        Self(val)
    }
}

impl Display for Var {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Var({})", self.0)
    }
}

impl Var {
    /// Represents the constant symbol "Top"
    pub const TOP: Var = Var(usize::MAX);
    /// Represents the constant symbol "Bot"
    pub const BOT: Var = Var(usize::MAX - 1);

    /// Returns the value of the [Var] as [usize]
    pub fn value(self) -> usize {
        self.0
    }

    /// Returns true if the value of the variable is a constant (i.e. Top or Bot)
    pub fn is_constant(&self) -> bool {
        self.value() >= Var::BOT.value()
    }
}

/// A [BddNode] is representing one Node in the decision diagram
///
/// Intuitively this is a binary tree structure, where the diagram is allowed to
/// pool same values to the same Node.
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct BddNode {
    var: Var,
    lo: Term,
    hi: Term,
}

impl Display for BddNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BddNode: {}, lo: {}, hi: {}", self.var, self.lo, self.hi)
    }
}

impl BddNode {
    /// Creates a new Node
    pub fn new(var: Var, lo: Term, hi: Term) -> Self {
        Self { var, lo, hi }
    }

    /// Returns the current Variable-value
    pub fn var(self) -> Var {
        self.var
    }

    /// Returns the `lo`-branch
    pub fn lo(self) -> Term {
        self.lo
    }

    /// Returns the `hi`-branch
    pub fn hi(self) -> Term {
        self.hi
    }

    /// Creates a node, which represents the `Bot`-truth value
    pub fn bot_node() -> Self {
        Self {
            var: Var::BOT,
            lo: Term::BOT,
            hi: Term::BOT,
        }
    }

    /// Creates a node, which represents the `Top`-truth value
    pub fn top_node() -> Self {
        Self {
            var: Var::TOP,
            lo: Term::TOP,
            hi: Term::TOP,
        }
    }
}

/// Type alias for the pair of counter-models and models
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct ModelCounts {
    /// Contains the number of counter-models
    pub cmodels: usize,
    /// Contains the number of models
    pub models: usize,
}

impl ModelCounts {
    /// Represents the top-node model-counts
    pub fn top() -> ModelCounts {
        (0, 1).into()
    }

    /// Represents the bot-node model-counts
    pub fn bot() -> ModelCounts {
        (1, 0).into()
    }
}

impl From<(usize, usize)> for ModelCounts {
    fn from(tuple: (usize, usize)) -> Self {
        ModelCounts {
            cmodels: tuple.0,
            models: tuple.1,
        }
    }
}
/// Type alias for the Modelcounts and the depth of a given Node in a BDD
pub type CountNode = (ModelCounts, usize);
/// Type alias for Facet counts, which contains number of facets and counter facets.
pub type FacetCounts = (usize, usize);

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck_macros::quickcheck;
    use test_log::test;

    #[test]
    fn cmp() {
        assert!(!Term::BOT.compare_inf(&Term::TOP));
        assert!(!Term::TOP.compare_inf(&Term::BOT));
        assert!(!Term::TOP.compare_inf(&Term(22)));
        assert!(!Term(22).compare_inf(&Term::BOT));
        assert!(Term(22).compare_inf(&Term(12323)));
        assert!(Term::TOP.compare_inf(&Term::TOP));
        assert!(Term::BOT.compare_inf(&Term::BOT));
        assert!(Term(22).compare_inf(&Term(22)));
    }

    #[quickcheck]
    fn deref_display_from(value: usize) -> bool {
        // from
        let term: Term = Term::from(value);
        let var = Var::from(value);
        // display
        assert_eq!(format!("{}", term), format!("Term({})", value));
        assert_eq!(format!("{}", var), format!("Var({})", value));
        //deref
        assert_eq!(value, *term);
        true
    }

    #[quickcheck]
    fn bdd_node(var: usize, lo: usize, hi: usize) -> bool {
        let node = BddNode::new(Var::from(var), Term::from(lo), Term::from(hi));
        assert_eq!(*node.var(), var);
        assert_eq!(*node.lo(), lo);
        assert_eq!(*node.hi(), hi);
        match node.var() {
            Var::TOP => {
                assert!(node.var().is_constant());
            }
            Var::BOT => {
                assert!(node.var().is_constant());
            }
            val => {
                assert!(!val.is_constant());
            }
        }
        true
    }
}
