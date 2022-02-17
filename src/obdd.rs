//! Represents an obdd
pub mod vectorize;
use crate::datatypes::*;
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, cmp::min, collections::HashMap, fmt::Display};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Bdd {
    pub(crate) nodes: Vec<BddNode>,
    #[serde(with = "vectorize")]
    cache: HashMap<BddNode, Term>,
    #[serde(skip, default = "Bdd::default_count_cache")]
    count_cache: RefCell<HashMap<Term, CountNode>>,
}

impl Display for Bdd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, " ")?;
        for (idx, elem) in self.nodes.iter().enumerate() {
            writeln!(f, "{} {}", idx, *elem)?;
        }
        Ok(())
    }
}

impl Bdd {
    pub fn new() -> Self {
        #[cfg(not(feature = "adhoccounting"))]
        {
            Self {
                nodes: vec![BddNode::bot_node(), BddNode::top_node()],
                cache: HashMap::new(),
                count_cache: RefCell::new(HashMap::new()),
            }
        }
        #[cfg(feature = "adhoccounting")]
        {
            let result = Self {
                nodes: vec![BddNode::bot_node(), BddNode::top_node()],
                cache: HashMap::new(),
                count_cache: RefCell::new(HashMap::new()),
            };
            result
                .count_cache
                .borrow_mut()
                .insert(Term::TOP, ((0, 1), 0));
            result
                .count_cache
                .borrow_mut()
                .insert(Term::BOT, ((1, 0), 0));
            result
        }
    }

