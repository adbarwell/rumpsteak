use super::{Fsm, StateIndex, Transition};
use std::fmt::{self, Display, Formatter};

pub enum Type<R, L> {
    End,
    Recursion(usize),
    Variable(usize, Box<Self>),
    Transitions(Vec<(Transition<R, L>, Box<Self>)>),
}

impl<R: Clone, L: Clone> Type<R, L> {
    pub fn new(fsm: &Fsm<R, L>) -> Self {
        let size = fsm.size().0;
        assert!(size > 0);

        let mut builder = Builder {
            fsm,
            seen: &mut vec![false; size],
            looped: &mut vec![None; size],
            variables: &mut 0,
        };

        builder.build(Default::default())
    }
}

struct Builder<'a, R, L> {
    fsm: &'a Fsm<R, L>,
    seen: &'a mut Vec<bool>,
    looped: &'a mut Vec<Option<usize>>,
    variables: &'a mut usize,
}

impl<'a, R: Clone, L: Clone> Builder<'a, R, L> {
    fn variable(&mut self, state: StateIndex) -> usize {
        let variable = &mut self.looped[state.index()];
        match variable {
            Some(variable) => *variable,
            None => {
                let next = *self.variables;
                *variable = Some(next);
                *self.variables += 1;
                next
            }
        }
    }

    fn build(&mut self, state: StateIndex) -> Type<R, L> {
        if self.seen[state.index()] {
            return Type::Recursion(self.variable(state));
        }

        let mut transitions = self.fsm.transitions_from(state).peekable();
        if transitions.peek().is_none() {
            return Type::End;
        }

        self.seen[state.index()] = true;
        let transitions = transitions
            .map(|(to, transition)| (Transition::to_owned(&transition), Box::new(self.build(to))));
        let ty = Type::Transitions(transitions.collect());
        self.seen[state.index()] = false;

        if let Some(variable) = self.looped[state.index()].take() {
            return Type::Variable(variable, Box::new(ty));
        }

        ty
    }
}

pub struct Binary<'a, R, L>(&'a Type<R, L>);

impl<'a, R, L> Binary<'a, R, L> {
    pub fn new(ty: &'a Type<R, L>) -> Self {
        Self(ty)
    }
}

impl<'a, R, L: Display> Display for Binary<'a, R, L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.0 {
            Type::End => write!(f, "end"),
            Type::Recursion(variable) => write!(f, "X{}", variable),
            Type::Variable(variable, ty) => write!(f, "rec X{} . {}", variable, Binary(ty)),
            Type::Transitions(transitions) => {
                assert!(!transitions.is_empty());

                if let [transition] = transitions.as_slice() {
                    return write!(f, "{}", BinaryTransition::from(transition));
                }

                write!(f, "[{}", BinaryTransition::from(&transitions[0]))?;
                for transition in &transitions[1..] {
                    write!(f, ", {}", BinaryTransition::from(transition))?;
                }

                write!(f, "]")
            }
        }
    }
}

struct BinaryTransition<'a, R, L>(&'a Transition<R, L>, &'a Type<R, L>);

impl<'a, R, L> From<&'a (Transition<R, L>, Box<Type<R, L>>)> for BinaryTransition<'a, R, L> {
    fn from((transition, ty): &'a (Transition<R, L>, Box<Type<R, L>>)) -> Self {
        Self(transition, ty)
    }
}

impl<'a, R, L: Display> Display for BinaryTransition<'a, R, L> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}; {}", self.0.action, self.0.label, Binary(self.1))
    }
}
