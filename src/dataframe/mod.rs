//! # Dataframe type.

// TODO: Document me!

use rayon::prelude::*;

pub use crate::traits::*;

/// Holds a collection of your `Records`.
pub struct DataFrame<R: Record>
{
    pub(crate) rows: Vec<R>,
}

impl<R: Record> DataFrame<R>
{
    /// Create an empty dataframe.
    pub fn new() -> Self
    {
        Self
        {
            rows: Vec::new(),
        }
    }

    /// Construct dataframe with the constructor, using values from the local
    /// environment.
    pub fn with<C: Fn() -> DataFrame<R>>(ctor: C) -> Self
    {
        ctor()
    }

    /// Appends the record to the end of this dataframe.
    pub fn push(&mut self, r: R)
    {
        self.rows.push(r);
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
    ///
    /// TODO: move this into it's own module, change predicate and transform into
    /// one arg as a tuple and use macros to fill with common ops like joins
    pub fn lookup<'a, 'b, Other, New, P, T, C>
    (
        &'a self,
        other: &'b DataFrame<Other>,
        predicate: P,
        transform: T,
        constructor: Option<C>
    ) -> DataFrame<New>
    where
        Other: Record,                     // The type of record stored in the other dataframe
        New:   Record,                     // The type of the result of this operation
        P:     Predicate<(&'a R, &'b Other)>,      // matches records between dataframes
        T:     Transform<(R, Other), New>, // how to create the output from the matches
        C:     Constructor<New>            // default value, if no match is found (optional)
    {
        DataFrame
        {
            rows: self.rows.par_iter()
                .filter_map(|s|
                {
                    // first find the match
                    if let Some(item) = other.rows.par_iter().find_first(|&p| predicate(&(s,p)))
                    {
                        // apply transform and return result
                        Some(transform((s.clone(), item.clone())))
                    }

                    // otherwise use ctor if provided
                    else if let Some(ref ctor) = constructor
                    {
                        Some(ctor())
                    }

                    // otherwise skip
                    else
                    {
                        None
                    }
                })
                .collect()
        }
    }

    /// General-purpose transformation method.
    /// Creates a copy of `self` with `op` applied to each row.
    pub fn transform<New: Record, T: Transform<R, New>>(&self, transform: T) -> DataFrame<New>
    {
        DataFrame
        {
            rows: self.rows.par_iter().cloned().map(transform).collect()
        }
    }
}

