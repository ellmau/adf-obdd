//! Module which represents obdds.
//!
pub mod vectorize;
use crate::datatypes::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::{cell::RefCell, cmp::min, collections::HashMap, fmt::Display};

/// Contains the data of (possibly) multiple roBDDs, managed over one collection of nodes.
/// It has a couple of methods to instantiate, update, and query properties on a given roBDD.
/// Each roBDD is identified by its corresponding [`Term`], which implicitly identifies the root node of a roBDD.
#[derive(Debug, Serialize, Deserialize)]
pub struct Bdd {
    pub(crate) nodes: Vec<BddNode>,
    #[cfg(feature = "variablelist")]
    #[serde(skip)]
    var_deps: Vec<HashSet<Var>>,
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

impl Default for Bdd {
    fn default() -> Self {
        Self::new()
    }
}

impl Bdd {
    /// Instantiate a new roBDD structures.
    /// Constants for the [`⊤`][crate::datatypes::Term::TOP] and [`⊥`][crate::datatypes::Term::BOT] concepts are prepared in that step too.
    pub fn new() -> Self {
        #[cfg(not(feature = "adhoccounting"))]
        {
            Self {
                nodes: vec![BddNode::bot_node(), BddNode::top_node()],
                #[cfg(feature = "variablelist")]
                var_deps: vec![HashSet::new(), HashSet::new()],
                cache: HashMap::new(),
                count_cache: RefCell::new(HashMap::new()),
            }
        }
        #[cfg(feature = "adhoccounting")]
        {
            let result = Self {
                nodes: vec![BddNode::bot_node(), BddNode::top_node()],
                #[cfg(feature = "variablelist")]
                var_deps: vec![HashSet::new(), HashSet::new()],
                cache: HashMap::new(),
                count_cache: RefCell::new(HashMap::new()),
            };
            result
                .count_cache
                .borrow_mut()
                .insert(Term::TOP, (ModelCounts::top(), ModelCounts::top(), 0));
            result
                .count_cache
                .borrow_mut()
                .insert(Term::BOT, (ModelCounts::bot(), ModelCounts::bot(), 0));
            result
        }
    }

    fn default_count_cache() -> RefCell<HashMap<Term, CountNode>> {
        RefCell::new(HashMap::new())
    }

    /// Instantiates a [variable][crate::datatypes::Var] and returns the representing roBDD as a [`Term`][crate::datatypes::Term].
    pub fn variable(&mut self, var: Var) -> Term {
        self.node(var, Term::BOT, Term::TOP)
    }

    /// Instantiates a constant, which is either [true] or [false].
    pub fn constant(val: bool) -> Term {
        if val {
            Term::TOP
        } else {
            Term::BOT
        }
    }

    /// Returns an roBDD, which represents the negation of the given roBDD.
    pub fn not(&mut self, term: Term) -> Term {
        self.if_then_else(term, Term::BOT, Term::TOP)
    }

    /// Returns an roBDD, which represents the conjunction of the two given roBDDs.
    pub fn and(&mut self, term_a: Term, term_b: Term) -> Term {
        self.if_then_else(term_a, term_b, Term::BOT)
    }

    /// Returns an roBDD, which represents the disjunction of the two given roBDDs.
    pub fn or(&mut self, term_a: Term, term_b: Term) -> Term {
        self.if_then_else(term_a, Term::TOP, term_b)
    }

    /// Returns an roBDD, which represents the implication of the two given roBDDs.
    pub fn imp(&mut self, term_a: Term, term_b: Term) -> Term {
        self.if_then_else(term_a, term_b, Term::TOP)
    }

    /// Returns an roBDD, which represents the if and only if relation  of the two given roBDDs.
    pub fn iff(&mut self, term_a: Term, term_b: Term) -> Term {
        let not_b = self.not(term_b);
        self.if_then_else(term_a, term_b, not_b)
    }