    fn default_count_cache() -> RefCell<HashMap<Term, CountNode>> {
        RefCell::new(HashMap::new())
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
    pub fn node(&mut self, var: Var, lo: Term, hi: Term) -> Term {
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
                    #[cfg(feature = "adhoccounting")]
                    {
                        log::debug!("newterm: {} as {:?}", new_term, node);
                        let mut count_cache = self.count_cache.borrow_mut();
                        let ((lo_cmodel, lo_model), lodepth) =
                            *count_cache.get(&lo).expect("Cache corrupted");
                        let ((hi_cmodel, hi_model), hidepth) =
                            *count_cache.get(&hi).expect("Cache corrupted");
                        log::debug!("lo (cm: {}, mo: {}, dp: {})", lo_cmodel, lo_model, lodepth);
                        log::debug!("hi (cm: {}, mo: {}, dp: {})", hi_cmodel, hi_model, hidepth);
                        let (lo_exp, hi_exp) = if lodepth > hidepth {
                            (1, 2usize.pow((lodepth - hidepth) as u32))
                        } else {
                            (2usize.pow((hidepth - lodepth) as u32), 1)
                        };
                        log::debug!("lo_exp {}, hi_exp {}", lo_exp, hi_exp);
                        count_cache.insert(
                            new_term,
                            (
                                (
                                    lo_cmodel * lo_exp + hi_cmodel * hi_exp,
                                    lo_model * lo_exp + hi_model * hi_exp,
                                ),
                                std::cmp::max(lodepth, hidepth) + 1,
                            ),
                        );
                    }
                    new_term
                }
            }
        }
    }

    /// Computes the number of counter-models and models for a given BDD-tree
    pub fn models(&self, term: Term, _memoization: bool) -> ModelCounts {
        #[cfg(feature = "adhoccounting")]
        {
            return self.count_cache.borrow().get(&term).unwrap().0;
        }
        #[cfg(not(feature = "adhoccounting"))]
        if _memoization {
            self.modelcount_memoization(term).0
        } else {
            self.modelcount_naive(term).0
        }
    }

    #[allow(dead_code)] // dead code due to more efficient ad-hoc building, still used for a couple of tests
    /// Computes the number of counter-models, models, and variables for a given BDD-tree
    fn modelcount_naive(&self, term: Term) -> CountNode {
        if term == Term::TOP {
            ((0, 1), 0)
        } else if term == Term::BOT {
            ((1, 0), 0)
        } else {
            let node = &self.nodes[term.0];
            let mut lo_exp = 0u32;
            let mut hi_exp = 0u32;
            let ((lo_counter, lo_model), lodepth) = self.modelcount_naive(node.lo());
            let ((hi_counter, hi_model), hidepth) = self.modelcount_naive(node.hi());
            if lodepth > hidepth {
                hi_exp = (lodepth - hidepth) as u32;
            } else {
                lo_exp = (hidepth - lodepth) as u32;
            }
            (
                (
                    lo_counter * 2usize.pow(lo_exp) + hi_counter * 2usize.pow(hi_exp),
                    lo_model * 2usize.pow(lo_exp) + hi_model * 2usize.pow(hi_exp),
                ),
                std::cmp::max(lodepth, hidepth) + 1,
            )
        }
    }

    fn modelcount_memoization(&self, term: Term) -> CountNode {
        if term == Term::TOP {
            ((0, 1), 0)
        } else if term == Term::BOT {
            ((1, 0), 0)
        } else {
            if let Some(result) = self.count_cache.borrow().get(&term) {
                return *result;
            }
            let result = {
                let node = &self.nodes[term.0];
                let mut lo_exp = 0u32;
                let mut hi_exp = 0u32;
                let ((lo_counter, lo_model), lodepth) = self.modelcount_memoization(node.lo());
                let ((hi_counter, hi_model), hidepth) = self.modelcount_memoization(node.hi());
                if lodepth > hidepth {
                    hi_exp = (lodepth - hidepth) as u32;
                } else {
                    lo_exp = (hidepth - lodepth) as u32;
                }
                (
                    (
                        lo_counter * 2usize.pow(lo_exp) + hi_counter * 2usize.pow(hi_exp),
                        lo_model * 2usize.pow(lo_exp) + hi_model * 2usize.pow(hi_exp),
                    ),
                    std::cmp::max(lodepth, hidepth) + 1,
                )
            };
            self.count_cache.borrow_mut().insert(term, result);
            result
        }
    }

    #[cfg(feature = "adhoccounting")]
    pub fn fix_import(&self) {
        self.count_cache.borrow_mut().insert(Term::TOP, ((0, 1), 0));
        self.count_cache.borrow_mut().insert(Term::BOT, ((1, 0), 0));
        for i in 0..self.nodes.len() {
            log::debug!("fixing Term({})", i);
            self.modelcount_memoization(Term(i));
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

    #[test]
    fn display() {
        let mut bdd = Bdd::new();

        let v1 = bdd.variable(Var(0));
        let v2 = bdd.variable(Var(1));
        let v3 = bdd.variable(Var(2));

        let a1 = bdd.and(v1, v2);
        let _a2 = bdd.or(a1, v3);

        assert_eq!(format!("{}", bdd), " \n0 BddNode: Var(18446744073709551614), lo: Term(0), hi: Term(0)\n1 BddNode: Var(18446744073709551615), lo: Term(1), hi: Term(1)\n2 BddNode: Var(0), lo: Term(0), hi: Term(1)\n3 BddNode: Var(1), lo: Term(0), hi: Term(1)\n4 BddNode: Var(2), lo: Term(0), hi: Term(1)\n5 BddNode: Var(0), lo: Term(0), hi: Term(3)\n6 BddNode: Var(1), lo: Term(4), hi: Term(1)\n7 BddNode: Var(0), lo: Term(4), hi: Term(6)\n");
    }

    #[test]
    fn counting() {
        let mut bdd = Bdd::new();

        let v1 = bdd.variable(Var(0));
        let v2 = bdd.variable(Var(1));
        let v3 = bdd.variable(Var(2));

        let formula1 = bdd.and(v1, v2);
        let formula2 = bdd.or(v1, v2);
        let formula3 = bdd.xor(v1, v2);
        let formula4 = bdd.and(v3, formula2);

        assert_eq!(bdd.models(v1, false), (1, 1));
        let mut x = bdd.count_cache.get_mut().iter().collect::<Vec<_>>();
        x.sort();
        log::debug!("{:?}", formula1);
        for x in bdd.nodes.iter().enumerate() {
            log::debug!("{:?}", x);
        }
        log::debug!("{:?}", x);
        assert_eq!(bdd.models(formula1, false), (3, 1));
        assert_eq!(bdd.models(formula2, false), (1, 3));
        assert_eq!(bdd.models(formula3, false), (2, 2));
        assert_eq!(bdd.models(formula4, false), (5, 3));
        assert_eq!(bdd.models(Term::TOP, false), (0, 1));
        assert_eq!(bdd.models(Term::BOT, false), (1, 0));

        assert_eq!(bdd.models(v1, true), (1, 1));
        assert_eq!(bdd.models(formula1, true), (3, 1));
        assert_eq!(bdd.models(formula2, true), (1, 3));
        assert_eq!(bdd.models(formula3, true), (2, 2));
        assert_eq!(bdd.models(formula4, true), (5, 3));
        assert_eq!(bdd.models(Term::TOP, true), (0, 1));
        assert_eq!(bdd.models(Term::BOT, true), (1, 0));

        assert_eq!(bdd.modelcount_naive(v1), ((1, 1), 1));
        assert_eq!(bdd.modelcount_naive(formula1), ((3, 1), 2));
        assert_eq!(bdd.modelcount_naive(formula2), ((1, 3), 2));
        assert_eq!(bdd.modelcount_naive(formula3), ((2, 2), 2));
        assert_eq!(bdd.modelcount_naive(formula4), ((5, 3), 3));
        assert_eq!(bdd.modelcount_naive(Term::TOP), ((0, 1), 0));
        assert_eq!(bdd.modelcount_naive(Term::BOT), ((1, 0), 0));

        assert_eq!(
            bdd.modelcount_naive(formula4),
            bdd.modelcount_memoization(formula4)
        );

        assert_eq!(bdd.modelcount_naive(v1), bdd.modelcount_memoization(v1));
        assert_eq!(
            bdd.modelcount_naive(formula1),
            bdd.modelcount_memoization(formula1)
        );
        assert_eq!(
            bdd.modelcount_naive(formula2),
            bdd.modelcount_memoization(formula2)
        );
        assert_eq!(
            bdd.modelcount_naive(formula3),
            bdd.modelcount_memoization(formula3)
        );
        assert_eq!(
            bdd.modelcount_naive(Term::TOP),
            bdd.modelcount_memoization(Term::TOP)
        );
        assert_eq!(
            bdd.modelcount_naive(Term::BOT),
            bdd.modelcount_memoization(Term::BOT)
        );
    }
}
