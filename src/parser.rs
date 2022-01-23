//! A Parser for ADFs with all needed helper-methods.
//! It utilises the [nom-crate](https://crates.io/crates/nom)
use lexical_sort::{natural_lexical_cmp, StringSort};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{alphanumeric1, multispace0},
    combinator::{all_consuming, value},
    multi::many1,
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult,
};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::datatypes::adf::VarContainer;

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

impl Formula<'_> {
    pub(crate) fn to_boolean_expr(
        &self,
    ) -> biodivine_lib_bdd::boolean_expression::BooleanExpression {
        match self {
            Formula::Top => biodivine_lib_bdd::boolean_expression::BooleanExpression::Const(true),
            Formula::Bot => biodivine_lib_bdd::boolean_expression::BooleanExpression::Const(false),
            Formula::Atom(name) => {
                biodivine_lib_bdd::boolean_expression::BooleanExpression::Variable(name.to_string())
            }
            Formula::Not(subformula) => {
                biodivine_lib_bdd::boolean_expression::BooleanExpression::Not(Box::new(
                    subformula.to_boolean_expr(),
                ))
            }
            Formula::And(sub_a, sub_b) => {
                biodivine_lib_bdd::boolean_expression::BooleanExpression::And(
                    Box::new(sub_a.to_boolean_expr()),
                    Box::new(sub_b.to_boolean_expr()),
                )
            }
            Formula::Or(sub_a, sub_b) => {
                biodivine_lib_bdd::boolean_expression::BooleanExpression::Or(
                    Box::new(sub_a.to_boolean_expr()),
                    Box::new(sub_b.to_boolean_expr()),
                )
            }
            Formula::Iff(sub_a, sub_b) => {
                biodivine_lib_bdd::boolean_expression::BooleanExpression::Iff(
                    Box::new(sub_a.to_boolean_expr()),
                    Box::new(sub_b.to_boolean_expr()),
                )
            }
            Formula::Imp(sub_a, sub_b) => {
                biodivine_lib_bdd::boolean_expression::BooleanExpression::Imp(
                    Box::new(sub_a.to_boolean_expr()),
                    Box::new(sub_b.to_boolean_expr()),
                )
            }
            Formula::Xor(sub_a, sub_b) => {
                biodivine_lib_bdd::boolean_expression::BooleanExpression::Xor(
                    Box::new(sub_a.to_boolean_expr()),
                    Box::new(sub_b.to_boolean_expr()),
                )
            }
        }
    }
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
/// Due to an internal representation with [RefCell][std::cell::RefCell] and [Rc][std::rc::Rc] the values can be
/// handed over to other structures without further storage needs.
///
/// Note that the parser can be utilised by an [ADF][`crate::datatypes::adf::Adf`] to initialise it with minimal overhead.
#[derive(Debug)]
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
    #[allow(dead_code)]
    fn parse_statements(&'a self) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
        move |input| {
            let (rem, _) = many1(self.parse_statement())(input)?;
            Ok((rem, ()))
        }
    }

    /// Parses a full input file and creates internal structures.
    /// Note that this method returns a closure (see the following Example for the correct usage).
    /// # Example
    /// ```
    /// let parser = adf_bdd::parser::AdfParser::default();
    /// parser.parse()("s(a).ac(a,c(v)).s(b).ac(b,a).s(c).ac(c,neg(b)).");
    /// let adf = adf_bdd::adf::Adf::from_parser(&parser);
    /// ```
    pub fn parse(&'a self) -> impl FnMut(&'a str) -> IResult<&'a str, ()> {
        log::info!("[Start] parsing");
        |input| {
            value(
                (),
                all_consuming(many1(alt((self.parse_statement(), self.parse_ac())))),
            )(input)
        }
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
    /// after an update to the namelist, all indizes are updated
    fn regenerate_indizes(&self) {
        self.namelist
            .as_ref()
            .borrow()
            .iter()
            .enumerate()
            .for_each(|(i, elem)| {
                self.dict.as_ref().borrow_mut().insert(elem.clone(), i);
            });
    }

    /// Sort the variables in lexicographical order.
    /// Results which got used before might become corrupted.
    pub fn varsort_lexi(&self) -> &Self {
        self.namelist.as_ref().borrow_mut().sort_unstable();
        self.regenerate_indizes();
        self
    }

    /// Sort the variables in alphanumerical order
    /// Results which got used before might become corrupted.
    pub fn varsort_alphanum(&self) -> &Self {
        self.namelist
            .as_ref()
            .borrow_mut()
            .string_sort_unstable(natural_lexical_cmp);
        self.regenerate_indizes();
        self
    }

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
    fn non_consuming_parse() {
        let parser = AdfParser::default();
        let input = "s(a).s(c).ac(a,b).ac(b,neg(a)).s(b).ac(c,and(c(v),or(c(f),a))). wee";

        let x = parser.parse()(input);
        assert!(x.is_err());
        assert_eq!(
            x.err().unwrap(),
            nom::Err::Error(nom::error::Error::new("wee", nom::error::ErrorKind::Eof))
        );
    }

    #[test]
    fn constant() {
        assert_eq!(AdfParser::constant("c(v)").unwrap().1, Formula::Top);
        assert_eq!(AdfParser::constant("c(f)").unwrap().1, Formula::Bot);
        assert_eq!(format!("{:?}", Formula::Top), "Const(T)");
        assert_eq!(format!("{:?}", Formula::Bot), "Const(B)");
    }

    #[test]
    fn sort_updates() {
        let parser = AdfParser::default();
        let input = "s(a).s(c).ac(a,b).ac(b,neg(a)).s(b).ac(c,and(c(v),or(c(f),a))).";

        parser.parse()(input).unwrap();
        assert_eq!(parser.dict_value("a"), Some(0));
        assert_eq!(parser.dict_value("b"), Some(2));
        assert_eq!(parser.dict_value("c"), Some(1));

        parser.varsort_lexi();

        assert_eq!(parser.dict_value("a"), Some(0));
        assert_eq!(parser.dict_value("b"), Some(1));
        assert_eq!(parser.dict_value("c"), Some(2));

        let parser = AdfParser::default();
        let input = "s(a2).s(0).s(1).s(2).s(10).s(11).s(20).ac(0,c(v)).ac(1,c(v)).ac(2,c(v)).ac(10,c(v)).ac(20,c(v)).ac(11,c(v)).ac(a2,c(f)).";

        parser.parse()(input).unwrap();
        assert_eq!(parser.dict_value("a2"), Some(0));
        assert_eq!(parser.dict_value("0"), Some(1));
        assert_eq!(parser.dict_value("1"), Some(2));
        assert_eq!(parser.dict_value("2"), Some(3));
        assert_eq!(parser.dict_value("10"), Some(4));
        assert_eq!(parser.dict_value("11"), Some(5));
        assert_eq!(parser.dict_value("20"), Some(6));

        parser.varsort_lexi();
        assert_eq!(parser.dict_value("0"), Some(0));
        assert_eq!(parser.dict_value("1"), Some(1));
        assert_eq!(parser.dict_value("2"), Some(4));
        assert_eq!(parser.dict_value("10"), Some(2));
        assert_eq!(parser.dict_value("11"), Some(3));
        assert_eq!(parser.dict_value("20"), Some(5));
        assert_eq!(parser.dict_value("a2"), Some(6));

        parser.varsort_alphanum();
        assert_eq!(parser.dict_value("0"), Some(0));
        assert_eq!(parser.dict_value("1"), Some(1));
        assert_eq!(parser.dict_value("2"), Some(2));
        assert_eq!(parser.dict_value("10"), Some(3));
        assert_eq!(parser.dict_value("11"), Some(4));
        assert_eq!(parser.dict_value("20"), Some(5));
        assert_eq!(parser.dict_value("a2"), Some(6));
    }
}
