ADF-BDD.dev allows you to solve Abstract Dialectical Frameworks (ADFs). The ADFs are represented as Binary Decision Diagrams (BDDs).
The Web UI mimics many options of the CLI version of the [underlying adf-bdd tool](https://github.com/ellmau/adf-obdd). The syntax for the ADF code is indentical.

In the below form, you can either type/paste your `code` or upload a file in the same format.
To put it briefly, an ADF consists of statements and accectance conditions for these statements.
For instance, the following code indicates that `a,b,c,d` are statements, that `a` is assumed to be true (verum), `b` is true if `b` is true (which is self-supporting), `c` is true if `a` and `b` are true, and `d` is true if `b` is false.

```
s(a).
s(b).
s(c).
s(d).
ac(a,c(v)).
ac(b,b).
ac(c,and(a,b)).
ac(d,neg(b)).
```

Internally, the ADF is respresented as a BDD.
The `Parsing Strategy` determines the internal implementation used for these. `Naive` uses the own BDD implementation of our tool. `Hybrid` mixes our approaches with the existing Rust BDD library [`biodivine`](https://crates.io/crates/biodivine-lib-bdd). Don't be concerned about this choice if you are new to this tool; just pick either one.
You will get a view on the BDD in the detail view after you added the problem.

You can optionally set a name for you ADF problem. Otherwise a random name will be chosen. At the moment the name cannot be changed later (but you could remove and re-add the problem).
