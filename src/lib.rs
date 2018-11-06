extern crate csv;
extern crate serde;
extern crate prettytable;
extern crate rayon;

#[macro_use]
extern crate serde_derive;

use rayon::prelude::*;

pub mod traits
{
    pub trait ThreadSafe: Send + Sync {}
    impl<T: Send + Sync> ThreadSafe for T {}

    pub trait Record<'a>: Clone + ThreadSafe + serde::Serialize + serde::Deserialize<'a> {}
    impl<'a, T: Clone + ThreadSafe + serde::Serialize + serde::Deserialize<'a>> Record<'a> for T {}
}

/// DataFrame type.
///
/// Holds a collection of your `Records`.
pub struct DataFrame<'a, Record: traits::Record<'a>>
{
    rows: Vec<Record>,
}

impl<Record: traits::Record<'a>> DataFrame<Record>
{
    /// Create an empty dataframe.
    pub fn new() -> Self
    {
        Self
        {
            rows: Vec::new(),
        }
    }

    /// Construct dataframe with the constructor.
    pub fn with<C: Fn() -> DataFrame<Record>>(ctor: C) -> Self
    {
        ctor()
    }

    /// Find rows in `self` and `other` which satisfy a predicate, then
    /// perform some action with the matches.
    ///
    /// For example,
    ///
    /// ```rust,ignore
    /// data.lookup(other,
    ///     |d,o| d.id == o.id,
    ///     |d,o| { let d = d.clone(); d.bar = Some(o.foo.clone()); d }
    /// )
    /// ```
    ///
    /// would match up the `id` field in `data` and `other`, and populate the 
    /// `bar` field of `d` with a copy of the `foo` field in `o`.
    pub fn lookup<OtherRecord, NewRecord, Predicate, Operation, Default>
    (
        &self,
        other: &DataFrame<OtherRecord>,
        p: Predicate,
        op: Operation,
        default: Default
    ) -> DataFrame<NewRecord>
    where
        OtherRecord: traits::Record<'a>,
        NewRecord:   traits::Record<'a>,
        Predicate:   traits::ThreadSafe + Fn(&Record, &OtherRecord) -> bool,
        Operation:   traits::ThreadSafe + Fn(&Record, &OtherRecord) -> NewRecord,
        Default:     traits::ThreadSafe + Fn() -> NewRecord
    {
        DataFrame
        {
            rows: self.rows.par_iter()
                .map(|s|
                {
                    if let Some(item) = other.rows.par_iter().find_first(|o| p(s, o)) { op(s, item) }
                    else { default() }
                })
                .collect()
        }
    }

    /// Equivalent to `other.lookup(self...)`, that is, "return rows in `other`
    /// which satisfy predicate against `self`."
    pub fn reverse_lookup<OtherRecord, NewRecord, Predicate, Operation, Default>
    (
        &self,
        other: &DataFrame<OtherRecord>,
        p: Predicate,
        op: Operation,
        default: Default
    ) -> DataFrame<NewRecord>
    where
        OtherRecord: Clone + Send + Sync,
        NewRecord:   Clone + Send + Sync,
        Predicate:   Send + Sync + Fn(&Record, &OtherRecord) -> bool,
        Operation:   Send + Sync + Fn(&Record, &OtherRecord) -> NewRecord,
        Default:     Send + Sync + Fn() -> NewRecord
    {
        other.lookup(self, |s,o| p(o,s), |s,o| op(o,s), default)
    }

    /// General-purpose transformation method.
    /// Creates a copy of `self` with `op` applied to each row.
    pub fn transform<Operation, NewRecord>(&self, op: Operation) -> DataFrame<NewRecord>
    where
        NewRecord: Clone + Send + Sync,
        Operation: Send  + Sync + Fn(Record) -> NewRecord,
    {
        DataFrame
        {
            rows: self.rows.par_iter().cloned().map(op).collect()
        }
    }

