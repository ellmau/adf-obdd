//! This module contains the Parser for ADFs
//! It utilises the [nom-crate](https://crates.io/crates/nom)
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alphanumeric1, multispace0},
    combinator::value,
    multi::many1,
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult,
};

/// A representation of a formula, still using the strings from the input
#[derive(Clone, PartialEq, Eq)]
pub enum Formula<'a> {
    /// c(v) in the input format
    Bot,
    /// c(f) in the input format
    Top,
    /// Some atomic variable in the input format
    Atom(&'a str),
    /// Negation of a subformula
    Not(Box<Formula<'a>>),
    /// Conjunction of two subformulae
    And(Box<Formula<'a>>, Box<Formula<'a>>),
    /// Disjunction of two subformulae
    Or(Box<Formula<'a>>, Box<Formula<'a>>),
    /// Implication of two subformulae
    Imp(Box<Formula<'a>>, Box<Formula<'a>>),
    /// Exclusive-Or of two subformulae
    Xor(Box<Formula<'a>>, Box<Formula<'a>>),
    /// If and only if connective between two formulae
    Iff(Box<Formula<'a>>, Box<Formula<'a>>),
}

impl std::fmt::Debug for Formula<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Formula::Atom(a) => {
                write!(f, "{}", a)?;
            }
            Formula::Not(n) => {
                write!(f, "not({:?})", n)?;
            }
            Formula::And(f1, f2) => {
                write!(f, "and({:?},{:?})", f1, f2)?;
            }
            Formula::Or(f1, f2) => {
                write!(f, "or({:?},{:?})", f1, f2)?;
            }
            Formula::Imp(f1, f2) => {
                write!(f, "imp({:?},{:?})", f1, f2)?;
            }
            Formula::Xor(f1, f2) => {
                write!(f, "xor({:?},{:?})", f1, f2)?;
            }
            Formula::Iff(f1, f2) => {
                write!(f, "iff({:?},{:?})", f1, f2)?;
            }
            Formula::Bot => {
                write!(f, "Const(B)")?;
            }
            Formula::Top => {
                write!(f, "Const(T)")?;
            }
        }
        write!(f, "")
    }
}

/// A parse structure to hold all the information given by the input file in one place
/// Due to an internal representation with [std::cell::RefCell] and [std::rc::Rc] the values can be
/// handed over to other structures without further storage needs.
pub struct AdfParser<'a> {
    namelist: Rc<RefCell<Vec<String>>>,
    dict: Rc<RefCell<HashMap<String, usize>>>,
    formulae: RefCell<Vec<Formula<'a>>>,
    formulaname: RefCell<Vec<String>>,
}

impl Default for AdfParser<'_> {
    fn default() -> Self {
        AdfParser {
            namelist: Rc::new(RefCell::new(Vec::new())),
            dict: Rc::new(RefCell::new(HashMap::new())),
            formulae: RefCell::new(Vec::new()),
            formulaname: RefCell::new(Vec::new()),
        }
    }
}

impl<'a, 'b> AdfParser<'b>
where
    'a: 'b,
{
    fn parse_statements(&'a self) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
        move |input| {
            let (rem, _) = many1(self.parse_statement())(input)?;
            Ok((rem, ()))
        }
    }

    /// Parses a full input file and creates internal structures.
    /// It can be provided to a [crate::datatypes::adf::Adf] to initialise a new ADF
    /// Note that this method returns a closure (see the following Example for the correct usage).
    /// # Example
    /// ```
    /// let parser = adf_bdd::parser::AdfParser::default();
    /// parser.parse()("s(a).ac(a,c(v)).s(b).ac(b,a).s(c).ac(c,neg(b)).");
    /// let adf = adf_bdd::datatypes::adf::Adf::from_parser(&parser);
    /// ```
    pub fn parse(&'a self) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
        |input| value((), many1(alt((self.parse_statement(), self.parse_ac()))))(input)
    }

    fn parse_statement(&'a self) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
        |input| {
            let mut dict = self.dict.borrow_mut();
            let mut namelist = self.namelist.borrow_mut();
            let (remain, statement) =
                terminated(AdfParser::statement, terminated(tag("."), multispace0))(input)?;
            if !dict.contains_key(statement) {
                let pos = namelist.len();
                namelist.push(String::from(statement));
                dict.insert(namelist[pos].clone(), pos);
            }
            Ok((remain, ()))
        }
    }

    fn parse_ac(&'a self) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
        |input| {
            let (remain, (name, formula)) =
                terminated(AdfParser::ac, terminated(tag("."), multispace0))(input)?;
            self.formulae.borrow_mut().push(formula);
            self.formulaname.borrow_mut().push(String::from(name));
            Ok((remain, ()))
        }
    }
}

