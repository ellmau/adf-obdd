use std::{fmt::Display, ops::Deref};

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
    pub const BOT: Term = Term(0);
    pub const TOP: Term = Term(1);

    pub fn new(val: usize) -> Term {
        Term(val)
    }

    pub fn value(self) -> usize {
        self.0
    }

    pub fn is_truth_value(&self) -> bool {
        self.0 <= Term::TOP.0
    }

    pub fn is_true(&self) -> bool {
        *self == Self::TOP
    }
}

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
    pub const TOP: Var = Var(usize::MAX);
    pub const BOT: Var = Var(usize::MAX - 1);

    pub fn value(self) -> usize {
        self.0
    }
}

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
    pub fn new(var: Var, lo: Term, hi: Term) -> Self {
        Self { var, lo, hi }
    }

    pub fn var(self) -> Var {
        self.var
    }

    pub fn lo(self) -> Term {
        self.lo
    }

    pub fn hi(self) -> Term {
        self.hi
    }

    pub fn bot_node() -> Self {
        Self {
            var: Var::BOT,
            lo: Term::BOT,
            hi: Term::BOT,
        }
    }

    pub fn top_node() -> Self {
        Self {
            var: Var::TOP,
            lo: Term::TOP,
            hi: Term::TOP,
        }
    }
}
