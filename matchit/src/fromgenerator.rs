use std::{collections::BTreeMap, ops::ControlFlow};

use crate::generator::{adapter::AdaptFnTrait, groupby::GroupBy, reducer::{ReducerTrait, Split}};

pub struct FromAdaptFn<AdaptFn, Iter>(AdaptFn, Iter, bool);
impl <AdaptFn, Iter> FromAdaptFn<AdaptFn, Iter> {
    pub fn new(a: AdaptFn, iter: Iter) -> Self {
        Self (a,iter, true)
    }
}

impl<AdaptFn, Iter> Iterator for FromAdaptFn<AdaptFn, Iter>
where
    Iter: Iterator,
    AdaptFn: AdaptFnTrait<Input = Iter::Item>,
    Iter::Item: std::fmt::Debug
{
    type Item = AdaptFn::Output;

    fn next(&mut self) -> Option<Self::Item> {
        for i in self.1.by_ref() {
            let result = self.0.adapt(i);
            if let ControlFlow::Continue(Some(res)) = result {
                return Some(res);
            } else if let ControlFlow::Break(()) = result {
                break;
            }
        }

        let mut out = self.0.finalize();
        while out.is_continue() {
            if let ControlFlow::Continue(Some(o)) = out {
                return Some(o);
            }
            out = self.0.finalize();
        }
        None
    }
}

pub trait FromAdaptFnCall: Iterator
        where Self: Sized {

    fn groupby<F>(self, f: F) -> FromAdaptFn<GroupBy<Self::Item, F>, Self>
    where
        Self::Item: Copy
    {
        FromAdaptFn::new( GroupBy::new(f), self )
    }

    fn split<RedFn,KeyFn,R,Key,Red>(mut self, keyfn: KeyFn, reducerfn: RedFn) -> BTreeMap<Key, R>
    where
        KeyFn: Fn(&Self::Item) -> Key,
        RedFn: Fn(&Key) -> Red,
        Self::Item: Copy,
        Key: Ord + Clone,
        Red: ReducerTrait<Input = Self::Item, Reduced = R>
    {
        let mut split: Split<RedFn,KeyFn, Self::Item, R, Key, Red> = Split::new(keyfn, reducerfn);

        for i in self.by_ref() {
            split.next(i);
        }
        split.finalize()
    }

}

impl<T> FromAdaptFnCall for T where T: Iterator {}

mod tests {
    #[allow(unused_imports)]
    use crate::fromgenerator::FromAdaptFnCall;


    #[test]
    fn groupby() {
        let data = [0,1,2,0,0,2,3,0,4,3];
        assert_eq!(4, data.into_iter().groupby(|a: &u32, b: &u32| b >= a).count());
    }

}