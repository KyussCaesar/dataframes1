//! # Test module for `df` type.
//!
//! Or; an adventure in achieving feeling-parity with R in Rust.
//!
//! # Background
//!
//! When manipulating a dataframe in R with functions like `filter` and `mutate`
//! (from the `dplyr` package), you can do things like:
//!
//!    df %>%
//!        filter(Migratory == TRUE) %>%
//!        mutate(Error = Estimate - Actual)
//!
//! Writing things in this style is pretty nice, but it's only possible due to
//! features in R like lazy evaluation and argument quoting and probably other
//! stuff which I don't know the inner workings of.
//!
//! However, you can't do this in most other languages, because they will fully
//! evaluate the function arguments before calling the function. This applies to
//! Rust too.
//!
//! I want my dataframe type to feel as much like using a dataframe in R (with
//! the `dplyr` workflow) as possible.
//!
//! My approach is to implement the `filter` and `mutate` methods like so:
//!
//! ```rust
//! fn <F: FnOnce(df) -> df>(&self, f: F) -> Result<Self>;
//! ```
//!
//! Where `f` is some closure which defines what needs to be done. Essentially,
//! the user's closure will be passed a virtual "notepad", and is told to write
//! down what they want. The implementation of `filter` is then responsible for
//! checking that the instructions in the notebook are valid, and then executing
//! them.
//!
//! ## Why not pass in a dataframe directly and implement the `std::ops` traits?
//!
//! The traits all require the operation to be infallible, so if the user tries
//! to add two things that don't make sense (like say, a column of factor and a
//! column of numbers), I (as the library author) have no opportunity to bail
//! except to `panic!`. This is undesirable because it's inconsitent with the
//! other methods on dataframes (which thus far return `Result<Self>` and
//! `Result<&mut Self>` respectively), and, in addition to not giving me (as
//! the library author) the opportunity to bail with an error, also doesn't give
//! you (as the library user) that opportunity either.
//!
//! Instead, the library user writes down what they want on this "notepad", and
//! the library author checks that what you've written makes sense before
//! executing it.
//!
//! # How this works
//!
//! I define a type `df`:
//!
//! ```rust
//! struct df
//! {
//!     items: RefCell<Vec<Item>>,
//! }
//! ```
//!
//! This is the type of the value that will be passed to your closure by `filter`
//! and `mutate`.
//! It is a stack, with a little `RefCell` in between that we'll get to in a moment.
//! `Item` is simply an enum, to allow us to store many different things on the
//! stack.
//!
//! What we're going to do is store the requested operations in reverse polish
//! notation, using the `Vec` above as the stack.
//! 
//! For this demo, we'll just try to express adding two columns together, and comparing
//! the result to a constant.
//! We can do this with the following rules:
//!
//! * Indexing a `df` pushes the index onto the stack.
//! * Adding two borrowed `df`s pushes the `Plus` variant of the `Item` enum onto
//!   the stack.
//! * Comparing (via `PartialEq`) a `df` and a constant pushes the constant onto
//!   the stack, followed by the `Equals` variant of the `Item` enum.
//!
//! In other words, this lets us write this R code:
//!
//!     data %>%
//!         filter(Foo + Bar < 3)
//!
//! as the following Rust code:
//!
//! ```rust
//! data.filter(|df| &df["Foo"] + &df["Bar"] < 3)
//! ```
//!
//! ...ok, so it's not quite as clean as the R code, but still.

use std::cell::RefCell;
use std::ops::Index;
use std::ops::Add;

/// Container for each item on the stack.
#[derive(Debug)]
enum Item
{
    Add,
    OwnColumn(&'static str),
}

impl Clone for Item
{
    fn clone(&self) -> Self
    {
        match self
        {
            &Item::Add => Item::Add,
            &Item::OwnColumn(s) => Item::OwnColumn(s),
        }
    }
}

impl Copy for Item {}

/// A value of this type is passed into the user's closure by `filter` and `mutate`.
#[derive(Debug)]
pub struct df
{
    stack: RefCell<Vec<Item>>,
    stage: RefCell<Vec<Item>>,
}

impl df
{
    fn new() -> Self
    {
        Self
        {
            stack: RefCell::new(Vec::new()),
            stage: RefCell::new(Vec::new()),
        }
    }
}

impl Clone for df
{
    fn clone(&self) -> Self
    {
        Self
        {
            stack: RefCell::new(self.stack.borrow().clone()),
            stage: RefCell::new(self.stage.borrow().clone()),
        }
    }
}

// Vec does not implement Copy, because it is a pointer, length and capacity.
// Pointer could cause double-free or use-after-free.
// impl Copy for df {}

impl Index<&'static str> for df
{
    type Output = Self;
    fn index(&self, index: &'static str) -> &Self::Output
    {
        self.stage.borrow_mut().push(Item::OwnColumn(index));
        self
    }
}

/// Assumption: That the "other" `df` is actually a reference to this `df`.
/// In all contexts where `df` is used, (i.e, `filter` and `mutate`), this should
/// be true.
impl<'a, 'b> Add<&'a df> for &'b df
{
    type Output = df;
    fn add(self, rhs: &'a df) -> Self::Output
    {
        let mut new: df = self.clone();
        {
            let mut stack = new.stack.borrow_mut();
            let mut stage = new.stage.borrow_mut();
            stack.push(stage.pop().unwrap());
            stack.push(stage.pop().unwrap());
            stack.push(Item::Add);
            stage.clear();
        }

        new
    }
}

fn test<F: Fn(df) -> df>(f: F)
{
    println!("{:?}", f(df::new()));
}

fn main()
{
    // normal test
    test(|d| &d["x"] + &d["y"]);

    // test with extraneous index.
    // "a" should not appear in the output.
    test(|d| 
    {
        &d["a"];
        &d["x"] + &d["y"]
    });

    test(|d|
    {
        &(&d["a"] + &d["b"]) + &d["c"]
    });
}

// // comparison with another column
// my_data.filter(|df| df["foo"] == df["bar"])

// my_data.filter(|df| df["foo"] < df["bar"])

// my_data.filter(|df| df["foo"] >= df["bar"])

// // complicated comparison with another column

// my_data.filter(|df| df["foo"] < (df["bar"] + df["baz"]))

// // comparison with constant
// my_data.filter(|df| df["baz"] == 3)

// // complicated comparison with constant
// my_data.filter(|df| df["baz"] < (3 * df["qux"]))

