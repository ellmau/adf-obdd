//! Represents an obdd
use crate::datatypes::*;
use serde::{Deserialize, Serialize};
use std::{cmp::min, collections::HashMap, fmt::Display};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Bdd {
    pub(crate) nodes: Vec<BddNode>,
    #[serde(with = "vectorize")]
    cache: HashMap<BddNode, Term>,
}

impl Display for Bdd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, " ")?;
        for (idx, elem) in self.nodes.iter().enumerate() {
            writeln!(f, "{}   {}", idx, *elem)?;
        }
        Ok(())
    }
}

impl Bdd {
    pub fn new() -> Self {
        Self {
            nodes: vec![BddNode::bot_node(), BddNode::top_node()],
            cache: HashMap::new(),
        }
    }

    pub fn variable(&mut self, var: Var) -> Term {
        self.node(var, Term::BOT, Term::TOP)
    }

    pub fn constant(val: bool) -> Term {
        if val {
            Term::TOP
        } else {
            Term::BOT
        }
    }

    pub fn not(&mut self, term: Term) -> Term {
        self.if_then_else(term, Term::BOT, Term::TOP)
    }

    pub fn and(&mut self, term_a: Term, term_b: Term) -> Term {
        self.if_then_else(term_a, term_b, Term::BOT)
    }

    pub fn or(&mut self, term_a: Term, term_b: Term) -> Term {
        self.if_then_else(term_a, Term::TOP, term_b)
    }

    pub fn imp(&mut self, term_a: Term, term_b: Term) -> Term {
        self.if_then_else(term_a, term_b, Term::TOP)
    }

    pub fn iff(&mut self, term_a: Term, term_b: Term) -> Term {
        let not_b = self.not(term_b);
        self.if_then_else(term_a, term_b, not_b)
    }

    pub fn xor(&mut self, term_a: Term, term_b: Term) -> Term {
        let not_b = self.not(term_b);
        self.if_then_else(term_a, not_b, term_b)
    }

    pub fn restrict(&mut self, tree: Term, var: Var, val: bool) -> Term {
        let node = self.nodes[tree.0];
        #[allow(clippy::collapsible_else_if)]
        // Readability of algorithm > code-elegance
        if node.var() > var || node.var() >= Var::BOT {
            tree
        } else if node.var() < var {
            let lonode = self.restrict(node.lo(), var, val);
            let hinode = self.restrict(node.hi(), var, val);
            self.node(node.var(), lonode, hinode)
        } else {
            if val {
                self.restrict(node.hi(), var, val)
            } else {
                self.restrict(node.lo(), var, val)
            }
        }
    }

    fn if_then_else(&mut self, i: Term, t: Term, e: Term) -> Term {
        if i == Term::TOP {
            t
        } else if i == Term::BOT {
            e
        } else if t == e {
            t
        } else if t == Term::TOP && e == Term::BOT {
            i
        } else {
            let minvar = Var(min(
                self.nodes[i.value()].var().value(),
                min(
                    self.nodes[t.value()].var().value(),
                    self.nodes[e.value()].var().value(),
                ),
            ));
            let itop = self.restrict(i, minvar, true);
            let ttop = self.restrict(t, minvar, true);
            let etop = self.restrict(e, minvar, true);
            let ibot = self.restrict(i, minvar, false);
            let tbot = self.restrict(t, minvar, false);
            let ebot = self.restrict(e, minvar, false);

            let top_ite = self.if_then_else(itop, ttop, etop);
            let bot_ite = self.if_then_else(ibot, tbot, ebot);
            self.node(minvar, bot_ite, top_ite)
        }
    }
    fn node(&mut self, var: Var, lo: Term, hi: Term) -> Term {
        if lo == hi {
            lo
        } else {
            let node = BddNode::new(var, lo, hi);
            match self.cache.get(&node) {
                Some(t) => *t,
                None => {
                    let new_term = Term(self.nodes.len());
                    self.nodes.push(node);
                    self.cache.insert(node, new_term);
                    new_term
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn newbdd() {
        let bdd = Bdd::new();
        assert_eq!(bdd.nodes.len(), 2);
    }

    #[test]
    fn addconst() {
        let bdd = Bdd::new();

        assert_eq!(Bdd::constant(true), Term::TOP);
        assert_eq!(Bdd::constant(false), Term::BOT);
        assert_eq!(bdd.nodes.len(), 2);
    }

    #[test]
    fn addvar() {
        let mut bdd = Bdd::new();
        assert_eq!(bdd.variable(Var(0)), Term(2));
        assert_eq!(bdd.variable(Var(1)), Term(3));
        assert_eq!(Var(1), Var(1));
        bdd.variable(Var(0));
        assert_eq!(bdd.variable(Var(0)), Term(2));
    }

    #[test]
    fn use_negation() {
        let mut bdd = Bdd::new();
        let v1 = bdd.variable(Var(0));
        assert_eq!(v1, 2.into());
        assert_eq!(bdd.not(v1), Term(3));
    }

    #[test]
    fn use_add() {
        let mut bdd = Bdd::new();
        let v1 = bdd.variable(Var(0));
        let v2 = bdd.variable(Var(1));
        let v3 = bdd.variable(Var(2));

        let a1 = bdd.and(v1, v2);
        let a2 = bdd.and(a1, v3);
        assert_eq!(a2, Term(7));
    }

    #[test]
    fn use_or() {
        let mut bdd = Bdd::new();

        let v1 = bdd.variable(Var(0));
        let v2 = bdd.variable(Var(1));
        let v3 = bdd.variable(Var(2));

        let a1 = bdd.and(v1, v2);
        let a2 = bdd.or(a1, v3);
        assert_eq!(a2, Term(7));
    }

    #[test]
    fn produce_different_conversions() {
        let mut bdd = Bdd::new();

        let v1 = bdd.variable(Var(0));
        let v2 = bdd.variable(Var(1));

        assert_eq!(v1, Term(2));
        assert_eq!(v2, Term(3));

        let t1 = bdd.and(v1, v2);
        let nt1 = bdd.not(t1);
        let ft = bdd.or(v1, nt1);

        assert_eq!(ft, Term::TOP);

        let v3 = bdd.variable(Var(2));
        let nv3 = bdd.not(v3);
        assert_eq!(bdd.and(v3, nv3), Term::BOT);

        let conj = bdd.and(v1, v2);
        assert_eq!(bdd.restrict(conj, Var(0), false), Term::BOT);
        assert_eq!(bdd.restrict(conj, Var(0), true), v2);

        let a = bdd.and(v3, v2);
        let b = bdd.or(v2, v1);

        let con1 = bdd.and(a, conj);

        let end = bdd.or(con1, b);
        let x = bdd.restrict(end, Var(1), false);
        assert_eq!(x, Term(2));
    }
}

/// vectorize maps with non-standard keys
pub mod vectorize {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::iter::FromIterator;

    /// Serialise into a Vector from a Map
    pub fn serialize<'a, T, K, V, S>(target: T, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: IntoIterator<Item = (&'a K, &'a V)>,
        K: Serialize + 'a,
        V: Serialize + 'a,
    {
        let container: Vec<_> = target.into_iter().collect();
        serde::Serialize::serialize(&container, ser)
    }

    /// Deserialize from a Vector to a Map
    pub fn deserialize<'de, T, K, V, D>(des: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromIterator<(K, V)>,
        K: Deserialize<'de>,
        V: Deserialize<'de>,
    {
        let container: Vec<_> = serde::Deserialize::deserialize(des)?;
        Ok(T::from_iter(container.into_iter()))
    }
}
