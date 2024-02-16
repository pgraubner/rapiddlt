use std::collections::BTreeMap;
use std::ops::ControlFlow;
use std::{marker::PhantomData};

use super::adapter::{AdapterTrait};

pub trait ReducerTrait {
    type Input;
    type Reduced;

    fn next(&mut self, next: Self::Input);
    fn finalize(self) -> Self::Reduced;
}

pub struct Reducer<Impl,Adapter>(Impl,Adapter);
impl <Impl, Adapter> Reducer<Impl, Adapter> {
    pub fn new(i: Impl, a: Adapter) -> Self {
        Self (i, a)
    }
}

impl<Impl, A> ReducerTrait for Reducer<Impl,A>
where
    Impl: ReducerTrait<Input = A::Output>,
    A: AdapterTrait,
    {
    type Input = A::Input;
    type Reduced = Impl::Reduced;

    #[inline(always)]
    fn next(&mut self, next: Self::Input)
    {
        let out = self.1.adapt(next);
        if let ControlFlow::Continue(Some(o)) = out {
            self.0.next(o)
        }
    }

    #[inline(always)]
    fn finalize(mut self) -> Self::Reduced {
        let mut out = self.1.finalize();
        while out.is_continue() {
            if let ControlFlow::Continue(Some(o)) = out {
                self.0.next(o);
            }
            out = self.1.finalize();
        }

        self.0.finalize()
    }
}

pub struct Split<RedFn,KeyFn,I,R,Key,Red>(KeyFn, RedFn, BTreeMap<Key, Red>, PhantomData<I>, PhantomData<R>, PhantomData<Key>, PhantomData<Red>);
impl <RedFn,KeyFn,I,R,Key,Red> Split<RedFn,KeyFn,I,R,Key,Red> {
    pub fn new(keyfn: KeyFn, reducerfn: RedFn) -> Self {
        Self (keyfn, reducerfn, BTreeMap::new(), PhantomData, PhantomData, PhantomData, PhantomData)
    }
}

impl<RedFn,KeyFn,I,R,Key,Red> ReducerTrait for Split<RedFn,KeyFn,I,R,Key,Red>
where
    KeyFn: Fn(&I) -> Key,
    RedFn: Fn(&Key) -> Red,
    I: Clone, Key: Ord + Clone,
    Red: ReducerTrait<Input = I, Reduced = R>
{
    type Input = I;
    type Reduced = BTreeMap<Key, R>;

    #[inline(always)]
    fn next(&mut self, next: Self::Input) {
        let key = (self.0)(&next);
        let reducer = self.2.entry(key.clone()).or_insert((self.1)(&key));
        reducer.next(next);
    }

    #[inline(always)]
    fn finalize(self) -> Self::Reduced {
        let mut result = BTreeMap::new();
        for (k,v) in self.2 {
            result.insert(k, v.finalize());
        }
        result
    }
}


pub struct Fork<Impl1,Impl2,I>(Impl1, Impl2, PhantomData<I>);
impl <Impl1, Impl2,I> Fork<Impl1, Impl2,I> {
    pub fn new(i1: Impl1, i2: Impl2) -> Self {
        Self (i1, i2, PhantomData)
    }
}

impl<Impl1,Impl2,I> ReducerTrait for Fork<Impl1,Impl2,I>
where
    Impl1: ReducerTrait<Input = I>,
    Impl2: ReducerTrait<Input = I>,
    I: Clone
{
    type Input = I;
    type Reduced = (Impl1::Reduced,Impl2::Reduced);

    #[inline(always)]
    fn next(&mut self, next: Self::Input) {
        self.0.next(next.clone());
        self.1.next(next);
    }

    #[inline(always)]
    fn finalize(self) -> Self::Reduced {
        (self.0.finalize(), self.1.finalize())
    }
}

pub struct Fold<Init,F,I>(Init, F, PhantomData<I>);
impl <Init,F,I> Fold<Init,F,I> {
    pub fn new(init: Init, f: F) -> Self {
        Self (init, f, PhantomData)
    }
}

impl<Init,F,I> ReducerTrait for Fold<Init,F,I>
where
    F: FnMut(Init,I) -> Init,
    Init: Clone
{
    type Input = I;
    type Reduced = Init;

    #[inline(always)]
    fn next(&mut self, next: Self::Input) {
        self.0 = (self.1)(self.0.clone(),next);
    }

    #[inline(always)]
    fn finalize(self) -> Self::Reduced {
        self.0.clone()
    }
}
