use std::{marker::PhantomData, ops::Add};

use super::{groupby::{GroupBy, Merge}, reducer::*};


/// An adapter is called either by a reducer or another adapter.
///
/// AdapterTrait::Input is the input provided by the reducer::next call.
/// AdapterTrait::Output is the output of an adapter after AdapterTrait::adapt() call.
///
/// Reducer.next(Reducer::Input) -> Adapter.adapt(Reducer::Input) ->  ... -> NilAdapter.adapt(Reducer::Input)
/// Reducer ... <- AdaptFn(Input=NilAdapter::Output, Output=Map|Filter.adapt()::Output) <- NilAdapter(Output=Reducer::Input)
///
///
///

pub trait AdaptFnTrait {
    type Input;
    type Output;

    fn adapt(&mut self, next: Self::Input) -> Option<Self::Output>;
    fn finalize(&mut self) -> Option<Self::Output>;
}

pub trait AdapterTrait
where
    Self: Sized
{
    type Input;
    type Output;

    fn adapt(&mut self, next: Self::Input) -> Option<Self::Output>;
    fn finalize(&mut self) -> Option<Self::Output>;

    #[inline(always)]
    fn split<RedFn,KeyFn,R,Key,Red>(self, keyfn: KeyFn, reducerfn: RedFn) -> Reducer<Split<RedFn,KeyFn,Self::Output,R,Key,Red>, Self>
    where
        KeyFn: Fn(&Self::Output) -> Key,
        RedFn: Fn(&Key) -> Red,
        Self::Output: Clone, Key: Ord + Clone,
        Red: ReducerTrait<Input = Self::Output, Reduced = R>
    {
        Reducer::new ( Split::new (keyfn, reducerfn), self )
    }

    #[inline(always)]
    fn fold<Init,Input,F>(self, init: Init, f: F) -> Reducer<Fold<Init,F,Input>,Self>
    where
        F: FnMut(Init, Input) -> Init,
    {
        Reducer::new ( Fold::new (init,f), self )
    }

    #[inline(always)]
    fn fork<R1,R2>(self, r1: R1, r2: R2) -> Reducer<Fork<R1, R2, Self::Input>, Self>
    where
        R1: ReducerTrait<Input = Self::Output>,
        R2: ReducerTrait<Input = Self::Output>
    {
        Reducer::new ( Fork::new( r1, r2 ), self )
    }

    #[inline(always)]
    fn filter<F>(self, f: F) -> Adapter<Filter<Self::Output,F>, Self>
    where
        F: Fn(&Self::Output) -> bool
    {
        Adapter::new( Filter::new(f), self )
    }

    #[inline(always)]
    fn map<Output,F>(self, f: F) -> Adapter<Map<Self::Output,Output,F>, Self>
    where
        F: Fn(&Self::Output) -> Output
    {
        Adapter::new( Map::new(f), self )
    }

    #[inline(always)]
    fn groupby<F>(self, f: F) -> Adapter<GroupBy<Self::Output,F>, Self>
    where
        F: Fn(&Self::Output, &Self::Output) -> bool,
        Self::Output: Clone
    {
        Adapter::new( GroupBy::new(f), self )
    }


    #[inline(always)]
    fn merge<F,KeyFn,Key>(self,f: F, keyfn: KeyFn) -> Reducer<Merge<Self::Output,F,KeyFn,Key>, Self>
    where
        KeyFn: Fn(&Self::Output) -> Key,
        F: Fn(&Self::Output, &Self::Output) -> Option<Self::Output>,
        Self::Output: Clone,
        Key: Ord
    {
        Reducer::new( Merge::new(f, keyfn), self )
    }

}

pub struct NilAdapter<I>(PhantomData<I>);
impl<I> NilAdapter<I> {
    pub fn new() -> Self {
        Self (PhantomData)
    }
}

impl<I> Default for NilAdapter<I> {
    fn default() -> Self {
        Self::new()
    }
}

impl<I> AdapterTrait for NilAdapter<I> {
    type Input = I;
    type Output = I;

    #[inline(always)]
    fn adapt(&mut self, next: Self::Input) -> Option<Self::Output> {
        Some(next)
    }

    #[inline(always)]
    fn finalize(&mut self) -> Option<Self::Output> {
        None
    }
}

pub struct Adapter<Impl, A>(Impl, A);
impl <Impl,A> Adapter<Impl,A> {
    pub fn new(i: Impl, a: A) -> Self {
        Self (i, a)
    }
}

impl<Impl, A> Adapter<Impl, A>
where
    Self: AdapterTrait
{
    #[inline(always)]
    pub fn count<Input>(self) -> Reducer<Fold<usize, impl FnMut(usize, Input) -> usize, Input>, Self>
    {
        self.fold(0usize, |acc: usize, _ | acc + 1)
    }

    #[inline(always)]
    pub fn sum<Input>(self) -> Reducer<Fold<usize, impl FnMut(usize, Input) -> usize, Input>, Self>
    where
        Input: Add<usize, Output = usize> + Clone + Copy
    {
        self.fold(0usize, |acc: usize, next: Input | next.add(acc) )
    }

    #[inline(always)]
    pub fn collect<Input>(self) -> Reducer<Fold<Vec<Input>, impl FnMut(Vec<Input>, Input) -> Vec<Input>, Input>, Self>
    where
        Input: std::fmt::Debug
    {
        self.fold(vec![], |mut acc: Vec<Input>, next: Input| {acc.push(next); acc})
    }

}

impl<Impl, A> AdapterTrait for Adapter<Impl, A>
where
    A: AdapterTrait,
    Impl: AdaptFnTrait<Input = A::Output>
{
    type Input = A::Input;
    type Output = Impl::Output;

    #[inline(always)]
    fn adapt(&mut self, next: Self::Input) -> Option<Self::Output> {
        let out = self.1.adapt(next)?;
        self.0.adapt(out)
    }

    #[inline(always)]
    fn finalize(&mut self) -> Option<Self::Output> {
        let out = self.1.finalize();
        let result = self.0.finalize();
        if let Some(o) = out {
            self.0.adapt(o)
        } else {
            result
        }

    }
}

pub struct Filter<Input,F>(F,PhantomData<Input>);
impl <Input,F> Filter<Input,F> {
    pub fn new(f: F) -> Self {
        Self (f, PhantomData)
    }
}

impl<Input,F> AdaptFnTrait for Filter<Input,F>
where
    F: Fn(&Input) -> bool
{
    type Input = Input;
    type Output = Input;

    #[inline(always)]
    fn adapt(&mut self, next: Self::Input) -> Option<Self::Output> {
        if (self.0)(&next) {
            Some(next)
        } else {
            None
        }
    }

    #[inline(always)]
    fn finalize(&mut self) -> Option<Self::Output> {
        None
    }
}

pub struct Map<Input,Output,F>(F,PhantomData<Input>, PhantomData<Output>);
impl <Input,Output,F> Map<Input,Output,F> {
    pub fn new(f: F) -> Self {
        Self (f, PhantomData, PhantomData)
    }
}
impl<Input,Output,F> AdaptFnTrait for Map<Input,Output,F>
where
    F: Fn(&Input) -> Output
{
    type Input = Input;
    type Output = Output;

    #[inline(always)]
    fn adapt(&mut self, next: Self::Input) -> Option<Self::Output> {
        Some((self.0)(&next))
    }

    #[inline(always)]
    fn finalize(&mut self) -> Option<Self::Output> {
        None
    }
}