impl AdfParser<'_> {
    fn statement(input: &str) -> IResult<&str, &str> {
        preceded(tag("s"), delimited(tag("("), AdfParser::atomic, tag(")")))(input)
    }

    fn ac(input: &str) -> IResult<&str, (&str, Formula)> {
        preceded(
            tag("ac"),
            delimited(
                tag("("),
                separated_pair(
                    AdfParser::atomic,
                    delimited(multispace0, tag(","), multispace0),
                    AdfParser::formula,
                ),
                tag(")"),
            ),
        )(input)
    }

    fn atomic_term(input: &str) -> IResult<&str, Formula> {
        AdfParser::atomic(input).map(|(input, result)| (input, Formula::Atom(result)))
    }

    fn formula(input: &str) -> IResult<&str, Formula> {
        alt((
            AdfParser::constant,
            AdfParser::binary_op,
            AdfParser::unary_op,
            AdfParser::atomic_term,
        ))(input)
    }

    fn unary_op(input: &str) -> IResult<&str, Formula> {
        preceded(
            tag("neg"),
            delimited(tag("("), AdfParser::formula, tag(")")),
        )(input)
        .map(|(input, result)| (input, Formula::Not(Box::new(result))))
    }

    fn constant(input: &str) -> IResult<&str, Formula> {
        alt((
            preceded(tag("c"), delimited(tag("("), tag("v"), tag(")"))),
            preceded(tag("c"), delimited(tag("("), tag("f"), tag(")"))),
        ))(input)
        .map(|(input, result)| {
            (
                input,
                match result {
                    "v" => Formula::Top,
                    "f" => Formula::Bot,
                    _ => unreachable!(),
                },
            )
        })
    }

    fn formula_pair(input: &str) -> IResult<&str, (Formula, Formula)> {
        separated_pair(
            preceded(tag("("), AdfParser::formula),
            delimited(multispace0, tag(","), multispace0),
            terminated(AdfParser::formula, tag(")")),
        )(input)
    }

    fn and(input: &str) -> IResult<&str, Formula> {
        preceded(tag("and"), AdfParser::formula_pair)(input)
            .map(|(input, (f1, f2))| (input, Formula::And(Box::new(f1), Box::new(f2))))
    }

    fn or(input: &str) -> IResult<&str, Formula> {
        preceded(tag("or"), AdfParser::formula_pair)(input)
            .map(|(input, (f1, f2))| (input, Formula::Or(Box::new(f1), Box::new(f2))))
    }
    fn imp(input: &str) -> IResult<&str, Formula> {
        preceded(tag("imp"), AdfParser::formula_pair)(input)
            .map(|(input, (f1, f2))| (input, Formula::Imp(Box::new(f1), Box::new(f2))))
    }

    fn xor(input: &str) -> IResult<&str, Formula> {
        preceded(tag("xor"), AdfParser::formula_pair)(input)
            .map(|(input, (f1, f2))| (input, Formula::Xor(Box::new(f1), Box::new(f2))))
    }

    fn iff(input: &str) -> IResult<&str, Formula> {
        preceded(tag("iff"), AdfParser::formula_pair)(input)
            .map(|(input, (f1, f2))| (input, Formula::Iff(Box::new(f1), Box::new(f2))))
    }

    fn binary_op(input: &str) -> IResult<&str, Formula> {
        alt((
            AdfParser::and,
            AdfParser::or,
            AdfParser::imp,
            AdfParser::xor,
            AdfParser::iff,
        ))(input)
    }

    fn atomic(input: &str) -> IResult<&str, &str> {
        alt((
            delimited(tag("\""), take_until("\""), tag("\"")),
            alphanumeric1,
        ))(input)
    }

    /// Allows insight of the number of parsed Statements
    pub fn dict_size(&self) -> usize {
        //self.dict.borrow().len()
        self.dict.as_ref().borrow().len()
    }

    /// Returns the number-representation and position of a given variable/statement in string-representation
    pub fn dict_value(&self, value: &str) -> Option<usize> {
        self.dict.as_ref().borrow().get(value).copied()
    }

    /// Returns the acceptance condition of a statement at the given positon
    pub fn ac_at(&self, idx: usize) -> Option<Formula> {
        self.formulae.borrow().get(idx).cloned()
    }

    pub(crate) fn dict_rc_refcell(&self) -> Rc<RefCell<HashMap<String, usize>>> {
        Rc::clone(&self.dict)
    }

    pub(crate) fn namelist_rc_refcell(&self) -> Rc<RefCell<Vec<String>>> {
        Rc::clone(&self.namelist)
    }

    pub(crate) fn formula_count(&self) -> usize {
        self.formulae.borrow().len()
    }

    pub(crate) fn formula_order(&self) -> Vec<usize> {
        self.formulaname
            .borrow()
            .iter()
            .map(|name| *self.dict.as_ref().borrow().get(name).unwrap())
            .collect()
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn atomic_parse() {
        assert_eq!(
            AdfParser::atomic("\"   123  21 ())) ((( {}|||    asfjklj fsajfj039409u902 jfi a \""),
            Ok((
                "",
                "   123  21 ())) ((( {}|||    asfjklj fsajfj039409u902 jfi a "
            ))
        );
        assert_eq!(AdfParser::atomic("foo"), Ok(("", "foo")));
        assert_eq!(AdfParser::atomic("foo()"), Ok(("()", "foo")));
        assert_eq!(
            AdfParser::atomic("()foo"),
            Err(nom::Err::Error(nom::error::Error::new(
                "()foo",
                nom::error::ErrorKind::AlphaNumeric
            )))
        );
        assert!(AdfParser::atomic(" adf").is_err());
    }

    #[test]
    fn statement_parse() {
        assert_eq!(AdfParser::statement("s(ab)"), Ok(("", "ab")));
        assert_eq!(AdfParser::statement("s(\"a b\")"), Ok(("", "a b")));
        assert!(AdfParser::statement("s(a_b)").is_err());
    }

    #[test]
    fn parse_statement() {
        let parser: AdfParser = AdfParser::default();

        let input = "s(a).    s(b). s(c).s(d).s(b).s(c).";
        //        many1(parser.parse_statement())(input).unwrap();
        let (_remain, _) = parser.parse_statement()(input).unwrap();
        assert_eq!(parser.dict_size(), 1);
        assert_eq!(parser.dict_value("c"), None);

        let (_remain, _) = parser.parse_statements()(input).unwrap();
        assert_eq!(parser.dict_size(), 4);
        assert_eq!(parser.dict_value("c"), Some(2usize));
    }

    #[test]
    fn parse_formula() {
        let input = "and(or(neg(a),iff(\" iff left \",b)),xor(imp(c,d),e))";
        let (_remain, result) = AdfParser::formula(input).unwrap();

        assert_eq!(
            format!("{:?}", result),
            "and(or(not(a),iff( iff left ,b)),xor(imp(c,d),e))"
        );

        assert_eq!(
            AdfParser::formula("and(c(v),c(f))").unwrap(),
            (
                "",
                Formula::And(Box::new(Formula::Top), Box::new(Formula::Bot))
            )
        );
    }

    #[test]
    fn parse() {
        let parser = AdfParser::default();
        let input = "s(a).s(c).ac(a,b).ac(b,neg(a)).s(b).ac(c,and(c(v),or(c(f),a))).";

        let (remain, _) = parser.parse()(input).unwrap();
        assert_eq!(remain, "");
        assert_eq!(parser.dict_size(), 3);
        assert_eq!(parser.dict_value("b"), Some(2usize));
        assert_eq!(
            format!("{:?}", parser.ac_at(1).unwrap()),
            format!("{:?}", Formula::Not(Box::new(Formula::Atom("a"))))
        );
        assert_eq!(parser.formula_count(), 3);
        assert_eq!(parser.formula_order(), vec![0, 2, 1]);
    }

    #[test]
    fn constant() {
        assert_eq!(AdfParser::constant("c(v)").unwrap().1, Formula::Top);
        assert_eq!(AdfParser::constant("c(f)").unwrap().1, Formula::Bot);
        assert_eq!(format!("{:?}", Formula::Top), "Const(T)");
        assert_eq!(format!("{:?}", Formula::Bot), "Const(B)");
    }
}
