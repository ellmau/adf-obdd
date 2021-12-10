//! This module describes the abstract dialectical framework
//!
//! It handles
//!  - parsing of statements and acceptance functions
//!  - computing interpretations
//!  - computing fixpoints
//!  - computing the least fixpoint by using a shortcut

#![warn(missing_docs)]

use std::{
    collections::HashMap,
    str::{self, FromStr},
    usize,
};

use crate::datatypes::{Term, Var};
use crate::obdd::Bdd;

struct Statement {
    label: String,     // label of the statement
    var: usize,        // variable node in bdd
    ac: Option<usize>, // node in bdd
}

impl Statement {
    pub fn new(label: &str, var: usize) -> Self {
        Statement {
            label: String::from_str(label).unwrap(),
            var,
            ac: None,
        }
    }

    pub fn _add_ac(&mut self, ac: usize) {
        self.ac = Some(ac);
    }
}

/// ADF structure which offers some statice and instance methods for handling ADFs.
pub struct Adf {
    bdd: Bdd,
    stmts: Vec<Statement>,
    dict: HashMap<String, usize>, // label to pos in vec
}

impl Default for Adf {
    fn default() -> Self {
        Adf::new()
    }
}

impl Adf {
    /// Creates a new and empty ADF.
    ///
    /// To add statements call [`init_statements`](Adf::init_statements()) and then add to each statement it's corresponding acceptance contidion via
    /// [`add_ac`](Adf::add_ac()).
    pub fn new() -> Self {
        Adf {
            bdd: Bdd::new(),
            stmts: Vec::new(),
            dict: HashMap::new(),
        }
    }

    fn add_statement(&mut self, statement: &str) {
        if self.dict.get(statement).is_none() {
            let pos = self.stmts.len();
            //self.bdd.variable(pos);
            //self.stmts
            //    .push(Statement::new(statement, pos.clone()));
            self.stmts.push(Statement::new(
                statement,
                self.bdd.variable(Var(pos)).value(),
            ));
            self.dict.insert(self.stmts[pos].label.to_string(), pos);
        }
    }

    /// Initialise statements
    ///
    /// The order of statements given as a parameter will determine die ordering for the OBDD.
    /// Note that only initialised statements will be regocnised as variables later on.
    /// This method can be called multiple times to add more statements.
    pub fn init_statements(&mut self, stmts: Vec<&str>) -> usize {
        for i in stmts.iter() {
            self.add_statement(*i);
        }
        self.stmts.len()
    }

    /// Adds to a given statement its acceptance condition.
    ///
    /// This ac needs to be in the prefix notation for ADFs as defined by the DIAMOND implementation.
    /// Every statement needs to be initialised before by [`init_statements'](Adf::init_statements()).
    pub fn add_ac(&mut self, statement: &str, ac: &str) {
        if let Some(stmt) = self.dict.get(statement) {
            let st = *stmt;
            self.add_ac_by_number(st, ac)
        }
    }

    fn add_ac_by_number(&mut self, st: usize, ac: &str) {
        let ac_num = self.parse_formula(ac);
        self.set_ac(st, ac_num);
    }

    fn set_ac(&mut self, st: usize, ac: usize) {
        self.stmts[st].ac = Some(ac);
    }

    /// Computes the grounded model of the adf.
    ///
    /// Note that this computation will shortcut interpretation updates and needs less steps than computing the least fixpoint by an usual fixpoint-construction
    pub fn grounded(&mut self) -> Vec<usize> {
        let mut interpretation: Vec<usize> = Vec::new();
        let mut change: bool;

        for it in self.stmts.iter() {
            interpretation.push((*it).ac.unwrap())
        }
        loop {
            change = false;
            for pos in 0..self.stmts.len() - 1 {
                let curint = interpretation.clone();
                match Term(curint[pos]) {
                    Term::BOT => {
                        if let Some(n) = self.setvarvalue(
                            curint,
                            self.bdd.nodes[self.stmts[pos].var].var().value(),
                            false,
                        ) {
                            interpretation.clone_from(&n);
                            change = true;
                        }
                    }
                    Term::TOP => {
                        if let Some(n) = self.setvarvalue(
                            curint,
                            self.bdd.nodes[self.stmts[pos].var].var().value(),
                            true,
                        ) {
                            interpretation.clone_from(&n);
                            change = true;
                        }
                    }
                    _ => (),
                }
            }
            if !change {
                break;
            }
        }
        interpretation
    }

