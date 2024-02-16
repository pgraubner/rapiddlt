use std::{collections::BTreeMap, ops::ControlFlow};

use super::adapter::{AdaptFnTrait};

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
    fn adapt(&mut self, next: Self::Input) -> ControlFlow<(), Option<Self::Output>> {
        match self.1.clone() {
            GroupByState::Empty => {
                self.1 = GroupByState::Consumed(next);
                ControlFlow::Continue(None)
            },
            GroupByState::Consumed(ref prev) => {
                if (self.0)(prev, &next) {
                    self.1 = GroupByState::Lazy((prev.clone(), next));
                    ControlFlow::Continue(None)
                } else if (self.0)(prev, prev) {
                    self.1 = GroupByState::Consumed(next);
                    ControlFlow::Continue(Some((prev.clone(), prev.clone())))
                } else {
                    ControlFlow::Continue(None)
                }
            },
            GroupByState::Lazy((ref start, ref stop)) => {
                if (self.0)(stop, &next) {
                    self.1 = GroupByState::Lazy((start.clone(), next));
                    ControlFlow::Continue(None)
                } else {
                    self.1 = GroupByState::Consumed(next);
                    ControlFlow::Continue(Some((start.clone(),stop.clone())))
                }
            },
        }
    }

    #[inline(always)]
    fn finalize(&mut self) -> ControlFlow<(), Option<Self::Output>> {
        match self.1.clone() {
            GroupByState::Empty => {
                ControlFlow::Break(())
            },
            GroupByState::Consumed(ref prev) => {
                self.1 = GroupByState::Empty;
                if (self.0)(prev, prev) {
                    ControlFlow::Continue(Some((prev.clone(), prev.clone())))
                } else {
                    ControlFlow::Break(())
                }
            },
            GroupByState::Lazy((ref start, ref stop)) => {
                self.1 = GroupByState::Empty;
                ControlFlow::Continue(Some((start.clone(),stop.clone())))
            },
        }
    }
}


pub struct Merge<Input,F,KeyFn,Key>(F, KeyFn, BTreeMap<Key,Input>, Option<AdapterIterator<<BTreeMap<Key, Input> as IntoIterator>::IntoIter>>)
where
    Input: Clone;
impl <Input,F,KeyFn,Key> Merge<Input,F,KeyFn,Key>
where
    Input: Clone
{
    pub fn new(f: F, keyfn: KeyFn) -> Self {
        Self (f, keyfn, BTreeMap::new(), None)
    }
}

struct AdapterIterator<It>(It);

impl<Input,F,KeyFn,Key> AdaptFnTrait for Merge<Input,F,KeyFn,Key>
where
    KeyFn: Fn(&Input) -> Key,
    F: Fn(&Input, &Input) -> Option<Input>,
    Input: Clone,
    Key: Ord + Clone
{
    type Input = Input;
    type Output = Input;

    #[inline(always)]
    fn finalize(&mut self) -> ControlFlow<(), Option<Self::Output>> {
        if self.3.is_none() {
            self.3 = Some(AdapterIterator(self.2.clone().into_iter()));
        }
        if let Some(ref mut iter) = self.3 {
            for (_,v) in iter.0.by_ref() {
                return ControlFlow::Continue(Some(v))
            }
        }
        ControlFlow::Break(())
    }


    #[inline(always)]
    fn adapt(&mut self, next: Self::Input) -> ControlFlow<(), Option<Self::Output>> {
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
        ControlFlow::Continue(None)
    }
}
