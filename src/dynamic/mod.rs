//! DataFrame: Dynamic.
//!
//! Everything checked at runtime.

use std::collections::HashMap;
use std::iter::Extend;
use self::error_string::ErrorString;

/// Represents a column in the dataframe.
#[derive(Debug, Clone)]
enum Column
{
    Float(Vec<f32>),
    Double(Vec<f64>),
    Factor(Vec<String>),
    Bool(Vec<bool>),
}

impl Column
{
    /// Return the number of elements in the column.
    fn len(&self) -> usize
    {
        use self::Column::*;
        match self
        {
            &Float(ref v)  => v.len(),
            &Double(ref v) => v.len(),
            &Factor(ref v) => v.len(),
            &Bool(ref v)   => v.len(),
        }
    }

    /// Return the type of this column, as a str.
    fn variant_str(&self) -> &'static str
    {
        use self::Column::*;
        match self
        {
            &Float(_)  => "Float",
            &Double(_) => "Double",
            &Factor(_) => "Factor",
            &Bool(_)   => "Boolean",
        }
    }
}

/// A dataframe.
#[derive(Clone)]
pub struct DataFrame
{
    columns: HashMap<String, Column>,
    nrow: usize,
}

/// Errors.
///
/// # TODOs
///
/// Replace uses of the general error with more specific ones.
#[derive(Debug)]
pub enum Error
{
    /// General error.
    General(&'static str),

    /// General error (owned string).
    GeneralBuf(String),
}

impl Error
{
    /// Convert self into a Result::Err.
    pub fn err<T>(self) -> Result<T>
    {
        Err(self)
    }

    pub fn print(&self)
    {
        println!("{:?}", self);
    }
}

/// Result type.
pub type Result<T> = ::std::result::Result<T, Error>;

impl DataFrame
{
    /// Create a new DataFrame.
    pub fn new() -> Self
    {
        Self::default()
    }

    /// Checks that this dataframe and `other` are compatible for `cbind`.
    fn cbind_ck(&self, other: &DataFrame) -> Result<()>
    {
        if self.nrow != other.nrow
        {
            return Error::General("Cannot cbind dataframes with differing number of rows.").err();
        }

        for key in other.columns.keys()
        {
            if self.columns.contains_key(key)
            {
                // TODO suggest to user to use `join` instead.
                return Error::General("Cannot cbind dataframes with conflicting column names.").err();
            }
        }

        Ok(())
    }

    /// Concatenate columns of two dataframes.
    pub fn cbind(&self, other: &DataFrame) -> Result<Self>
    {
        self.cbind_ck(other)?;

        let mut result = DataFrame::new();
        result.nrow = self.nrow;

        for (key, val) in self.columns.iter()
        {
            result.columns.insert(key.clone(), val.clone());
        }

        for (key, val) in other.columns.iter()
        {
            result.columns.insert(key.clone(), val.clone());
        }

        Ok(result)
    }

    /// Concatenate other dataframe onto `self`.
    pub fn cbind_mut(&mut self, other: &DataFrame) -> Result<&mut Self>
    {
        self.cbind_ck(other)?;

        // not sure if this will actually help at all but whatever.
        self.columns.reserve(other.columns.len());

        for (key, val) in other.columns.iter()
        {
            self.columns.insert(key.clone(), val.clone());
        }

        Ok(self)
    }

    fn rbind_ck_base(a: &DataFrame, b: &DataFrame) -> Result<()>
    {
        for key in a.columns.keys()
        {
            if !b.columns.contains_key(key)
            {
                return ErrorString::from("Column `")
                    .p(key)
                    .p("` is present in self but not in other")
                    .err();
            }

            use self::Column::*;
            match (&a.columns[key], &b.columns[key])
            {
                (&Float(_) , &Float(_))  => continue,
                (&Double(_), &Double(_)) => continue,
                (&Factor(_), &Factor(_)) => continue,
                (&Bool(_)  , &Bool(_))   => continue,

                _ => return ErrorString::new()
                    .p("Column `")
                    .p(key)
                    .p("` is a `")
                    .p(a.columns[key].variant_str())
                    .p("` in self, but a `")
                    .p(b.columns[key].variant_str())
                    .p("` in other.")
                    .err()
            }
        }

        Ok(())
    }

    fn rbind_ck(&self, other: &DataFrame) -> Result<()>
    {
        DataFrame::rbind_ck_base(self, other)?;
        DataFrame::rbind_ck_base(other, self)?;

        // should always succeed
        assert!(self.columns.len() == other.columns.len());

        Ok(())
    }

