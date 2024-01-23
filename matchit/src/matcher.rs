use std::ops::RangeInclusive;

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum MatchRange<T>
        where T: Copy {
    Single(T),
    Many(T,T)
}

impl<T> MatchRange<T>
        where T: Copy {
    #[inline(always)]
    pub fn start(self) -> T {
        match self {
            MatchRange::Single(a) => a,
            MatchRange::Many(a, _b) => a,
        }
    }

    #[inline(always)]
    pub fn end(self) -> T {
        match self {
            MatchRange::Single(a) => a,
            MatchRange::Many(_a, b) => b,
        }
    }
}

impl<T> Into<RangeInclusive<T>> for MatchRange<T>
        where T: Copy {
    fn into(self) -> RangeInclusive<T> {
        match self {
            MatchRange::Single(a) => todo!(),
            MatchRange::Many(a, b) => a..=b,
        }
    }
}

impl<T> From<(T, T)> for MatchRange<T>
            where T: Copy {
    fn from(range: (T, T)) -> Self {
        MatchRange::Many( range.0, range.1 )
    }
}
impl<T> From<T> for MatchRange<T>
        where T: Copy {

    fn from(val: T) -> Self {
        MatchRange::Single( val )
    }
}

pub type MatchType<T> = (T,T);

#[derive(Debug)]
pub struct Matcher<I, M> where I: Iterator, M : TMatch<I::Item>, I::Item: Copy {
    pub iter: I,
    matcher: M,
}

impl<I,M> Matcher<I,M>
        where I: Iterator, M : TMatch<I::Item>, I::Item: Copy
{
    pub fn new(iter: I, matcher: M) -> Self {
        Matcher { iter, matcher }
    }
}

impl<I, M> Iterator for Matcher<I, M>
where
    I: Iterator, M : TMatch<I::Item>, I::Item: Copy
{
    type Item = MatchType<I::Item>;

    fn next(&mut self) -> Option<MatchType<I::Item>> {
        for next in self.iter.by_ref() {
            let result = self.matcher.next(next);
            if result.is_some() {
                return result;
            }
        }
        if !self.matcher.is_empty() {
            self.matcher.finalize()
        } else {
            None
        }
    }

}
pub trait TMatcherCall: Iterator
        where Self: Sized {

    fn matches<M>(self, m: M) -> Matcher<Self, M>
            where M: TMatch<Self::Item>, Self::Item: Copy
    {
        Matcher::new(self, m)
    }
}

impl<T> TMatcherCall for T where T: Iterator {}
pub trait TMatch<T>
        where T:Copy {
    fn next(&mut self, next: T) -> Option<MatchType<T>>;
    fn finalize(&mut self) -> Option<MatchType<T>>;
    fn is_empty(&mut self) -> bool;
}
