use crate::matcher::{MatchType, TMatch};

#[derive(Debug)]
pub enum GroupByState<T> {
    Empty,
    Consumed(T),
    LazyReflexive(T),
    Lazy(MatchType<T>)
}

impl<T> GroupByState<T>
        where T: Copy
{
    #[inline(always)]
    fn current_match(&self) -> Option<MatchType<T>> {
        match *self {
            GroupByState::Empty => None,
            GroupByState::Consumed(a) => None,
            GroupByState::LazyReflexive(a) => Some((a,a)),
            GroupByState::Lazy((a, b)) => Some((a,b)),
        }
    }
}

#[derive(Debug)]
pub struct GroupBy<T, R> {
    state: GroupByState<T>,
    relation: R
}

impl<T, R> GroupBy<T, R>
        where T: Copy , R: FnMut(T,T) -> bool
{
    #[inline(always)]
    pub fn new(relation: R) -> Self where
        R: FnMut(T,T) -> bool,
    {
        GroupBy { relation, state: GroupByState::Empty }
    }

    #[inline(always)]
    fn from_next(&mut self, next: T) -> GroupByState<T> {
        if self.predicate(next, next) {
            GroupByState::LazyReflexive(next)
        }
        else {
            GroupByState::Consumed(next)
        }
    }

    #[inline(always)]
    fn predicate(&mut self, a:T, b:T) -> bool {
        (self.relation)(a,b)
    }

    #[inline(always)]
    fn change_state(&mut self, s: GroupByState<T>) {
        self.state = s;
    }

    #[inline(always)]
    fn state(&self) -> &GroupByState<T> {
        &self.state
    }

    #[inline(always)]
    fn transition(&mut self, o: GroupByState<T>) -> Option<MatchType<T>> {
        match o {
            GroupByState::Empty => {
                let res = self.state().current_match();
                self.change_state(GroupByState::Empty);
                res
            },
            GroupByState::Consumed(next) |
            GroupByState::LazyReflexive(next) => {
                match *self.state() {
                    GroupByState::Empty => {
                        let s = self.from_next(next);
                        self.change_state(s);
                        None
                    },
                    GroupByState::Lazy((begin, end)) => {
                        if self.predicate(end, next) {
                            self.change_state(GroupByState::Lazy((begin, next)));
                            None
                        } else {
                            let s = self.from_next(next);
                            self.change_state(s);
                            Some((begin, end))
                        }
                    }
                    GroupByState::Consumed(end) => {
                        if self.predicate(end, next) {
                            self.change_state(GroupByState::Lazy((end, next)));
                            None
                        } else {
                            let s = self.from_next(next);
                            self.change_state(s);
                            None
                        }
                    }
                    GroupByState::LazyReflexive(end) => {
                        if self.predicate(end, next) {
                            self.change_state(GroupByState::Lazy((end, next)));
                            None
                        } else {
                            let s = self.from_next(next);
                            self.change_state(s);
                            Some((end, end))
                        }
                    },
                }
            },
            GroupByState::Lazy((_, _)) => {
                todo!("merge two ranges")
            }
        }
    }

}

impl<T,R> TMatch<T> for GroupBy<T,R>
        where T: Copy , R: FnMut(T,T) -> bool {

    #[inline(always)]
    fn next(&mut self, next: T) -> Option<MatchType<T>> {
        let s = self.from_next(next);
        self.transition(s)
    }

    #[inline(always)]
    fn finalize(&mut self) -> Option<MatchType<T>> {
        self.transition(GroupByState::Empty)
    }

    #[inline(always)]
    fn is_empty(&mut self) -> bool {
        match self.state() {
            GroupByState::Empty => true,
            _ => false
        }
    }

}


#[cfg(test)]
mod tests {
    use crate::{matcher::TMatcherCall};

    use super::*;

    #[test]
    fn GroupBy_reflexive() {
        const N:usize = 2;
        let bytes = [0x0,0x0,0x1,0x2,0x3,0x4].repeat(N);
        let mat = bytes.iter().enumerate()
                .matches(GroupBy::new(|a: (usize, &i32),b: (usize, &i32)| b.1 >= a.1));

        let mut i = 0;
        for r in mat {
            i+=1;
            assert_eq!((r.0.1, r.1.1), (&0x0, &0x4));
            assert_eq!(5, r.1.0 - r.0.0);
            println!("match={:?}", (r.0, r.1));
        }
        assert_eq!(N, i);
    }

    #[test]
    fn GroupBy_greater() {
        const N:usize = 2;
        let bytes: Vec<i32> = [0x0,0x0,0x1,0x2,0x3,0x4].repeat(N);
        let mat = bytes.iter().enumerate()
                .matches(GroupBy::new(|a: (usize,&i32), b: (usize,&i32)| b.1 > a.1));

        let mut i = 0;
        for r in mat {
            i+=1;
            assert_eq!((r.0.1, r.1.1), (&0x0, &0x4));
            assert_eq!(4, r.1.0 - r.0.0);
            println!("match={:?}", (r.0, r.1));
        }
        assert_eq!(N, i);
    }

    #[test]
    fn GroupBy_count() {
        const N:usize = 10;
        let bytes = [0x0,0x1,0x2,0x3,0x4,0x5,0x0,0x1,0x2,0x3,0x4,0x5].repeat(N);
        let mat = bytes.as_slice().into_iter().matches(GroupBy::new(|a,b| a < b));

        let mut i = 0;
        for m in mat {
            i+=1;
            println!("match={:?}", m);
        }
        assert_eq!(N * 2, i);

    }

}