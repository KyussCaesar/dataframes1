/// DataFrame type.
///
/// Holds a collection of your `Records`.
#[derive(Debug)]
pub struct DataFrame<Record: Clone>
{
    rows: Vec<Record>,
}

impl<Record: Clone> DataFrame<Record>
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

    /// Return subset of the columns in `self`.
    pub fn select
        <NewRecord: Clone,
        Selector: Fn(Record) -> NewRecord>
        (&self, s: Selector) -> DataFrame<NewRecord>
    {
        DataFrame
        {
            rows: self.rows.iter().cloned().map(s).collect()
        }
    }

    /// Return subset of rows in `self` which satisfy predicate against `other`.
    pub fn lookup
        <OtherRecord: Clone,
        Predicate: Fn(&Record, &OtherRecord) -> bool>
        (&self, other: &DataFrame<OtherRecord>, p: Predicate) -> DataFrame<Record>
    {
        DataFrame
        {
            rows: self.rows.iter()
                .zip(other.rows.iter())
                .filter_map(|(s,o)|
                {
                    if p(s, o) { Some(s.clone()) }
                    else { None }
                })
                .collect()
        }
    }

    /// Same as `lookup`, but additionally, perform an operation to produce
    /// the new rows.
    pub fn lookup_then<OtherRecord, NewRecord, Predicate, Operation>
    (
        &self,
        other: &DataFrame<OtherRecord>,
        p: Predicate,
        op: Operation
    ) -> DataFrame<NewRecord>
    where
        OtherRecord: Clone,
        NewRecord:   Clone,
        Predicate:   Fn(&Record, &OtherRecord) -> bool,
        Operation:   Fn(&Record, &OtherRecord) -> NewRecord,
    {
        DataFrame
        {
            rows: self.rows.iter()
                .zip(other.rows.iter())
                .filter_map(|(s,o)|
                {
                    if p(s, o) { Some(op(s, o)) }
                    else { None }
                })
                .collect()
        }
    }

    /// Return subset of rows in `other` which satisfy predicate against `self`.
    pub fn reverse_lookup
        <OtherRecord: Clone,
        Predicate: Fn(&OtherRecord, &Record) -> bool>
        (&self, other: &DataFrame<OtherRecord>, p: Predicate) -> DataFrame<OtherRecord>
    {
        other.lookup(self, p)
    }

    /// Create or alter records using `Op`.
    pub fn mutate<NewRecord: Clone, Op: Fn(Record) -> NewRecord>
        (&self, op: Op) -> DataFrame<NewRecord>
    {
        DataFrame
        {
            rows: self.rows.iter().cloned().map(op).collect()
        }
    }

    /// Return subset of rows that satisfy predicate.
    pub fn filter<Predicate: Fn(&Record) -> bool>(&self, p: Predicate) -> DataFrame<Record>
    {
        DataFrame
        {
            rows: self.rows.iter().filter(|&i| p(i)).cloned().collect()
        }
    }
}

impl<Record: Clone> Extend<Record> for DataFrame<Record>
{
    fn extend<T: IntoIterator<Item=Record>>(&mut self, iter: T)
    {
        for item in iter.into_iter()
        {
            self.rows.push(item);
        }
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

        println!("{:?}", df.mutate(|mut r| { r.foo = 2.0*r.foo; r }))
    }
}

