use super::adf::Adf as NaiveAdf;
use super::adfbiodivine::*;
use super::datatypes::*;
use super::parser::*;
use test_log::test;

#[test]
fn adf_biodivine() {
    let parser = AdfParser::default();
    parser.parse()("s(a).s(b).s(c).s(d).ac(a,c(v)).ac(b,b).ac(c,and(a,b)).ac(d,neg(b)).\ns(e).ac(e,and(b,or(neg(b),c(f)))).s(f).\n\nac(f,xor(a,e)).")
            .unwrap();
    let adf = Adf::from_parser(&parser);
    let mut naive_adf = NaiveAdf::from_biodivine(&adf);

    let result = adf.grounded();

    assert_eq!(
        result,
        vec![
            Term::TOP,
            Term::UND,
            Term::UND,
            Term::UND,
            Term::BOT,
            Term::TOP
        ]
    );
    assert_eq!(
        format!("{}", adf.print_interpretation(&result)),
        "T(a) u(b) u(c) u(d) F(e) T(f) \n"
    );

    let naive_result = naive_adf.grounded();
    assert_eq!(
        format!("{}", adf.print_interpretation(&adf.grounded())),
        format!("{}", naive_adf.print_interpretation(&naive_result))
    );

    let parser = AdfParser::default();
    parser.parse()(
        "s(a).s(b).s(c).s(d).s(e).ac(a,c(v)).ac(b,a).ac(c,b).ac(d,neg(c)).ac(e,and(a,d)).",
    )
    .unwrap();
    let adf = Adf::from_parser(&parser);
    let mut naive_adf = NaiveAdf::from_biodivine(&adf);
    let result = adf.grounded();
    let naive_result = naive_adf.grounded();
    assert_eq!(
        result,
        vec![Term::TOP, Term::TOP, Term::TOP, Term::BOT, Term::BOT]
    );

    assert_eq!(
        format!("{}", adf.print_interpretation(&result)),
        format!("{}", naive_adf.print_interpretation(&naive_result))
    );
}
