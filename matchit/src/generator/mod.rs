pub mod adapter;
pub mod reducer;
pub mod generator;
pub mod groupby;


#[cfg(test)]
mod tests {
    use super::*;
    use reducer::*;
    use adapter::*;
    use generator::*;

    #[test]
    fn fold_generator() {
        const N: u32 = 255;
        let mut fold = Generator::fold(0u32, |acc, _:u32| acc+1);
        for a in 0..N {
            fold.next(a);
        }
        assert_eq!(N, fold.finalize());
    }

    #[test]
    fn fold_adapter() {
        const N: u32 = 255;
        const MIN: u32 = 120;
        let filter = Generator::filter(|next: &u32| *next < MIN);
        let mut fold = filter.fold(0u32, |acc, _:u32| acc+1);
        for a in 0..N {
            fold.next(a);
        }
        assert_eq!(MIN, fold.finalize());
    }

    #[test]
    fn filter_generator() {
        const N: u32 = 255;
        const MIN: u32 = 120;
        let filter = Generator::filter(|next: &u32| *next < MIN);
        let mut fold = filter.count();

        for a in 0..N {
            fold.next(a);
        }
        assert_eq!(MIN, fold.finalize() as u32);
    }

    #[test]
    fn filter_adapter() {
        const N: u32 = 255;
        const MIN: f32 = 120.0f32;
        let map = Generator::map(|next: &u32| (*next as f32) / 2f32);
        let filter = map.filter(|next: &f32| *next < MIN);
        let mut fold = filter.count();

        for a in 0..N {
            fold.next(a);
        }
        assert_eq!((MIN as usize) * 2, fold.finalize());
    }


    #[test]
    fn map_generator() {
        const N: u32 = 10;
        let map = Generator::map(|next: &u32| (*next as f32) / 2f32 );
        let mut fold = map.fold(0f32, |acc: f32, next: f32| acc+next);

        for a in 0..N+1 {
            fold.next(a);
        }
        assert_eq!(N*(N+1)/4, fold.finalize() as u32);
    }

    #[test]
    fn map_adapter() {
        const N: u32 = 10;
        let filter = Generator::filter(|next: &u32| *next > 3 );
        let map = filter.map(|next: &u32| (*next as f32) / 2f32 );
        let mut fold = map.fold(0f32, |acc: f32, next: f32| acc+next);

        for a in 0..N+1 {
            fold.next(a);
        }
        assert_eq!(N*(N+1)/4 - 3, fold.finalize() as u32);
    }

    #[test]
    fn fork_adapter() {
        const N: u32 = 10;
        let map = Generator::map(|next: &u32| *next*2 );
        let mut fork = map.fork(
            Generator::map(|next: &u32| *next as f32 / 2f32).fold(0f32, |acc, next:f32| acc+(next as f32)),
            Generator::map(|next: &u32| (next * 2) as usize).sum());

        for a in 0..N+1 {
            fork.next(a);
        }
        let (res1, res2) = fork.finalize();
        assert_eq!(N*(N+1) / 2, res1 as u32);
        assert_eq!(N*(N+1) * 2, res2 as u32);
    }
    #[test]
    fn fork_generator() {
        const N: u32 = 10;
        let mut fork = Generator::fork(
            Generator::map(|next: &u32| *next as f32 / 2f32).fold(0f32, |acc, next:f32| acc+(next as f32)),
            Generator::map(|next: &u32| next * 2).fold(0u32, |acc, next| acc+next));

        for a in 0..N+1 {
            fork.next(a);
        }
        let (res1, res2) = fork.finalize();
        assert_eq!(N*(N+1) / 4, res1 as u32);
        assert_eq!(N*(N+1) , res2);
    }

    #[test]
    fn groupby_generator() {
        let groupby = Generator::groupby(|a: &u32, b: &u32| b >= a);
        let mut fold = groupby.fold(0u32, |acc: u32, _: (u32, u32)| acc+1);

        for a in [0,1,2,0,0,2,3,0,4,3] {
            fold.next(a);
        }
        assert_eq!(4, fold.finalize() as u32);
    }