    /// Returns an roBDD, which represents the exclusive disjunction of the two given roBDDs.
    pub fn xor(&mut self, term_a: Term, term_b: Term) -> Term {
        let not_b = self.not(term_b);
        self.if_then_else(term_a, not_b, term_b)
    }

    /// Computes the interpretations represented in the roBDD, which are either models or counter-models.
    /// **goal_var** is the [variable][Var] to which the roBDD is related to and it is ensured that the goal is consistent with the respective interpretation.
    /// **goal** is a boolean [variable][Var], which defines whether the models or counter-models are of interest.
    pub fn interpretations(
        &self,
        tree: Term,
        goal: bool,
        goal_var: Var,
        negative: &[Var],
        positive: &[Var],
    ) -> Vec<(Vec<Var>, Vec<Var>)> {
        let mut result = Vec::new();
        let node = self.nodes[tree.value()];
        let var = node.var();
        if tree.is_truth_value() {
            return Vec::new();
        }
        // if the current var is the goal var, then only work with the hi-node if the goal is true
        if (goal_var != var) || goal {
            if node.hi().is_truth_value() {
                if node.hi().is_true() == goal {
                    result.push((negative.to_vec(), [positive, &[var]].concat()));
                }
            } else {
                result.append(&mut self.interpretations(
                    node.hi(),
                    goal,
                    goal_var,
                    negative,
                    &[positive, &[var]].concat(),
                ));
            }
        }
        // if the current var is the goal var, then only work with the lo-node if the goal is false
        if (goal_var != var) || !goal {
            if node.lo().is_truth_value() {
                if node.lo().is_true() == goal {
                    result.push(([negative, &[var]].concat(), positive.to_vec()));
                }
            } else {
                result.append(&mut self.interpretations(
                    node.lo(),
                    goal,
                    goal_var,
                    &[negative, &[var]].concat(),
                    positive,
                ));
            }
        }
        result
    }