    /// Function not working - do not use
    pub(crate) fn _complete(&mut self) -> Vec<Vec<usize>> {
        let base_int = self.cur_interpretation();
        let mut complete: Vec<Vec<usize>> = Vec::new();
        let mut change: bool;
        let mut pos: usize = 0;
        let mut cache: HashMap<Vec<usize>, usize> = HashMap::new();

        // compute grounded interpretation
        complete.push(self.compute_fixpoint(base_int.as_ref()).unwrap());
        loop {
            change = false;
            let interpr = complete.get(pos).unwrap().clone();
            for (pos, it) in interpr.iter().enumerate() {
                if *it > 1 {
                    let mut int1 = interpr.clone();
                    int1[pos] = 0;
                    if let Some(n) = self.compute_fixpoint(int1.as_ref()) {
                        if !cache.contains_key(&n) {
                            cache.insert(n.clone(), complete.len());
                            complete.push(n);
                            change = true;
                        }
                    }
                    int1[pos] = 1;
                    if let Some(n) = self.compute_fixpoint(int1.as_ref()) {
                        if !cache.contains_key(&n) {
                            cache.insert(n.clone(), complete.len());
                            complete.push(n);
                            change = true;
                        }
                    }
                }
            }
            if !change {
                break;
            };
            pos += 1;
            // println!("{}",complete.len());
        }
        complete
    }

    /// represents the starting interpretation due to the acceptance conditions. (i.e. the currently represented set of formulae)
    pub fn cur_interpretation(&self) -> Vec<usize> {
        let mut interpretation: Vec<usize> = Vec::new();
        for it in self.stmts.iter() {
            interpretation.push((*it).ac.unwrap())
        }
        interpretation
    }

    /// Given an Interpretation, follow the `Γ`-Operator to a fixpoint. Returns `None` if no valid fixpoint can be reached from the given interpretation and
    /// the interpretation which represents the fixpoint otherwise.
    #[allow(clippy::ptr_arg)]
    pub fn compute_fixpoint(&mut self, interpretation: &Vec<usize>) -> Option<Vec<usize>> {
        let new_interpretation = self.apply_interpretation(interpretation.as_ref());
        match Adf::information_enh(interpretation, new_interpretation.as_ref()) {
            Some(n) => {
                if n {
                    self.compute_fixpoint(new_interpretation.as_ref())
                } else {
                    Some(new_interpretation)
                }
            }
            None => None,
        }
    }

    #[allow(clippy::ptr_arg)]
    fn apply_interpretation(&mut self, interpretation: &Vec<usize>) -> Vec<usize> {
        let mut new_interpretation = interpretation.clone();
        for (pos, it) in interpretation.iter().enumerate() {
            match Term(*it) {
                Term::BOT => {
                    if let Some(n) = self.setvarvalue(
                        new_interpretation.clone(),
                        self.bdd.nodes[self.stmts[pos].var].var().value(),
                        false,
                    ) {
                        new_interpretation.clone_from(&n);
                    }
                }
                Term::TOP => {
                    if let Some(n) = self.setvarvalue(
                        new_interpretation.clone(),
                        self.bdd.nodes[self.stmts[pos].var].var().value(),
                        true,
                    ) {
                        new_interpretation.clone_from(&n);
                    }
                }
                _ => (),
            }
        }
        new_interpretation
    }

    #[allow(clippy::ptr_arg)]
    fn information_enh(i1: &Vec<usize>, i2: &Vec<usize>) -> Option<bool> {
        let mut enhanced = false;
        for i in 0..i1.len() {
            if i1[i] < 2 {
                if i1[i] != i2[i] {
                    return None;
                }
            } else if (i1[i] >= 2) & (i2[i] < 2) {
                enhanced = true;
            }
        }
        Some(enhanced)
    }

    fn setvarvalue(
        &mut self,
        interpretation: Vec<usize>,
        var: usize,
        val: bool,
    ) -> Option<Vec<usize>> {
        let mut interpretation2: Vec<usize> = vec![0; interpretation.len()];
        let mut change: bool = false;
        for (pos, _it) in interpretation.iter().enumerate() {
            interpretation2[pos] = self
                .bdd
                .restrict(Term(interpretation[pos]), Var(var), val)
                .value();
            if interpretation[pos] != interpretation2[pos] {
                change = true
            }
        }
        if change {
            Some(interpretation2)
        } else {
            None
        }
    }

    fn parse_formula(&mut self, ac: &str) -> usize {
        if ac.len() > 3 {
            match &ac[..4] {
                "and(" => {
                    let (left, right) = Adf::findpairs(&ac[4..]);
                    let lterm: Term = self.parse_formula(left).into();
                    let rterm: Term = self.parse_formula(right).into();
                    return self.bdd.and(lterm, rterm).value();
                }
                "iff(" => {
                    let (left, right) = Adf::findpairs(&ac[4..]);
                    let lterm: Term = self.parse_formula(left).into();
                    let rterm: Term = self.parse_formula(right).into();
                    return self.bdd.iff(lterm, rterm).value();
                }
                "xor(" => {
                    let (left, right) = Adf::findpairs(&ac[4..]);
                    let lterm: Term = self.parse_formula(left).into();
                    let rterm: Term = self.parse_formula(right).into();
                    return self.bdd.xor(lterm, rterm).value();
                }
                "imp(" => {
                    let (left, right) = Adf::findpairs(&ac[4..]);
                    let lterm: Term = self.parse_formula(left).into();
                    let rterm: Term = self.parse_formula(right).into();
                    return self.bdd.imp(lterm, rterm).value();
                }
                "neg(" => {
                    let pos = Adf::findterm(&ac[4..]).unwrap() + 4;
                    let term: Term = self.parse_formula(&ac[4..pos]).into();
                    return self.bdd.not(term).value();
                }
                "c(f)" => return Bdd::constant(false).value(),
                "c(v)" => return Bdd::constant(true).value(),
                _ if &ac[..3] == "or(" => {
                    let (left, right) = Adf::findpairs(&ac[3..]);
                    let lterm: Term = self.parse_formula(left).into();
                    let rterm: Term = self.parse_formula(right).into();
                    return self.bdd.or(lterm, rterm).value();
                }
                _ => (),
            }
        }
        match self.dict.get(ac) {
            Some(it) => self.bdd.variable(Var(*it)).value(),
            _ => {
                println!("{}", ac);
                unreachable!()
            }
        }
    }