    #[test]
    fn groupby_adapter() {
        let map = Generator::map(|next: &u32| *next * 2 );
        let groupby = map.groupby(|a: &u32, b: &u32| b >= a);
        let mut fold = groupby.fold(0u32, |acc: u32, _: (u32, u32)| acc+1);

        for a in [0,1,2,0,0,2,3,0,4,3] {
            fold.next(a);
        }
        assert_eq!(4, fold.finalize() as u32);
    }

    #[test]
    fn groupby_collect() {
        let predicate = |a: &usize, b: &usize| b >= a;
        let mapping = |r: &(usize,usize)| r.1 - r.0;

        let mut groupby = Generator::groupby(predicate).map(mapping).collect();

        for a in [0,1,2,0,0,2,3,0,4,3,4,4] {
            groupby.next(a);
        }
        println!("{:?}", groupby.finalize());
    }

    #[test]
    fn merge_generator() {
        let max = |a,b| {if a >= b { return a }; b};
        let union = |a: &(i32, i32), b: &(i32, i32)| {
            if b.0 >= a.0 && b.0 <= a.1 {
                Some((a.0, max(a.1, b.1)))
            } else if a.0 >= b.0 && a.0 <= b.1 {
                Some((b.0, max(a.1, b.1)))
            } else {
                None
            }
        };

        let mut merge =
            Generator::merge(union, |v: &(i32,i32)| *v).collect();

        for a in [(0,1),(1,1),(2,3),(0,0),(0,0),(2,2),(3,4),(0,0),(4,5),(3,6)] {
            merge.next(a);
        }
        println!("{:?}", merge.finalize());
        // assert_eq!(4, merge.finalize() as u32);
    }

    #[test]
    fn merge_adapter() {
        let max = |a,b| {if a >= b { return a }; b};
        let union = |a: &(i32, i32), b: &(i32, i32)| {
            if b.0 >= a.0 && b.0 <= a.1 {
                Some((a.0, max(a.1, b.1)))
            } else if a.0 >= b.0 && a.0 <= b.1 {
                Some((b.0, max(a.1, b.1)))
            } else {
                None
            }
        };

        let map = Generator::map(|a: &(i32, i32)| (a.0+1, a.1+1));
        let mut merge =
            map.merge(union, |v: &(i32,i32)| *v).collect();

        for a in [(0,1),(1,1),(2,3),(0,0),(0,0),(2,2),(3,4),(0,0),(4,5),(3,6)] {
            merge.next(a);
        }
        println!("{:?}", merge.finalize());
        // assert_eq!(4, merge.finalize() as u32);
    }

    #[test]
    fn split_generator() {
        let mut split = Generator::split(|a: &usize| *a, |_| Generator::sum());

        for a in [0,1,2,0,0,2,3,0,4,3,4,4] {
            split.next(a);
        }

        let mut fin = split.finalize();
        println!("{:?}", fin);
        assert_eq!(0usize, *fin.entry(0).or_default() );
        assert_eq!(1usize, *fin.entry(1).or_default() );
        assert_eq!(4usize, *fin.entry(2).or_default() );
        assert_eq!(6usize, *fin.entry(3).or_default() );
        assert_eq!(12usize, *fin.entry(4).or_default() );
    }

    #[test]
    fn split_adapter() {
        let map = Generator::map(|next: &usize| *next * 2 );
        let mut split = map.split(|a: &usize| *a, |_| Generator::groupby(|a:&usize, b:&usize| b >= a).count());

        for a in [0,1,2,0,0,2,3,0,4,3,4,4] {
            split.next(a);
        }

        let mut fin = split.finalize();
        println!("{:?}", fin);
        assert_eq!(1usize, *fin.entry(0).or_default() );
        assert_eq!(1usize, *fin.entry(2).or_default() );
        assert_eq!(1usize, *fin.entry(4).or_default() );
        assert_eq!(1usize, *fin.entry(6).or_default() );
        assert_eq!(1usize, *fin.entry(8).or_default() );
    }

    #[test]
    fn split_collect() {
        let predicate = |a: &usize, b: &usize| b >= a;
        let mapping = |r: &(usize,usize)| r.1 - r.0;

        let mut split = Generator::split(|a: &usize| *a, |_| Generator::groupby(predicate).map(mapping).collect());

        for a in [0,1,2,0,0,2,3,0,4,3,4,4] {
            split.next(a);
        }
        println!("{:?}", split.finalize());
    }


}
