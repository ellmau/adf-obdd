use std::{
    collections::HashMap,
    num::ParseFloatError,
    ops::Deref,
    str::{self, FromStr},
};

use super::obdd::Bdd;

struct Statement {
    label: String,     // label of the statement
    var: usize,        // variable node in bdd
    ac: Option<usize>, // node in bdd
}

impl Statement {
    pub fn new(label: &str, var: usize) -> Self {
        Statement {
            label: String::from_str(label).unwrap(),
            var: var,
            ac: None,
        }
    }

    pub fn add_ac(&mut self, ac: usize) {
        self.ac = Some(ac);
    }
}

struct Adf {
    bdd: Bdd,
    stmts: Vec<Statement>,
    dict: HashMap<String, usize>, // label to pos in vec
}

impl Adf {
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
            self.stmts
                .push(Statement::new(statement, self.bdd.variable(pos).clone()));
            self.dict.insert(self.stmts[pos].label.to_string(), pos);
        }
    }

    pub fn init_statements(&mut self, stmts: Vec<&str>) -> usize {
        for i in stmts.iter() {
            self.add_statement(*i);
        }
        self.stmts.len()
    }

    pub fn add_ac(&mut self, statement: &str, ac: &str) {
        if let Some(stmt) = self.dict.get(statement) {
          self.add_ac_by_number(*stmt, ac)
        }
    }

    fn add_ac_by_number(&mut self, st:usize, ac: &str){
      let ac_num = self.parseformula(ac);
      self.set_ac(st, ac_num);
    }

    fn set_ac(&mut self, st: usize, ac: usize) {
        self.stmts[st].ac = Some(ac);
    }

    fn parseformula(&mut self, ac: &str) -> usize {
        if let Some(split) = ac.find(',') {
            let (l, r) = ac.split_at(split);
            let rterm = self.parseformula(&r[1..r.len() - 1]);
            if ac.starts_with("and(") {
                let lterm = self.parseformula(&l[4..]);
                self.bdd.and(lterm, rterm)
            } else if ac.starts_with("or(") {
                let lterm = self.parseformula(&l[3..]);
                self.bdd.or(lterm, rterm)
            } else if ac.starts_with("iff(") {
                let lterm = self.parseformula(&l[4..]);
                let neglterm = self.bdd.not(lterm);
                let negrterm = self.bdd.not(rterm);
                let con1 = self.bdd.and(lterm, rterm);
                let con2 = self.bdd.and(neglterm, negrterm);

                self.bdd.or(con1, con2)
            } else {
                //if ac.starts_with("xor("){
                let lterm = self.parseformula(&l[4..]);
                let neglterm = self.bdd.not(lterm);
                let negrterm = self.bdd.not(rterm);
                let con1 = self.bdd.and(neglterm, rterm);
                let con2 = self.bdd.and(lterm, negrterm);

                self.bdd.or(con1, con2)
            }
        } else {
            if ac.starts_with("neg(") {
                let term = self.parseformula(&ac[4..ac.len() - 1]);
                self.bdd.not(term)
            } else if ac.eq("c(v)") {
                self.bdd.constant(true)
            } else if ac.eq("c(f)") {
                self.bdd.constant(false)
            } else {
                match self.dict.get(ac) {
                    Some(it) => self.stmts[*it].var,
                    _ => unreachable!(),
                }
            }
        }
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
    fn parseformula() {
        let mut adf = Adf::new();

        adf.add_statement("a");
        adf.add_statement("b");
        adf.add_statement("c");

        assert_eq!(adf.parseformula("and(a,or(b,c))"), 6);
        assert_eq!(adf.parseformula("xor(a,b)"), 11);
        assert_eq!(adf.parseformula("or(c(f),b)"), 3); // is b
    }

    #[test]
    #[should_panic]
    fn parseformula_panic() {
        let mut adf = Adf::new();

        adf.add_statement("a");
        adf.add_statement("b");
        adf.add_statement("c");

        adf.parseformula("and(a,or(b,d))");
    }

    #[test]
    fn init_statements() {
        let mut adf = Adf::new();

        let stmts: Vec<&str> = vec!["a", "b", "c", "extra long text type statement"];

        assert_eq!(adf.init_statements(stmts), 4);
        assert_eq!(adf.stmts[3].label, "extra long text type statement");
    }
}
