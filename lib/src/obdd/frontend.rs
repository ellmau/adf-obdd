//! Implementation of frontend-feature related methods and functions
//! See the Structs in the [obdd-module][super] for most of the implementations

use crate::datatypes::Term;

use super::BddNode;
impl super::Bdd {
    /// Instantiate a new [roBDD][super::Bdd] structure.
    /// Constants for the [`⊤`][crate::datatypes::Term::TOP] and [`⊥`][crate::datatypes::Term::BOT] concepts are prepared in that step too.
    /// # Attention
    /// Constants for [`⊤`][crate::datatypes::Term::TOP] and [`⊥`][crate::datatypes::Term::BOT] concepts are not sent, as they are considered to be existing in every [Bdd][super::Bdd] structure.
    pub fn with_sender(sender: crossbeam_channel::Sender<BddNode>) -> Self {
        // TODO nicer handling of the initialisation though overhead is not an issue here
        let mut result = Self::new();
        result.set_sender(sender);
        result
    }

    /// Instantiate a new [roBDD][super::Bdd] structure.
    /// Constants for the [`⊤`][crate::datatypes::Term::TOP] and [`⊥`][crate::datatypes::Term::BOT] concepts are prepared in that step too.
    /// # Attention
    /// Note that mixing manipulating operations and utilising the communication channel for a receiving [roBDD][super::Bdd] may end up in inconsistent data.
    /// So far, only manipulate the [roBDD][super::Bdd] if no further [recv][Self::recv] will be called.
    pub fn with_receiver(receiver: crossbeam_channel::Receiver<BddNode>) -> Self {
        // TODO nicer handling of the initialisation though overhead is not an issue here
        let mut result = Self::new();
        result.set_receiver(receiver);
        result
    }

    /// Instantiate a new [roBDD][super::Bdd] structure.
    /// Constants for the [`⊤`][crate::datatypes::Term::TOP] and [`⊥`][crate::datatypes::Term::BOT] concepts are prepared in that step too.
    /// # Attention
    /// - Constants for [`⊤`][crate::datatypes::Term::TOP] and [`⊥`][crate::datatypes::Term::BOT] concepts are not sent, as they are considered to be existing in every [Bdd][super::Bdd] structure.
    /// - Mixing manipulating operations and utilising the communication channel for a receiving [roBDD][super::Bdd] may end up in inconsistent data.
    ///
    /// So far, only manipulate the [roBDD][super::Bdd] if no further [recv][Self::recv] will be called.
    pub fn with_sender_receiver(
        sender: crossbeam_channel::Sender<BddNode>,
        receiver: crossbeam_channel::Receiver<BddNode>,
    ) -> Self {
        let mut result = Self::new();
        result.set_receiver(receiver);
        result.set_sender(sender);
        result
    }

    /// Updates the currently used [sender][crossbeam_channel::Sender]
    pub fn set_sender(&mut self, sender: crossbeam_channel::Sender<BddNode>) {
        self.sender = Some(sender);
    }

    /// Updates the currently used [receiver][crossbeam_channel::Receiver]
    pub fn set_receiver(&mut self, receiver: crossbeam_channel::Receiver<BddNode>) {
        self.receiver = Some(receiver);
    }