    /// General-purpose transformation method.
    /// This version consumes self, and applies the transformation directly.
    pub fn transform_mut<Operation, NewRecord>(self, op: Operation) -> DataFrame<NewRecord>
    where
        NewRecord: Clone + Send + Sync,
        Operation: Send  + Sync + Fn(Record) -> NewRecord,
    {
        DataFrame
        {
            rows: self.rows.into_par_iter().map(op).collect()
        }
    }

    /// Return subset of the columns in `self`.
    ///
    /// In the general case, this method uses `op` to transform the input records
    /// (of type `Record`) into new records (of type `NewRecord`). Unfortunately,
    /// this leads to some clunky syntax:
    ///
    /// ```rust,ignore
    /// df.select(|r| NewRecord { id: r.id, foo: r.foo })
    /// ```
    ///
    /// If you `impl From<(Id, Foo, Bar)> for NewRecord`, then you can use
    /// something a little closer to the R syntax, shown below.
    ///
    /// ```rust,ignore
    /// df.select(|r| (r.id, r.foo, r.bar).into())
    /// ```
    //  TODO: Write macro to fill args for you.
    pub fn select<Operation, NewRecord>(&self, op: Operation) -> DataFrame<NewRecord>
    where
        NewRecord: Clone + Send + Sync,
        Operation: Send  + Sync + Fn(Record) -> NewRecord,
    {
        self.transform(|r| op(r).into())
    }

    /// Create or alter columns using `Op`.
    pub fn mutate<Operation>(&self, op: Operation) -> DataFrame<Record>
    where
        Operation: Send  + Sync + Fn(&mut Record),
    {
        self.transform(|mut r| {op(&mut r); r})
    }

    /// Collect many columns into a key-value pair, repeating other values as neccessary.
    pub fn gather<Value, Collect, Update, Fields>(&self, item: (Collect, Update)) -> DataFrame<Record>
    where
        Fields: Iterator<Item=(&'static str, Value)>,
        Collect: Send + Sync + Fn(Record) -> Fields,
        Update: Send + Sync + Fn(&mut Record, String, Value),
    {
        // Accepts a record and returns an iterable of `(key, value)` pairs,
        // where `key` is a `&'static str` representing the name of the field,
        // and `value` is the value of that field for this record.
        let collect = item.0;

        // Updates the record with the key-value pair.
        let update  = item.1;

        // stores the result
        let mut new_records = Vec::new();

        for record in self.rows.iter()
        {
            // gather the fields
            for collected in collect(record.clone())
            {
                let mut new_record = record.clone();
                let string = collected.0.to_string();
                let value  = collected.1;
                update(&mut new_record, string, value);
                new_records.push(new_record);
            }
        }

        DataFrame
        {
            rows: new_records,
        }
    }

    // /// Spread the values of one column into many columns
    // pub fn spread<Operation, NewRecord>(&self, op: Operation) -> DataFrame<NewRecord>
    // where
    //     NewRecord: Clone + Send + Sync,
    //     Operation: Send  + Sync + Fn(Record) -> NewRecord,
    // {
    //     self.transform(op)
    // }

    /// Return subset of rows that satisfy predicate.
    pub fn filter<Predicate: Send + Sync + Fn(&Record) -> bool>(&self, p: Predicate) -> DataFrame<Record>
    {
        DataFrame
        {
            rows: self.rows.par_iter().filter(|&i| p(i)).cloned().collect()
        }
    }

    /// Appends the record to the end of this dataframe.
    pub fn push(&mut self, r: Record)
    {
        self.rows.push(r);
    }
}

impl<Record: traits::Record<'a>> Extend<Record> for DataFrame<Record>
{
    fn extend<T: IntoIterator<Item=Record>>(&mut self, iter: T)
    {
        for item in iter.into_iter()
        {
            self.rows.push(item);
        }
    }
}

use std::fmt;
impl<Record: traits::Record<'a>> fmt::Debug for DataFrame<Record>
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        let mut wtr = csv::Writer::from_writer(vec![]);
        println!("DataFrame with {} records", self.rows.len());
        for rec in self.rows.iter()
        {
            wtr.serialize(rec.clone())?;
        }

        let table_s = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
        let table = prettytable::Table::from_csv_string(&table_s).unwrap();

        table.printstd();

        Ok(())
    }
}

