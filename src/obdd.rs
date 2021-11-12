use std::usize;

/// constant which represents the bottom concept (i.e. inconsistency)
pub const BDD_BOT: usize = 0;
/// constant which represents the top concept (i.e. universal truth)
pub const BDD_TOP: usize = 1;

use std::collections::HashMap;

/// Convenience type substition
type Term = usize;

/// Represents a node in the BDD
///
///
#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub(crate) struct BddNode {
    var: usize,
    lo: Term,
    hi: Term,
}

impl std::fmt::Display for BddNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BDDNode: {}, lo:{}, hi: {}", self.var, self.lo, self.hi)
    }
}

impl BddNode {
    pub fn var(self) -> usize {
        self.var
    }
}

pub(crate) struct Bdd {
    pub(crate) nodes: Vec<BddNode>,
    hash: HashMap<BddNode, Term>,
}

impl Bdd {
    pub fn new() -> Self {
        let botnode = BddNode {
            var: usize::MAX,
            lo: BDD_BOT,
            hi: BDD_BOT,
        };
        let topnode = BddNode {
            var: usize::MAX,
            lo: BDD_TOP,
            hi: BDD_TOP,
        };
        Self {
            nodes: vec![botnode, topnode],
            hash: HashMap::new(),
        }
    }

    fn create_node(&mut self, var: usize, lo: Term, hi: Term) -> Term {
        if lo == hi {
            lo
        } else {
            let node = BddNode { var, lo, hi };
            match self.hash.get(&node) {
                Some(n) => *n,
                None => {
                    let newid = self.nodes.len();
                    if newid == usize::MAX {
                        panic!("Maximal amount of elements in node-table reached!")
                    }
                    self.nodes.push(node);
                    self.hash.insert(node, newid);
                    newid
                }
            }
        }
    }

    pub fn variable(&mut self, var: usize) -> Term {
        self.create_node(var, BDD_BOT, BDD_TOP)
    }

    pub fn constant(&self, val: bool) -> Term {
        if val {
            BDD_TOP
        } else {
            BDD_BOT
        }
    }

    pub fn restrict(&mut self, subtree: Term, var: usize, val: bool) -> Term {
        let treenode = self.nodes[subtree];
        #[allow(clippy::collapsible_else_if)]
        // Better readabilty of the if/then/else structure of the algorithm
        if treenode.var > var || treenode.var == usize::MAX {
            subtree
        } else if treenode.var < var {
            let lonode = self.restrict(treenode.lo, var, val);
            let hinode = self.restrict(treenode.hi, var, val);
            self.create_node(treenode.var, lonode, hinode)
        } else {
            if val {
                self.restrict(treenode.hi, var, val)
            } else {
                self.restrict(treenode.lo, var, val)
            }
        }
    }

    fn if_then_else(&mut self, i: Term, t: Term, e: Term) -> Term {
        if i == BDD_TOP {
            t
        } else if i == BDD_BOT {
            e
        } else if t == e {
            t
        } else if t == BDD_TOP && e == BDD_BOT {
            i
        } else {
            let ivar = self.nodes[i].var;
            let tvar = self.nodes[t].var;
            let evar = self.nodes[e].var;

            let minvar = std::cmp::min(ivar, std::cmp::min(tvar, evar));

            let itop = self.restrict(i, minvar, true);
            let ttop = self.restrict(t, minvar, true);
            let etop = self.restrict(e, minvar, true);
            let ibot = self.restrict(i, minvar, false);
            let tbot = self.restrict(t, minvar, false);
            let ebot = self.restrict(e, minvar, false);

            let topite = self.if_then_else(itop, ttop, etop);
            let botite = self.if_then_else(ibot, tbot, ebot);
            self.create_node(minvar, botite, topite)
        }
    }

    pub fn not(&mut self, term: Term) -> Term {
        self.if_then_else(term, BDD_BOT, BDD_TOP)
    }

    pub fn and(&mut self, terma: Term, termb: Term) -> Term {
        self.if_then_else(terma, termb, BDD_BOT)
    }

    pub fn or(&mut self, terma: Term, termb: Term) -> Term {
        self.if_then_else(terma, BDD_TOP, termb)
    }

    fn _printtree(&self, tree: Term) {
        let node = self.nodes[tree];
        println!("Index: {}, Node: {}", tree, node);

        if tree > BDD_TOP {
            self._printtree(node.lo);
            self._printtree(node.hi);
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
        assert_eq!(bdd.constant(true), BDD_TOP);
        assert_eq!(bdd.constant(false), BDD_BOT);
        assert_eq!(bdd.nodes.len(), 2);
    }

    #[test]
    fn addvar() {
        let mut bdd = Bdd::new();
        assert_eq!(bdd.variable(0), 2);
        assert_eq!(bdd.variable(1), 3);
        assert_eq!(bdd.variable(0), 2);
    }

    #[test]
    fn use_negation() {
        let mut bdd = Bdd::new();
        let v1 = bdd.variable(0);
        assert_eq!(v1, 2);
        assert_eq!(bdd.not(v1), 3);
    }

    #[test]
    fn use_add() {
        let mut bdd = Bdd::new();
        let v1 = bdd.variable(0);
        let v2 = bdd.variable(1);
        let v3 = bdd.variable(2);

        let a1 = bdd.and(v1, v2);
        let a2 = bdd.and(a1, v3);
        assert_eq!(a2, 7);
    }

    #[test]
    fn use_or() {
        let mut bdd = Bdd::new();

        let v1 = bdd.variable(0);
        let v2 = bdd.variable(1);
        let v3 = bdd.variable(2);

        let a1 = bdd.and(v1, v2);
        let a2 = bdd.or(a1, v3);
        assert_eq!(a2, 7);
    }

    #[test]
    fn produce_different_conversions() {
        let mut bdd = Bdd::new();

        let v1 = bdd.variable(0);
        let v2 = bdd.variable(1);

        let t1 = bdd.and(v1, v2);
        let nt1 = bdd.not(t1);
        let ft = bdd.or(v1, nt1);

        assert_eq!(ft, BDD_TOP);

        let v3 = bdd.variable(2);
        let nv3 = bdd.not(v3);
        assert_eq!(bdd.and(v3, nv3), BDD_BOT);

        let conj = bdd.and(v1, v2);
        assert_eq!(bdd.restrict(conj, 0, false), BDD_BOT);
        assert_eq!(bdd.restrict(conj, 0, true), v2);

        let a = bdd.and(v3, v2);
        let b = bdd.or(v2, v1);

        let con1 = bdd.and(a, conj);

        let end = bdd.or(con1, b);
        let x = bdd.restrict(end, 1, false);
        assert_eq!(x, 2);
    }
}
