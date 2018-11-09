//! `select`

use rayon::prelude::*;

use crate::traits::*;
use crate::dataframe::DataFrame;

macro_rules! select
{
    ( $typename:ident, $($item:ident),* ) =>
    {
        |r| $typename { $(r.$item)* }
    }
}

impl<R: Record> DataFrame<R>
{
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
    pub fn select<New: Record, T: Transform<R, New>>(&self, transform: T) -> DataFrame<New>
    {
        self.transform(transform)
    }
}