    /// Receives all information till the looked for [`Term`][crate::datatypes::Term] is either found or all data is read.
    /// Note that the values are read, consumed, and added to the [Bdd][super::Bdd].
    /// # Returns
    /// - [`true`] if the [term][crate::datatypes::Term] is found (either in the [Bdd][super::Bdd] or in the channel.
    /// - [`false`] if neither the [Bdd][super::Bdd] nor the channel contains the [term][crate::datatypes::Term].
    pub fn recv(&mut self, term: Term) -> bool {
        if term.value() < self.nodes.len() {
            true
        } else if let Some(recv) = &self.receiver {
            loop {
                match recv.try_recv() {
                    Ok(node) => {
                        let new_term = Term(self.nodes.len());
                        self.nodes.push(node);
                        self.cache.insert(node, new_term);
                        if let Some(send) = &self.sender {
                            match send.send(node) {
                                Ok(_) => log::trace!("Sent {node} to the channel."),
                                Err(e) => {
                                    log::error!(
                                        "Error {e} occurred when sending {node} to {:?}",
                                        send
                                    )
                                }
                            }
                        }
                        if new_term == term {
                            return true;
                        }
                    }
                    Err(_) => return false,
                }
            }
        } else {
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::super::*;

    #[test]
    fn get_bdd_updates() {
        let (send, recv) = crossbeam_channel::unbounded();
        let mut bdd = Bdd::with_sender(send);

        let solving = std::thread::spawn(move || {
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
        });

        let updates: Vec<BddNode> = recv.iter().collect();
        assert_eq!(
            updates,
            vec![
                BddNode::new(Var(0), Term(0), Term(1)),
                BddNode::new(Var(1), Term(0), Term(1)),
                BddNode::new(Var(0), Term(0), Term(3)),
                BddNode::new(Var(1), Term(1), Term(0)),
                BddNode::new(Var(0), Term(1), Term(5)),
                BddNode::new(Var(2), Term(0), Term(1)),
                BddNode::new(Var(2), Term(1), Term(0)),
                BddNode::new(Var(1), Term(0), Term(7)),
                BddNode::new(Var(0), Term(3), Term(1)),
                BddNode::new(Var(0), Term(0), Term(9)),
            ]
        );
        solving.join().expect("Both threads should terminate");
    }

    #[test]
    fn recv_send() {
        let (send1, recv1) = crossbeam_channel::unbounded();
        let (send2, recv2) = crossbeam_channel::unbounded();
        let mut bdd1 = Bdd::with_sender(send1);
        let mut bddm = Bdd::with_sender_receiver(send2, recv1);
        let mut bddl = Bdd::with_receiver(recv2);

        let solving = std::thread::spawn(move || {
            let v1 = bdd1.variable(Var(0));
            let v2 = bdd1.variable(Var(1));

            assert_eq!(v1, Term(2));
            assert_eq!(v2, Term(3));

            let t1 = bdd1.and(v1, v2);
            let nt1 = bdd1.not(t1);
            let ft = bdd1.or(v1, nt1);

            assert_eq!(ft, Term::TOP);

            let v3 = bdd1.variable(Var(2));
            let nv3 = bdd1.not(v3);
            assert_eq!(bdd1.and(v3, nv3), Term::BOT);

            let conj = bdd1.and(v1, v2);
            assert_eq!(bdd1.restrict(conj, Var(0), false), Term::BOT);
            assert_eq!(bdd1.restrict(conj, Var(0), true), v2);

            let a = bdd1.and(v3, v2);
            let b = bdd1.or(v2, v1);

            let con1 = bdd1.and(a, conj);

            let end = bdd1.or(con1, b);
            log::debug!("Restrict test: restrict({},{},false)", end, Var(1));
            let x = bdd1.restrict(end, Var(1), false);
            assert_eq!(x, Term(2));
        });
        // allow the worker to fill the channels
        std::thread::sleep(std::time::Duration::from_millis(10));
        // both are initialised, no updates so far
        assert_eq!(bddm.nodes, bddl.nodes);
        // receiving a truth constant should work without changing the bdd
        assert!(bddm.recv(Term::TOP));
        assert_eq!(bddm.nodes, bddl.nodes);
        // receiving some element works for middle -> last, but not last -> middle
        assert!(bddm.recv(Term(2)));
        assert!(bddl.recv(Term(2)));
        assert_eq!(bddl.nodes.len(), 3);
        assert!(!bddl.recv(Term(5)));
        // get all elements into middle bdd1
        assert!(!bddm.recv(Term(usize::MAX)));

        assert_eq!(
            bddm.nodes,
            vec![
                BddNode::bot_node(),
                BddNode::top_node(),
                BddNode::new(Var(0), Term(0), Term(1)),
                BddNode::new(Var(1), Term(0), Term(1)),
                BddNode::new(Var(0), Term(0), Term(3)),
                BddNode::new(Var(1), Term(1), Term(0)),
                BddNode::new(Var(0), Term(1), Term(5)),
                BddNode::new(Var(2), Term(0), Term(1)),
                BddNode::new(Var(2), Term(1), Term(0)),
                BddNode::new(Var(1), Term(0), Term(7)),
                BddNode::new(Var(0), Term(3), Term(1)),
                BddNode::new(Var(0), Term(0), Term(9)),
            ]
        );

        // last bdd is still in the previous state
        assert_eq!(
            bddl.nodes,
            vec![
                BddNode::bot_node(),
                BddNode::top_node(),
                BddNode::new(Var(0), Term(0), Term(1)),
            ]
        );

        // and now catch up till 10
        assert!(bddl.recv(Term(10)));

        assert_eq!(
            bddl.nodes,
            vec![
                BddNode::bot_node(),
                BddNode::top_node(),
                BddNode::new(Var(0), Term(0), Term(1)),
                BddNode::new(Var(1), Term(0), Term(1)),
                BddNode::new(Var(0), Term(0), Term(3)),
                BddNode::new(Var(1), Term(1), Term(0)),
                BddNode::new(Var(0), Term(1), Term(5)),
                BddNode::new(Var(2), Term(0), Term(1)),
                BddNode::new(Var(2), Term(1), Term(0)),
                BddNode::new(Var(1), Term(0), Term(7)),
                BddNode::new(Var(0), Term(3), Term(1)),
            ]
        );

        solving.join().expect("Both threads should terminate");

        // asking for 10 again works too
        assert!(bddl.recv(Term(10)));
        // fully catch up with the last bdd
        assert!(bddl.recv(Term(11)));
        assert_eq!(bddl.nodes, bddm.nodes);
    }
}
