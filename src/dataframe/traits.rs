//! Trait implementations for `DataFrame`.

use dataframe::DataFrame;
use traits::*;

use std::iter::Extend;

impl<R: Record> Extend<R> for DataFrame<R>
{
    fn extend<T: IntoIterator<Item=R>>(&mut self, iter: T)
    {
        for item in iter.into_iter()
        {
            self.rows.push(item);
        }
    }
}


