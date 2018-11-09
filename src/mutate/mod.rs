//! `mutate`

use rayon::prelude::*;

use crate::traits::*;
use crate::dataframe::DataFrame;

impl<R: Record> DataFrame<R>
{
    /// Create or alter columns using `Mutation`.
    pub fn mutate<M: Mutation<R>>(&self, mutation: M) -> DataFrame<R>
    {
        self.transform(|mut r| { mutation(&mut r); r })
    }
}