    /// Concatenate rows of two dataframes.
    pub fn rbind(&self, other: &DataFrame) -> Result<Self>
    {
        self.rbind_ck(other)?;

        let mut result = DataFrame::new();
        result.nrow = self.nrow + other.nrow;

        for (key, val) in self.columns.iter()
        {
            let mut vec: Column = val.clone();

            use self::Column::*;
            match (&mut vec, &other.columns[key]) 
            {
                (&mut Float(ref mut v) , Float(ref o))  => v.extend(o.iter()),
                (&mut Double(ref mut v), Double(ref o)) => v.extend(o.iter()),
                (&mut Factor(ref mut v), Factor(ref o)) => v.extend(o.iter().cloned()),
                (&mut Bool(ref mut v)  , Bool(ref o))   => v.extend(o.iter()),

                // rbind_ck already checked that this is not the case.
                _ => panic!("invalid rbind"),
            }

            result.columns.insert(key.clone(), vec);
        }

        Ok(result)
    }

    /// Concatenate rows of other onto self.
    pub fn rbind_mut(&mut self, other: &DataFrame) -> Result<&mut Self>
    {
        self.rbind_ck(other)?;

        for (key, val) in self.columns.iter_mut()
        {
            use self::Column::*;
            match (val, &other.columns[key]) 
            {
                (Float(ref mut v) , &Float(ref o))  => v.extend(o.iter()),
                (Double(ref mut v), &Double(ref o)) => v.extend(o.iter()),
                (Factor(ref mut v), &Factor(ref o)) => v.extend(o.iter().cloned()),
                (Bool(ref mut v)  , &Bool(ref o))   => v.extend(o.iter()),

                // rbind_ck already checked that this is not the case.
                _ => panic!("invalid rbind"),
            }
        }

        self.nrow += other.nrow;
        Ok(self)
    }

    fn select_ck(&self, columns: &[&str]) -> Result<()>
    {
        for col in columns
        {
            if !self.columns.contains_key(*col)
            {
                return ErrorString::from("Column `").p(col).p("` is not present in dataframe.").err();
            }
        }

        Ok(())
    }

    /// Return a new dataframe with a subset of the columns in `self`.
    pub fn select(&self, columns: &[&str]) -> Result<Self>
    {
        self.select_ck(columns)?;

        let mut result = Self::default();
        for col in columns
        {
            result.columns.insert(col.to_string(), self.columns[*col].clone());
        }

        result.nrow = self.nrow;
        Ok(result)
    }

    /// Remove columns from `self` that are not in `columns`.
    pub fn select_mut(&mut self, columns: &[&str]) -> Result<&mut Self>
    {
        self.select_ck(columns)?;

        for col in columns
        {
            self.columns.remove(*col);
        }

        Ok(self)
    }

    /// Create a new dataframe with columns which satisfy the predicate.
    pub fn filter<F: FnOnce(DfToken) -> bool>(&self, p: F) -> Result<Self>
    {
        ErrorString::from("unimplemented").err()
    }
}

impl Default for DataFrame
{
    fn default() -> Self
    {
        Self
        {
            columns: HashMap::default(),
            nrow: 0 as usize,
        }
    }
}

pub mod error_string
{
    use super::*;

    pub struct ErrorString
    {
        s: String
    }

    impl ErrorString
    {
        pub fn new() -> Self
        {
            Self
            {
                s: String::new(),
            }
        }

        /// `p`, short for `paste`. Extends string with the argument.
        pub fn p(mut self, ss: &str) -> Self
        {
            self.s.extend(ss.chars());
            self
        }

        /// Convert self into Result::Err
        pub fn err<T>(self) -> Result<T>
        {
            Error::GeneralBuf(self.s).err()
        }
    }

    impl Default for ErrorString
    {
        fn default() -> Self
        {
            Self
            {
                s: String::default(),
            }
        }
    }

    impl Into<String> for ErrorString
    {
        fn into(self) -> String
        {
            self.s
        }
    }

    impl From<String> for ErrorString
    {
        fn from(s: String) -> Self
        {
            Self
            {
                s
            }
        }
    }

    impl<'a> From<&'a str> for ErrorString
    {
        fn from(ss: &'a str) -> Self
        {
            Self
            {
                s: String::from(ss)
            }
        }
    }

    impl Into<Error> for ErrorString
    {
        fn into(self) -> Error
        {
            Error::GeneralBuf(self.s)
        }
    }
}