    /// Given a pair of literals, returns both literals as strings
    /// # Example
    /// ```rust
    /// use adf_bdd::adf::Adf;
    /// assert_eq!(Adf::findpairs("a,or(b,c))"), ("a", "or(b,c)"));
    /// assert_eq!(Adf::findpairs("a,or(b,c)"), ("a", "or(b,c)"));
    /// ```
    pub fn findpairs(formula: &str) -> (&str, &str) {
        let lpos = Adf::findterm(formula).unwrap();
        let rpos = Adf::findterm(&formula[lpos + 1..]).unwrap() + lpos;
        (&formula[..lpos], &formula[lpos + 1..rpos + 1])
    }

    /// Returns the first term in the given slice as a string slice
    /// # Example
    /// ```rust
    /// use adf_bdd::adf::Adf;  
    /// assert_eq!(Adf::findterm_str("formula"), "formula");
    /// assert_eq!(Adf::findterm_str("and(a,b),or(a,b)"), "and(a,b)");
    /// assert_eq!(Adf::findterm_str("neg(atom(a.d.ee))"), "neg(atom(a.d.ee))");
    /// assert_eq!(Adf::findterm_str("formula)"), "formula");
    /// ```
    pub fn findterm_str(formula: &str) -> &str {
        &formula[..Adf::findterm(formula).unwrap()]
    }

    fn findterm(formula: &str) -> Option<usize> {
        let mut sqbrack: i16 = 0;
        let mut cobrack: i16 = 0;

        for (i, c) in formula.chars().enumerate() {
            match c {
                '(' => sqbrack += 1,
                '[' => cobrack += 1,
                ',' => {
                    if sqbrack + cobrack == 0 {
                        return Some(i);
                    }
                }
                ')' => {
                    if sqbrack > 0 {
                        sqbrack -= 1;
                    } else {
                        return Some(i);
                    }
                }
                ']' => {
                    if cobrack > 0 {
                        cobrack -= 1;
                    } else {
                        return Some(i);
                    }
                }
                _ => (),
            }
        }
        Some(formula.len())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_statement() {
        let mut adf = Adf::new();

        adf.add_statement("A");
        adf.add_statement("B");
        adf.add_statement("A");

        assert_eq!(adf.stmts.len(), 2);

        adf.add_statement("C");
        assert_eq!(adf.stmts.len(), 3);
    }

    #[test]
    fn parse_formula() {
        let mut adf = Adf::new();

        adf.add_statement("a");
        adf.add_statement("b");
        adf.add_statement("c");

        assert_eq!(adf.parse_formula("and(a,or(b,c))"), 6);
        assert_eq!(adf.parse_formula("xor(a,b)"), 8);
        assert_eq!(adf.parse_formula("or(c(f),b)"), 3); // is b

        adf.parse_formula("and(or(c(f),a),and(b,c))");
    }

    #[test]
    #[should_panic]
    fn parse_formula_panic() {
        let mut adf = Adf::new();

        adf.add_statement("a");
        adf.add_statement("b");
        adf.add_statement("c");

        adf.parse_formula("and(a,or(b,d))");
    }

    #[test]
    fn findterm() {
        assert_eq!(Adf::findterm("formula"), Some(7));
        assert_eq!(Adf::findterm("and(a,b),or(a,b)"), Some(8));
        assert_eq!(Adf::findterm("neg(atom(a.d.ee))"), Some(17));
        assert_eq!(Adf::findterm("formula)"), Some(7));
    }

    #[test]
    fn findpairs() {
        assert_eq!(Adf::findpairs("a,or(b,c))"), ("a", "or(b,c)"));
    }

    #[test]
    fn init_statements() {
        let mut adf = Adf::new();

        let stmts: Vec<&str> = vec!["a", "b", "c", "extra long text type statement"];

        assert_eq!(adf.init_statements(stmts), 4);
        assert_eq!(adf.stmts[3].label, "extra long text type statement");
    }
}
