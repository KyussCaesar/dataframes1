//! `gather`

use crate::traits::*;
use crate::dataframe::DataFrame;

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

impl<R: Record> DataFrame<R>
{
    /// Collect many columns into a key-value pair, repeating other values as neccessary.
    ///
    /// Use the `gather` macro to fill the arguments to this method.
    ///
    /// Small caveat; the columns being collected must all be of the same type.
    /// Furthermore, all of the fields have to already exist as Options
    ///
    /// ## Implementation Details
    ///
    /// If you're wondering what exactly the arguments do; `Collect` is
    /// responsible for selecting the values in each record that
    /// you want to collect into a single column, and `Update` is responsible
    /// for creating the new records with the appropriate new values.
    pub fn gather<Value, Collect, Update, Fields>(&self, item: (Collect, Update)) -> DataFrame<R>
    where
        Fields: Iterator<Item=(&'static str, Value)>,
        Collect: ThreadSafe + Fn(R) -> Fields,
        Update: ThreadSafe + Fn(&mut R, String, Value),
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
}

