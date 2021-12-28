//! To represent a BDD, a couple of datatypes is needed.
//! This module consists of all internally and externally used datatypes, such as
//! [Term], [Var], and [BddNode]
use std::{fmt::Display, ops::Deref};

/// Representation of a Term
/// Each Term is represented in a number ([usize]) and relates to a
/// Node in the decision diagram
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Copy, Clone)]
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
}

/// Representation of Variables
/// Note that the algorithm only uses [usize] values to identify variables.
/// The order of these values will be defining for the Variable order of the decision diagram.
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Copy)]
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
}

/// A [BddNode] is representing one Node in the decision diagram
///
/// Intuitively this is a binary tree structure, where the diagram is allowed to
/// pool same values to the same Node.
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Hash, Clone, Copy)]
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