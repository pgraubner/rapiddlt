use std::marker::PhantomData;
use std::ops::Add;

use super::groupby::GroupBy;
use super::groupby::Merge;
use super::reducer::*;
use super::adapter::*;

pub struct Generator<Input>(PhantomData<Input>);
impl<Input> Generator<Input> {

    #[inline(always)]
    pub fn split<RedFn,KeyFn,R,Key,Red>(keyfn: KeyFn, reducerfn: RedFn) -> Reducer<Split<RedFn,KeyFn,Input,R,Key,Red>,NilAdapter<Input>>
    where
        KeyFn: Fn(&Input) -> Key,
        RedFn: Fn(&Key) -> Red,
        Input: Clone, Key: Ord + Clone,
        Red: ReducerTrait<Input = Input, Reduced = R>
    {
        Reducer::new ( Split::new (keyfn, reducerfn), NilAdapter::new() )
    }

    #[inline(always)]
    pub fn fold<Init,F>(init: Init, f: F) -> Reducer<Fold<Init,F,Input>,NilAdapter<Input>>
    where
        F: FnMut(Init, Input) -> Init,
    {
        Reducer::new ( Fold::new (init,f), NilAdapter::new() )
    }

    #[inline(always)]
    pub fn count() -> Reducer<Fold<usize, impl FnMut(usize, Input) -> usize, Input>, NilAdapter<Input>>
    {
        Generator::fold(0usize, |acc: usize, _ | acc + 1)
    }

    #[inline(always)]
    pub fn sum() -> Reducer<Fold<usize, impl FnMut(usize, Input) -> usize, Input>, NilAdapter<Input>>
    where
        Input: Add<usize, Output = usize> + Clone + Copy
    {
        Generator::fold(0usize, |acc: usize, next: Input | next.add(acc) )
    }


    #[inline(always)]
    pub fn fork<R1,R2>(r1: R1, r2: R2) -> Reducer<Fork<R1, R2, Input>, NilAdapter<Input>>
    where
        R1: ReducerTrait<Input = Input>,
        R2: ReducerTrait<Input = Input>
    {
        Reducer::new( Fork::new( r1, r2 ), NilAdapter::new() )
    }

    #[inline(always)]
    pub fn filter<F>(f: F) -> Adapter<Filter<Input,F>, NilAdapter<Input>>
    where
        F: Fn(&Input) -> bool
    {
        Adapter::new( Filter::new(f), NilAdapter::new() )
    }

    #[inline(always)]
    pub fn map<Output,F>(f: F) -> Adapter<Map<Input,Output,F>, NilAdapter<Input>>
    where
        F: Fn(&Input) -> Output
    {
        Adapter::new( Map::new(f), NilAdapter::new() )
    }

    #[inline(always)]
    pub fn groupby<F>(f: F) -> Adapter<GroupBy<Input,F>, NilAdapter<Input>>
    where
        F: Fn(&Input, &Input) -> bool,
        Input: Clone
    {
        Adapter::new( GroupBy::new(f), NilAdapter::new() )
    }

    #[inline(always)]
    pub fn merge<F,KeyFn,Key>(f: F, keyfn: KeyFn) -> Adapter<Merge<Input,F,KeyFn,Key>, NilAdapter<Input>>
    where
        KeyFn: Fn(&Input) -> Key,
        F: Fn(&Input, &Input) -> Option<Input>,
        Input: Clone,
        Key: Ord
    {
        Adapter::new( Merge::new(f, keyfn), NilAdapter::new() )
    }
}