/// Populates the required argument for `gather`.
macro_rules! gather
{
    ($key:ident, $val:ident, $( $col:ident ),*) =>
    {
        (
            |r| vec![ $( (stringify!($col), r.$col) ),* ].into_iter(),
            |r,cn,val|
            {
                $( r.$col = None; )*
                r.$key = Some(cn);
                r.$val = val;
            }
        )
    }
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

        type DataFrame = super::DataFrame<Record>;

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

        println!("{:?}", df.mutate(|r| r.foo = 2.0*r.foo));
        println!("{:?}", df.filter(|r| r.name == "name"));
        println!("{:?}", df.filter(|r| r.foo > 3.0));
    }

    #[test]
    fn test_gather()
    {
        #[derive(Debug, Clone)]
        struct Record
        {
            id: usize,
            bar: Option<i32>,
            baz: Option<i32>,
            qux: Option<i32>,
            key: Option<String>,
            val: Option<i32>,
        }

        #[derive(Debug, Clone)]
        struct RecordOut
        {
            id: usize,
            key: String,
            val: i32,
        }

        impl From<Record> for RecordOut
        {
            fn from(r: Record) -> Self
            {
                Self
                {
                    id: r.id,
                    key: r.key.unwrap(),
                    val: r.val.unwrap(),
                }
            }
        }

        type DataFrame = super::DataFrame<Record>;

        let mut df = DataFrame::new();
        df.extend([
            Record { id: 32 as usize , bar: Some(3352) , baz: Some(2732) , qux: Some(6103) , key: None, val: None } ,
            Record { id: 1  as usize , bar: Some(12518), baz: Some(19259), qux: Some(24792), key: None, val: None } ,
            Record { id: 2  as usize , bar: Some(6824) , baz: Some(6241) , qux: Some(213)  , key: None, val: None } ,
            Record { id: 3  as usize , bar: Some(20251), baz: Some(2748) , qux: Some(9298) , key: None, val: None } ,
            Record { id: 4  as usize , bar: Some(26986), baz: Some(32432), qux: Some(23360), key: None, val: None } ,
            Record { id: 5  as usize , bar: Some(11932), baz: Some(30413), qux: Some(32029), key: None, val: None } ,
            Record { id: 6  as usize , bar: Some(29390), baz: Some(5429) , qux: Some(1462) , key: None, val: None } ,
            Record { id: 7  as usize , bar: Some(26701), baz: Some(19303), qux: Some(16651), key: None, val: None } ,
            Record { id: 8  as usize , bar: Some(9272) , baz: Some(6790) , qux: Some(19905), key: None, val: None } ,
            Record { id: 11 as usize , bar: Some(32092), baz: Some(23556), qux: Some(29983), key: None, val: None } ,
            Record { id: 21 as usize , bar: Some(26206), baz: Some(5959) , qux: Some(9391) , key: None, val: None }
        ].into_iter().cloned());

        println!("{:?}", df);
        println!("{:?}", df.gather(gather!(key, val, bar, baz, qux)).transform(RecordOut::from));
    }

    use csv;
    use std;
    use prettytable;

    #[test]
    fn csv_test_print() -> csv::Result<()>
    {
        #[derive(Clone, Debug, Serialize, Deserialize)]
        struct Record
        {
            sepal_length : f32,
            sepal_width  : f32,
            petal_length : f32,
            petal_width  : f32,
            class        : String,
        }

        type DataFrame = super::DataFrame<Record>;

        let mut rdr = csv::Reader::from_path("tests/iris.csv")?;
        let mut df = DataFrame::new();
        for result in rdr.deserialize()
        {
            let record: Record = result?;
            df.push(record);
        }

        let mut wtr = csv::Writer::from_writer(vec![]);
        println!("DataFrame with {} records", df.rows.len());
        for rec in df.rows.iter()
        {
            wtr.serialize(rec.clone())?;
        }

        let table_s = String::from_utf8(wtr.into_inner().unwrap()).unwrap();
        let table = prettytable::Table::from_csv_string(&table_s).unwrap();

        table.printstd();

        Ok(())
    }
}

