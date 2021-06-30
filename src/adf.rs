use std::{collections::HashMap, str::{self, FromStr}, usize};

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
            var,
            ac: None,
        }
    }

    
    pub fn _add_ac(&mut self, ac: usize) {
        self.ac = Some(ac);
    }
}

pub struct Adf {
    bdd: Bdd,
    stmts: Vec<Statement>,
    dict: HashMap<String, usize>, // label to pos in vec
}

impl Default for Adf{
    fn default() -> Self {
        Adf::new()
    }
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
            //self.bdd.variable(pos);
            //self.stmts
            //    .push(Statement::new(statement, pos.clone()));
            self.stmts
               .push(Statement::new(statement, self.bdd.variable(pos)));
            self.dict.insert(self.stmts[pos].label.to_string(), pos);
        }
    }

    /// Initialise statements
    ///
    /// The order of statements given as a parameter will determine die ordering for the OBDD. 
    /// Note that only initialised statements will regocnised as variables later on
    pub fn init_statements(&mut self, stmts: Vec<&str>) -> usize {
        for i in stmts.iter() {
            self.add_statement(*i);
        }
        self.stmts.len()
    }

    /// Adds to a given statement its acceptance condition.
    /// 
    /// This ac needs to be in the prefix notation for ADFs as defined by the DIAMOND implementation. 
    pub fn add_ac(&mut self, statement: &str, ac: &str) {
        if let Some(stmt) = self.dict.get(statement) {
            let st = *stmt;
            self.add_ac_by_number(st, ac)
        }
    }

    fn add_ac_by_number(&mut self, st:usize, ac: &str){
      let ac_num = self.parseformula(ac);
      self.set_ac(st, ac_num);
    }

    fn set_ac(&mut self, st: usize, ac: usize) {
        self.stmts[st].ac = Some(ac);
    }

    pub fn grounded(&mut self) -> Vec<usize> {
        let mut interpretation: Vec<usize> = Vec::new();
        let mut change:bool;

        for it in self.stmts.iter(){
            interpretation.push((*it).ac.unwrap())
        }
        loop{
            change = false;
            for pos in 0..self.stmts.len()-1{
                let curint = interpretation.clone();
                match curint[pos] {
                    super::obdd::BDD_BOT => {
                        if let Some(n) = self.setvarvalue(curint,self.bdd.nodes[self.stmts[pos].var].var(),false){
                            interpretation.clone_from(&n);
                            change = true;
                        }
                    },
                    super::obdd::BDD_TOP => {
                        if let Some(n) = self.setvarvalue(curint,self.bdd.nodes[self.stmts[pos].var].var(),true){
                            interpretation.clone_from(&n);
                            change = true;
                        }
                    },
                    _ => (),
                }
            }
            if !change {break;}
        }
        interpretation
    }

    pub fn cur_interpretation(&self) -> Vec<usize>{
        let mut interpretation: Vec<usize> = Vec::new();
        for it in self.stmts.iter(){
            interpretation.push((*it).ac.unwrap())
        }
        interpretation
    }

    pub fn to_fixpoint(&mut self, interpretation:Vec<usize>) -> Option<Vec<usize>>{
        let new_interpretation = self.apply_interpretation(interpretation.as_ref());
        match Adf::information_enh(interpretation.as_ref(), new_interpretation.as_ref()){
            Some(n) => {
                if n {
                    return self.to_fixpoint(new_interpretation);
                }else{
                    return Some(new_interpretation)
                }
            },
            None => None,
        }
    }

    fn apply_interpretation(&mut self, interpretation:&Vec<usize>) -> Vec<usize>{
        let mut new_interpretation = interpretation.clone();
        for (pos,it) in interpretation.iter().enumerate(){
            match *it {
                super::obdd::BDD_BOT => {
                    if let Some(n) = self.setvarvalue(new_interpretation.clone(),self.bdd.nodes[self.stmts[pos].var].var(),false){
                        new_interpretation.clone_from(&n);
                        
                    }
                },
                super::obdd::BDD_TOP => {
                    if let Some(n) = self.setvarvalue(new_interpretation.clone(),self.bdd.nodes[self.stmts[pos].var].var(),true){
                        new_interpretation.clone_from(&n);                        
                    }
                },
                _ => (),
            }
            
        }
        new_interpretation
    }

    fn information_enh(i1:&Vec<usize>,i2:&Vec<usize>) -> Option<bool> {
        let mut enhanced = false;
        for i in 0..i1.len(){
            if i1[i] < 2{
                if i1[i] != i2[i] {
                    return None;
                }
            } else if (i1[i]>=2) & (i2[i] < 2){
                enhanced = true;
            }
        }
        Some(enhanced)
    }

    fn setvarvalue(&mut self,interpretation:Vec<usize>, var:usize, val:bool) -> Option<Vec<usize>>{
        let mut interpretation2:Vec<usize> = vec![0;interpretation.len()];
        let mut change: bool = false;
        for (pos,_it) in interpretation.iter().enumerate(){
            interpretation2[pos] = self.bdd.restrict(interpretation[pos], var, val);
            if interpretation[pos] != interpretation2[pos]{
                change = true
            }
        }
        if change{
            Some(interpretation2)
        }else{
            None
        }
    }

    fn parseformula(&mut self, ac: &str) -> usize {
        if ac.len() > 3 {
            match &ac[..4] {
                "and(" => {
                    let (left, right) = Adf::findpairs(&ac[4..]);
                    let lterm = self.parseformula(left);
                    let rterm = self.parseformula(right);
                    return self.bdd.and(lterm, rterm);
                },
                "iff(" => {
                    let (left, right) = Adf::findpairs(&ac[4..]);
                    let lterm = self.parseformula(left);
                    let rterm = self.parseformula(right);
                    let notlt = self.bdd.not(lterm);
                    let notrt = self.bdd.not(rterm);
                    let con1 = self.bdd.and(lterm, rterm);
                    let con2 = self.bdd.and(notlt,notrt);
                    return self.bdd.or(con1,con2);
                },
                "xor(" => {
                    let (left, right) = Adf::findpairs(&ac[4..]);
                    let lterm = self.parseformula(left);
                    let rterm = self.parseformula(right);
                    let notlt = self.bdd.not(lterm);
                    let notrt = self.bdd.not(rterm);
                    let con1 = self.bdd.and(notlt, rterm);
                    let con2 = self.bdd.and(lterm,notrt);
                    return self.bdd.or(con1,con2);
                },
                "neg(" => {
                    let pos = Adf::findterm(&ac[4..]).unwrap()+4;
                    let term = self.parseformula(&ac[4..pos]);
                    return self.bdd.not(term);
                },
                "c(f)" => return self.bdd.constant(false),
                "c(v)" => return self.bdd.constant(true),
                _ if &ac[..3] == "or(" => {
                    let (left, right) = Adf::findpairs(&ac[3..]);
                    let lterm = self.parseformula(left);
                    let rterm = self.parseformula(right);
                    return self.bdd.or(lterm, rterm);
                },
                _ => (),
            }
            
        }
        match self.dict.get(ac) {
            Some(it) => self.bdd.variable(*it),
            _ => {println!("{}",ac); unreachable!()}
        }
    }

    pub fn findpairs(formula: &str) -> (&str,&str){
        let lpos = Adf::findterm(formula).unwrap();
        let rpos = Adf::findterm(&formula[lpos+1..]).unwrap() + lpos;
        (&formula[..lpos],&formula[lpos+1..rpos+1])
    }

    pub fn findterm_str(formula: &str) -> &str{
        &formula[..Adf::findterm(formula).unwrap()]
    }

    fn findterm(formula: &str) -> Option<usize> {
        let mut sqbrack: i16 = 0;
        let mut cobrack: i16 = 0;

        for (i,c) in formula.chars().enumerate() {
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
                    }else{
                        return Some(i);
                    }
                }
                ']' => {
                    if cobrack > 0 {
                        cobrack -= 1;
                    }else{
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
    fn parseformula() {
        let mut adf = Adf::new();

        adf.add_statement("a");
        adf.add_statement("b");
        adf.add_statement("c");

        assert_eq!(adf.parseformula("and(a,or(b,c))"), 6);
        assert_eq!(adf.parseformula("xor(a,b)"), 11);
        assert_eq!(adf.parseformula("or(c(f),b)"), 3); // is b

        adf.parseformula("and(or(c(f),a),and(b,c))");
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
    fn findterm() {
        assert_eq!(Adf::findterm("formula"),Some(7));
        assert_eq!(Adf::findterm("and(a,b),or(a,b)"),Some(8));
        assert_eq!(Adf::findterm("neg(atom(a.d.ee))"),Some(17));
        assert_eq!(Adf::findterm("formula)"),Some(7));
    }

    #[test]
    fn findpairs() {
        assert_eq!(Adf::findpairs("a,or(b,c))"),("a","or(b,c)"));
    }

    #[test]
    fn init_statements() {
        let mut adf = Adf::new();

        let stmts: Vec<&str> = vec!["a", "b", "c", "extra long text type statement"];

        assert_eq!(adf.init_statements(stmts), 4);
        assert_eq!(adf.stmts[3].label, "extra long text type statement");
    }
}
