mod lib
{
    /// DataFrame type.
    ///
    /// Holds a collection of your `Records`.
    struct DataFrame<Record>
    {
        rows: Vec<Record>,
    }

    impl<Record> DataFrame<Record>
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
        fn with<C: Fn() -> DataFrame<Record>>(ctor: C) -> Self
        {
            f()
        }

        /// Return subset of the columns in `self`.
        fn select<NewRecord, Selector: Fn(Record) -> NewRecord>(&self, s: Selector) -> DataFrame<NewRecord>
        {
            DataFrame<NewRecord>
            {
                rows: self.rows.iter().cloned().map(s).collect()
            }
        }

        /// Return subset of rows in `self` which satisfy predicate against `other`.
        fn lookup
            <OtherRecord,
            Predicate: Fn(Record, OtherRecord) -> bool>
            (&self, other: &DataFrame<OtherRecord>, p: Predicate) -> DataFrame<Record>
        {
            DataFrame<Record>
            {
                rows: self.rows.iter()
                    .cloned()
                    .zip(other.rows.iter().cloned())
                    .filter(|(s,o)| p(s, o))
                    .collect()
            }
        }

        /// Same as `lookup`, but additionally, perform an operation to produce
        /// the new rows.
        fn lookup_then
            <OtherRecord,
            NewRecord,
            Operation: Fn(Record, OtherRecord) -> Option<NewRecord>>
            (&self, other: &DataFrame<OtherRecord>, op: Operation) -> DataFrame<NewRecord>
        {
            DataFrame<Record>
            {
                rows: self.rows.iter()
                    .cloned()
                    .zip(other.rows.iter().cloned())
                    .filter_map(|(s,o)| p(s, o))
                    .collect()
            }
        }

        /// Return subset of rows in `other` which satisfy predicate against `self`.
        fn reverse_lookup
            <OtherRecord,
            Predicate: Fn(Record, OtherRecord) -> bool>
            (&self, other: &DataFrame<OtherRecord>, p: Predicate) -> DataFrame<OtherRecord>
        {
            other.lookup(self, p)
        }

        fn mutate<NewRecord, Op: Fn(Record) -> NewRecord>(&self, op: Op) -> DataFrame<NewRecord>
        {
            DataFrame<NewRecord>
            {
                rows: self.rows.iter().cloned().map(op).collect()
            }
        }

        fn filter<Predicate: Fn(Record) -> bool>(&self, p: Predicate) -> DataFrame<Record>
        {
            DataFrame<Record>
            {
                rows: self.rows.iter().cloned().filter(p).collect()
            }
        }
    }

    impl<Record> Extend<Record> for DataFrame<Record>
    {
        fn extend<T: IntoIterator<Item=Record>>(&mut self, iter: T)
        {
            self.rows.extend(iter)
        }
    }
}

mod client
{
    use super::lib;

    struct Record
    {
        id: usize,
        foo: f32,
        name: String,
    }

    type DataFrame = lib::DataFrame<Record>;

    fn main()
    {
        let mut df = DataFrame::new();
        df.extend(&[
            Record { id: 32 as usize, foo: 3.14, name: "name".to_string()
        );

        println!("{}", df);

        df.mutate(Record2::from)

        println!("{}", df);
    }
}

fn main()
{
    client::main()
}