    /// Restrict the value of a given [variable][crate::datatypes::Var] to **val**.
    pub fn restrict(&mut self, tree: Term, var: Var, val: bool) -> Term {
        let node = self.nodes[tree.0];
        #[cfg(feature = "variablelist")]
        {
            if !self.var_deps[tree.value()].contains(&var) {
                return tree;
            }
        }
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

    /// Creates an roBDD, based on the relation of three roBDDs, which are in an `if-then-else` relation.
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

    /// Creates a new node in the roBDD.
    /// It will not create duplicate nodes and uses already existing nodes, if applicable.
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
                    #[cfg(feature = "variablelist")]
                    {
                        let mut var_set: HashSet<Var> = self.var_deps[lo.value()]
                            .union(&self.var_deps[hi.value()])
                            .copied()
                            .collect();
                        var_set.insert(var);
                        self.var_deps.push(var_set);
                    }
                    log::debug!("newterm: {} as {:?}", new_term, node);
                    #[cfg(feature = "adhoccounting")]
                    {
                        let mut count_cache = self.count_cache.borrow_mut();
                        let (lo_counts, lo_paths, lodepth) =
                            *count_cache.get(&lo).expect("Cache corrupted");
                        let (hi_counts, hi_paths, hidepth) =
                            *count_cache.get(&hi).expect("Cache corrupted");
                        log::debug!(
                            "lo (cm: {}, mo: {}, p-: {}, p+: {}, dp: {})",
                            lo_counts.cmodels,
                            lo_counts.models,
                            lo_paths.cmodels,
                            lo_paths.models,
                            lodepth
                        );
                        log::debug!(
                            "hi (cm: {}, mo: {}, p-: {}, p+: {}, dp: {})",
                            hi_counts.cmodels,
                            hi_counts.models,
                            hi_paths.cmodels,
                            hi_paths.models,
                            hidepth
                        );
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
                                    lo_counts.cmodels * lo_exp + hi_counts.cmodels * hi_exp,
                                    lo_counts.models * lo_exp + hi_counts.models * hi_exp,
                                )
                                    .into(),
                                (
                                    lo_paths.cmodels + hi_paths.cmodels,
                                    lo_paths.models + hi_paths.models,
                                )
                                    .into(),
                                std::cmp::max(lodepth, hidepth) + 1,
                            ),
                        );
                    }
                    new_term
                }
            }
        }
    }

    /// Computes the number of counter-models and models for a given roBDD.
    ///
    /// Use the flag `_memoization` to choose between using the memoization approach or not. (This flag does nothing, if the feature `adhoccounting` is used)
    pub fn models(&self, term: Term, _memoization: bool) -> ModelCounts {
        #[cfg(feature = "adhoccounting")]
        {
            return self.count_cache.borrow().get(&term).expect("The term should be originating from this bdd, otherwise the result would be inconsistent anyways").0;
        }
        #[cfg(not(feature = "adhoccounting"))]
        if _memoization {
            self.modelcount_memoization(term).0
        } else {
            self.modelcount_naive(term).0
        }
    }

    /// Computes the number of paths, which lead to ⊥ respectively ⊤.
    ///
    /// Use the flag `_memoization` to choose between using the memoization approach or not. (This flag does nothing, if the feature `adhoccounting` is used)
    pub fn paths(&self, term: Term, _memoization: bool) -> ModelCounts {
        #[cfg(feature = "adhoccounting")]
        {
            return self.count_cache.borrow().get(&term).expect("The term should be originating from this bdd, otherwise the result would be inconsistent anyways").1;
        }
        #[cfg(not(feature = "adhoccounting"))]
        if _memoization {
            self.modelcount_memoization(term).1
        } else {
            self.modelcount_naive(term).1
        }
    }

    /// Computes the maximal depth of the given sub-diagram.
    ///
    /// Intuitively this will compute the longest possible path from **term** to a leaf-node (i.e., ⊥ or ⊤).
    #[allow(dead_code)] // max depth may be used in future heuristics
    pub fn max_depth(&self, term: Term) -> usize {
        #[cfg(feature = "adhoccounting")]
        {
            return self.count_cache.borrow().get(&term).expect("The term should be originating from this bdd, otherwise the result would be inconsistent anyways").2;
        }
        #[cfg(not(feature = "adhoccounting"))]
        match self.count_cache.borrow().get(&term) {
            Some((_mc, _pc, depth)) => *depth,
            None => {
                if term.is_truth_value() {
                    0
                } else {
                    self.max_depth(self.nodes[term.0].hi())
                        .max(self.max_depth(self.nodes[term.0].lo()))
                }
            }
        }
    }

    #[allow(dead_code)] // dead code due to more efficient ad-hoc building, still used for a couple of tests
    /// Computes the number of counter-models, models, and variables for a given roBDD
    fn modelcount_naive(&self, term: Term) -> CountNode {
        if term == Term::TOP {
            (ModelCounts::top(), ModelCounts::top(), 0)
        } else if term == Term::BOT {
            (ModelCounts::bot(), ModelCounts::bot(), 0)
        } else {
            let node = &self.nodes[term.0];
            let mut lo_exp = 0u32;
            let mut hi_exp = 0u32;
            let (lo_counts, lo_paths, lodepth) = self.modelcount_naive(node.lo());
            let (hi_counts, hi_paths, hidepth) = self.modelcount_naive(node.hi());
            if lodepth > hidepth {
                hi_exp = (lodepth - hidepth) as u32;
            } else {
                lo_exp = (hidepth - lodepth) as u32;
            }
            (
                (
                    lo_counts.cmodels * 2usize.pow(lo_exp) + hi_counts.cmodels * 2usize.pow(hi_exp),
                    lo_counts.models * 2usize.pow(lo_exp) + hi_counts.models * 2usize.pow(hi_exp),
                )
                    .into(),
                (
                    lo_paths.cmodels + hi_paths.cmodels,
                    lo_paths.models + hi_paths.models,
                )
                    .into(),
                std::cmp::max(lodepth, hidepth) + 1,
            )
        }
    }

    fn modelcount_memoization(&self, term: Term) -> CountNode {
        if term == Term::TOP {
            (ModelCounts::top(), ModelCounts::top(), 0)
        } else if term == Term::BOT {
            (ModelCounts::bot(), ModelCounts::bot(), 0)
        } else {
            if let Some(result) = self.count_cache.borrow().get(&term) {
                return *result;
            }
            let result = {
                let node = &self.nodes[term.0];
                let mut lo_exp = 0u32;
                let mut hi_exp = 0u32;
                let (lo_counts, lo_paths, lodepth) = self.modelcount_memoization(node.lo());
                let (hi_counts, hi_paths, hidepth) = self.modelcount_memoization(node.hi());
                if lodepth > hidepth {
                    hi_exp = (lodepth - hidepth) as u32;
                } else {
                    lo_exp = (hidepth - lodepth) as u32;
                }
                (
                    (
                        lo_counts.cmodels * 2usize.pow(lo_exp)
                            + hi_counts.cmodels * 2usize.pow(hi_exp),
                        lo_counts.models * 2usize.pow(lo_exp)
                            + hi_counts.models * 2usize.pow(hi_exp),
                    )
                        .into(),
                    (
                        lo_paths.cmodels + hi_paths.cmodels,
                        lo_paths.models + hi_paths.models,
                    )
                        .into(),
                    std::cmp::max(lodepth, hidepth) + 1,
                )
            };
            self.count_cache.borrow_mut().insert(term, result);
            result
        }
    }

    /// Repairs the internal structures after an import.
    pub fn fix_import(&mut self) {
        self.generate_var_dependencies();
        #[cfg(feature = "adhoccounting")]
        {
            self.count_cache
                .borrow_mut()
                .insert(Term::TOP, (ModelCounts::top(), ModelCounts::top(), 0));
            self.count_cache
                .borrow_mut()
                .insert(Term::BOT, (ModelCounts::bot(), ModelCounts::bot(), 0));
            for i in 0..self.nodes.len() {
                log::debug!("fixing Term({})", i);
                self.modelcount_memoization(Term(i));
            }
        }
    }

    fn generate_var_dependencies(&mut self) {
        #[cfg(feature = "variablelist")]
        self.nodes.iter().for_each(|node| {
            if node.var() >= Var::BOT {
                self.var_deps.push(HashSet::new());
            } else {
                let mut var_set: HashSet<Var> = self.var_deps[node.lo().value()]
                    .union(&self.var_deps[node.hi().value()])
                    .copied()
                    .collect();
                var_set.insert(node.var());
                self.var_deps.push(var_set);
            }
        });
    }

    /// Returns a [HashSet] of [variables][crate::datatypes::Var], which occur in a given roBDD.
    pub fn var_dependencies(&self, tree: Term) -> HashSet<Var> {
        #[cfg(feature = "variablelist")]
        {
            self.var_deps[tree.value()].clone()
        }
        #[cfg(not(feature = "variablelist"))]
        {
            let node = self.nodes[tree.value()];
            if node.var().is_constant() {
                return HashSet::new();
            }
            let mut var_set = self
                .var_dependencies(node.lo())
                .union(&self.var_dependencies(node.hi()))
                .copied()
                .collect::<HashSet<Var>>();
            var_set.insert(node.var());
            var_set
        }
    }

    /// Returns the variable impact of a [variable][crate::datatypes::Var] with respect to a given set of roBDDs.
    pub fn passive_var_impact(&self, var: Var, termlist: &[Term]) -> usize {
        termlist.iter().fold(0usize, |acc, val| {
            if self.var_dependencies(*val).contains(&var) {
                acc + 1
            } else {
                acc
            }
        })
    }

    /// Counts how often another roBDD uses a [variable][crate::datatypes::Var], which occurs in this roBDD.
    pub fn active_var_impact(&self, var: Var, termlist: &[Term]) -> usize {
        (0..termlist.len()).into_iter().fold(0usize, |acc, idx| {
            if self
                .var_dependencies(termlist[var.value()])
                .contains(&Var(idx))
            {
                acc + 1
            } else {
                acc
            }
        })
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
        log::debug!("Restrict test: restrict({},{},false)", end, Var(1));
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

        assert_eq!(bdd.models(v1, false), (1, 1).into());
        let mut x = bdd.count_cache.get_mut().iter().collect::<Vec<_>>();
        x.sort();
        log::debug!("{:?}", formula1);
        for x in bdd.nodes.iter().enumerate() {
            log::debug!("{:?}", x);
        }
        log::debug!("{:?}", x);
        assert_eq!(bdd.models(formula1, false), (3, 1).into());
        assert_eq!(bdd.models(formula2, false), (1, 3).into());
        assert_eq!(bdd.models(formula3, false), (2, 2).into());
        assert_eq!(bdd.models(formula4, false), (5, 3).into());
        assert_eq!(bdd.models(Term::TOP, false), (0, 1).into());
        assert_eq!(bdd.models(Term::BOT, false), (1, 0).into());

        assert_eq!(bdd.models(v1, true), (1, 1).into());
        assert_eq!(bdd.models(formula1, true), (3, 1).into());
        assert_eq!(bdd.models(formula2, true), (1, 3).into());
        assert_eq!(bdd.models(formula3, true), (2, 2).into());
        assert_eq!(bdd.models(formula4, true), (5, 3).into());
        assert_eq!(bdd.models(Term::TOP, true), (0, 1).into());
        assert_eq!(bdd.models(Term::BOT, true), (1, 0).into());

        assert_eq!(bdd.paths(formula1, false), (2, 1).into());
        assert_eq!(bdd.paths(formula2, false), (1, 2).into());
        assert_eq!(bdd.paths(formula3, false), (2, 2).into());
        assert_eq!(bdd.paths(formula4, false), (3, 2).into());
        assert_eq!(bdd.paths(Term::TOP, false), (0, 1).into());
        assert_eq!(bdd.paths(Term::BOT, false), (1, 0).into());

        assert_eq!(bdd.paths(v1, true), (1, 1).into());
        assert_eq!(bdd.paths(formula1, true), (2, 1).into());
        assert_eq!(bdd.paths(formula2, true), (1, 2).into());
        assert_eq!(bdd.paths(formula3, true), (2, 2).into());
        assert_eq!(bdd.paths(formula4, true), (3, 2).into());
        assert_eq!(bdd.paths(Term::TOP, true), (0, 1).into());
        assert_eq!(bdd.paths(Term::BOT, true), (1, 0).into());

        assert_eq!(bdd.modelcount_naive(v1), ((1, 1).into(), (1, 1).into(), 1));
        assert_eq!(
            bdd.modelcount_naive(formula1),
            ((3, 1).into(), (2, 1).into(), 2)
        );
        assert_eq!(
            bdd.modelcount_naive(formula2),
            ((1, 3).into(), (1, 2).into(), 2)
        );
        assert_eq!(
            bdd.modelcount_naive(formula3),
            ((2, 2).into(), (2, 2).into(), 2)
        );
        assert_eq!(
            bdd.modelcount_naive(formula4),
            ((5, 3).into(), (3, 2).into(), 3)
        );
        assert_eq!(
            bdd.modelcount_naive(Term::TOP),
            ((0, 1).into(), (0, 1).into(), 0)
        );
        assert_eq!(
            bdd.modelcount_naive(Term::BOT),
            ((1, 0).into(), (1, 0).into(), 0)
        );

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

        assert_eq!(bdd.max_depth(Term::BOT), 0);
        assert_eq!(bdd.max_depth(v1), 1);
        assert_eq!(bdd.max_depth(formula3), 2);
        assert_eq!(bdd.max_depth(formula4), 3);
    }

    #[cfg(feature = "variablelist")]
    #[test]
    fn generate_var_dependencies() {
        let mut bdd = Bdd::new();

        let v1 = bdd.variable(Var(0));
        let v2 = bdd.variable(Var(1));
        let v3 = bdd.variable(Var(2));

        let formula1 = bdd.and(v1, v2);
        let formula2 = bdd.or(v1, v2);
        let formula3 = bdd.xor(v1, v2);
        let formula4 = bdd.and(v3, formula2);

        bdd.iff(formula1, formula3);
        bdd.not(formula4);

        let constructed = bdd.var_deps.clone();
        bdd.var_deps = Vec::new();
        bdd.generate_var_dependencies();

        constructed
            .iter()
            .zip(bdd.var_deps.iter())
            .for_each(|(left, right)| {
                assert!(left == right);
            });

        assert_eq!(
            bdd.passive_var_impact(Var(0), &[formula1, formula2, formula3, formula4]),
            4
        );
        assert_eq!(
            bdd.passive_var_impact(Var(2), &[formula1, formula2, formula3, formula4]),
            1
        );
        assert_eq!(
            bdd.passive_var_impact(Var(2), &[formula1, formula2, formula3]),
            0
        );
    }

    #[test]
    fn var_impact() {
        let mut bdd = Bdd::new();
        let v1 = bdd.variable(Var(0));
        let v2 = bdd.variable(Var(1));
        let v3 = bdd.variable(Var(2));

        let formula1 = bdd.and(v1, v2);
        let formula2 = bdd.or(v1, v2);

        let ac: Vec<Term> = vec![formula1, formula2, v3];

        assert_eq!(bdd.passive_var_impact(Var(0), &ac), 2);
        assert_eq!(bdd.passive_var_impact(Var(1), &ac), 2);
        assert_eq!(bdd.passive_var_impact(Var(2), &ac), 1);

        assert_eq!(bdd.active_var_impact(Var(0), &ac), 2);
        assert_eq!(bdd.active_var_impact(Var(2), &ac), 1);
    }

    #[test]
    fn interpretations() {
        let mut bdd = Bdd::new();

        let v1 = bdd.variable(Var(0));
        let v2 = bdd.variable(Var(1));

        let formula1 = bdd.and(v1, v2);
        let formula2 = bdd.xor(v1, v2);

        assert_eq!(
            bdd.interpretations(formula1, true, Var(2), &[], &[]),
            vec![(vec![], vec![Var(0), Var(1)])]
        );

        assert_eq!(
            bdd.interpretations(formula1, true, Var(0), &[], &[]),
            vec![(vec![], vec![Var(0), Var(1)])]
        );

        assert_eq!(
            bdd.interpretations(formula1, false, Var(2), &[], &[]),
            vec![(vec![Var(1)], vec![Var(0)]), (vec![Var(0)], vec![])]
        );

        assert_eq!(
            bdd.interpretations(formula1, false, Var(0), &[], &[]),
            vec![(vec![Var(0)], vec![])]
        );

        assert_eq!(
            bdd.interpretations(formula2, false, Var(2), &[], &[]),
            vec![
                (vec![], vec![Var(0), Var(1)]),
                (vec![Var(0), Var(1)], vec![])
            ]
        );

        assert_eq!(
            bdd.interpretations(formula2, true, Var(2), &[], &[]),
            vec![(vec![Var(1)], vec![Var(0)]), (vec![Var(0)], vec![Var(1)])]
        );

        assert_eq!(
            bdd.interpretations(formula2, true, Var(0), &[], &[]),
            vec![(vec![Var(1)], vec![Var(0)])]
        );

        assert_eq!(
            bdd.interpretations(Term::TOP, true, Var(0), &[], &[]),
            vec![]
        );
    }
}
