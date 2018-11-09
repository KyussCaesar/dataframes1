extern crate csv;
extern crate serde;
extern crate prettytable;
extern crate rayon;

#[macro_use]
extern crate serde_derive;

use std::iter::Extend;

pub mod dataframe;
pub use dataframe::DataFrame;
pub use dataframe::traits::*;

pub mod select;
pub mod mutate;
pub mod filter;
pub mod gather;
// pub mod spread;

/// Here, we define some pseudo-trait aliases just to make things
/// a little easier.
pub mod traits
{
    /// Alias for `Send + Sync`
    pub trait ThreadSafe: Send + Sync {}
    impl<T: Send + Sync> ThreadSafe for T {}

    /// This trait is used as a bound on the type being stored in the dataframe.
    pub trait Record: Clone + ThreadSafe {}
    impl<T: Clone + ThreadSafe> Record for T {}

    /// Represents a function that transforms records from one type into another.
    pub trait Transform<R: Record, N: Record>: ThreadSafe + Fn(R) -> N {}
    impl<R: Record, N: Record, T: ThreadSafe + Fn(R) -> N> Transform<R, N> for T {}

    /// Represents a function that mutates records in place.
    pub trait Mutation<R: Record>: ThreadSafe + Fn(&mut R) {}
    impl<R: Record, T: ThreadSafe + Fn(&mut R)> Mutation<R> for T {}

    /// Represents a function that applies a predicate to a record.
    pub trait Predicate<R: Record>: ThreadSafe + Fn(&R) -> bool {}
    impl<R: Record, T: ThreadSafe + Fn(&R) -> bool> Predicate<R> for T {}

    /// Represents a function that creates a new value
    pub trait Constructor<R: Record>: ThreadSafe + Fn() -> R {}
    impl<R: Record, T: ThreadSafe + Fn() -> R> Constructor<R> for T {}
}

#[cfg(test)]
mod test
{
    #[test]
    fn test()
    {
        #[derive(Debug, Clone)]
        struct Record
        {
            id: usize,
            foo: f32,
            name: String,
        }

        type DataFrame = super::dataframe::DataFrame<Record>;

        let mut df = DataFrame::new();
        df.extend([
            Record { id: 32 as usize, foo: 3.43, name: "name".to_string() },
            Record { id: 1  as usize, foo: 6.54, name: "nrme".to_string() },
            Record { id: 2  as usize, foo: 9.66, name: "nlme".to_string() },
            Record { id: 3  as usize, foo: 0.25, name: "nfme".to_string() },
            Record { id: 4  as usize, foo: 2.29, name: "naoe".to_string() },
            Record { id: 5  as usize, foo: 1.74, name: "nase".to_string() },
            Record { id: 6  as usize, foo: 5.49, name: "name".to_string() },
            Record { id: 7  as usize, foo: 6.30, name: "naye".to_string() },
            Record { id: 8  as usize, foo: 7.72, name: "nace".to_string() },
            Record { id: 11 as usize, foo: 8.81, name: "name".to_string() },
            Record { id: 21 as usize, foo: 9.96, name: "nvme".to_string() }
        ].into_iter().cloned());

        println!("{:?}", df);

        println!("{:?}", df.mutate(|r: &mut Record| r.foo = 2.0*r.foo));
        println!("{:?}", df.filter(|r: &Record| r.name == "name"));
        println!("{:?}", df.filter(|r: &Record| r.foo > 3.0));
    }

    // #[test]
    // fn test_gather()
    // {
    //     #[derive(Debug, Clone)]
    //     struct Record
    //     {
    //         id: usize,
    //         bar: Option<i32>,
    //         baz: Option<i32>,
    //         qux: Option<i32>,
    //         key: Option<String>,
    //         val: Option<i32>,
    //     }

    //     #[derive(Debug, Clone)]
    //     struct RecordOut
    //     {
    //         id: usize,
    //         key: String,
    //         val: i32,
    //     }

    //     impl From<Record> for RecordOut
    //     {
    //         fn from(r: Record) -> Self
    //         {
    //             Self
    //             {
    //                 id: r.id,
    //                 key: r.key.unwrap(),
    //                 val: r.val.unwrap(),
    //             }
    //         }
    //     }

    //     type DataFrame = super::dataframe::DataFrame<Record>;

    //     let mut df = DataFrame::new();
    //     df.extend([
    //         Record { id: 32 as usize , bar: Some(3352) , baz: Some(2732) , qux: Some(6103) , key: None, val: None } ,
    //         Record { id: 1  as usize , bar: Some(12518), baz: Some(19259), qux: Some(24792), key: None, val: None } ,
    //         Record { id: 2  as usize , bar: Some(6824) , baz: Some(6241) , qux: Some(213)  , key: None, val: None } ,
    //         Record { id: 3  as usize , bar: Some(20251), baz: Some(2748) , qux: Some(9298) , key: None, val: None } ,
    //         Record { id: 4  as usize , bar: Some(26986), baz: Some(32432), qux: Some(23360), key: None, val: None } ,
    //         Record { id: 5  as usize , bar: Some(11932), baz: Some(30413), qux: Some(32029), key: None, val: None } ,
    //         Record { id: 6  as usize , bar: Some(29390), baz: Some(5429) , qux: Some(1462) , key: None, val: None } ,
    //         Record { id: 7  as usize , bar: Some(26701), baz: Some(19303), qux: Some(16651), key: None, val: None } ,
    //         Record { id: 8  as usize , bar: Some(9272) , baz: Some(6790) , qux: Some(19905), key: None, val: None } ,
    //         Record { id: 11 as usize , bar: Some(32092), baz: Some(23556), qux: Some(29983), key: None, val: None } ,
    //         Record { id: 21 as usize , bar: Some(26206), baz: Some(5959) , qux: Some(9391) , key: None, val: None }
    //     ].into_iter().cloned());

    //     use super::gather::gather;

    //     println!("{:?}", df);
    //     println!("{:?}", df.gather(gather!(key, val, bar, baz, qux)).transform(RecordOut::from));
    // }
}

