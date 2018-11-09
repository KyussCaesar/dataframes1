//! `filter`

use rayon::prelude::*;

use crate::traits::*;
use crate::dataframe::DataFrame;

impl<R: Record> DataFrame<R>
{
    /// Return subset of rows that satisfy predicate.
    pub fn filter<P: Predicate<R>>(&self, p: P) -> DataFrame<R>
    {
        DataFrame
        {
            rows: self.rows.par_iter()
                .filter(|&i| p(i))
                .cloned() // only clone if it satisfies predicate
                .collect()
        }
    }
}
