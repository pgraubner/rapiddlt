use std::collections::{BTreeMap};

use super::{adapter::AdaptFnTrait, reducer::ReducerTrait};

#[derive(Clone)]
pub enum GroupByState<T>
where
    T: Clone
{
    Empty,
    Consumed(T),
    Lazy((T, T))
}

pub struct GroupBy<Input,F>(F, GroupByState<Input>)
where
    Input: Clone;
impl <Input,F> GroupBy<Input,F>
where
    Input: Clone
{
    pub fn new(f: F) -> Self {
        Self (f, GroupByState::Empty)
    }
}

impl<Input,F> AdaptFnTrait for GroupBy<Input,F>
where
    F: Fn(&Input, &Input) -> bool,
    Input: Clone
{
    type Input = Input;
    type Output = (Input, Input);

    #[inline(always)]
    fn adapt(&mut self, next: Self::Input) -> Option<Self::Output> {
        match self.1.clone() {
            GroupByState::Empty => {
                self.1 = GroupByState::Consumed(next);
                None
            },
            GroupByState::Consumed(ref prev) => {
                if (self.0)(prev, &next) {
                    self.1 = GroupByState::Lazy((prev.clone(), next));
                    None
                } else if (self.0)(prev, prev) {
                    self.1 = GroupByState::Consumed(next);
                    Some((prev.clone(), prev.clone()))
                } else {
                    None
                }
            },
            GroupByState::Lazy((ref start, ref stop)) => {
                if (self.0)(stop, &next) {
                    self.1 = GroupByState::Lazy((start.clone(), next));
                    None
                } else {
                    self.1 = GroupByState::Consumed(next);
                    Some((start.clone(),stop.clone()))
                }
            },
        }
    }

    #[inline(always)]
    fn finalize(&mut self) -> Option<Self::Output> {
        match self.1 {
            GroupByState::Empty => {
                None
            },
            GroupByState::Consumed(ref prev) => {
                if (self.0)(prev, prev) {
                    Some((prev.clone(), prev.clone()))
                } else {
                    None
                }
            },
            GroupByState::Lazy((ref start, ref stop)) => {
                Some((start.clone(),stop.clone()))
            },
        }
    }
}


pub struct Merge<Input,F,KeyFn,Key>(F, KeyFn, BTreeMap<Key,Input>)
where
    Input: Clone;
impl <Input,F,KeyFn,Key> Merge<Input,F,KeyFn,Key>
where
    Input: Clone
{
    pub fn new(f: F, keyfn: KeyFn) -> Self {
        Self (f, keyfn, BTreeMap::new())
    }
}

impl<Input,F,KeyFn,Key> ReducerTrait for Merge<Input,F,KeyFn,Key>
where
    KeyFn: Fn(&Input) -> Key,
    F: Fn(&Input, &Input) -> Option<Input>,
    Input: Clone,
    Key: Ord
{
    type Input = Input;
    type Reduced = BTreeMap<Key,Input>;

    #[inline(always)]
    fn next(&mut self, next: Self::Input) {
        let mut consumed = None;

        for (_, prev) in self.2.iter() {
            let ret = (self.0)(prev, &next);
            if let Some(val) = ret {
                consumed = Some((prev.clone(), val));
                break;
            }
        }
        if let Some((prev, val)) = consumed {
            self.2.remove(&(self.1)(&prev));
            self.2.insert((self.1)(&val), val);
        } else {
            self.2.insert((self.1)(&next), next);
        }
    }

    #[inline(always)]
    fn finalize(self) -> Self::Reduced {
        self.2
    }
}
