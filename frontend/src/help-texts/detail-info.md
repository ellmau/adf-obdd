First of all you can review the code that you added. You can also delete the problem if you made a mistake or do not need it anymore.

Further below, you can have a look at the BDD representations of your problem using different semantics.
In principle, each statement gets it's own BDD that indicates how its truth value can be obtained from the other ones. Note that every BDD has the `BOT` and `TOP` nodes ultimately indicating the truth value (false or true respectively).
All these individual BDDs are displayed in a merged representation where the `Root for:` labels tell you where to start looking if you want to 
get the truth value of an individual statement.
For instance, consider a BDD that (besides `BOT` and `TOP`) only contains a node `b` annotated with `Root for: a` and the annotation `Root for: b` at the `TOP` node.
Since the root for `b` is the `TOP` node, we know that `b` must be true. Then, to obtain the truth value for `a`, we start at the `b` and since we know that `b` must be true, we can follow the blue edge to obtain the value for `a` (we will end up in `BOT` or `TOP` there). If `b` would be false, we would follow the orange edge analogously. Note that is not always possible to directly determine the truth values of statements (which is exactly why we need tools like this).

On the very left, you can view the initial representation of your problem after parsing. This also indicates the parsing strategy that you have chosen (`Naive` or `Hybrid`).
The other tabs allow you to solve the problem using different semantics and optimizations. Some of them (e.g. `complete`) may produce multiple models that you can cycle through.
To get a better idea of the differences, you can have a look at the [command line tool](https://github.com/ellmau/adf-obdd/tree/main/bin).

