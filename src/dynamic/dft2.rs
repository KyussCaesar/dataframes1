//! Maybe `filter` starts a "filter context"?

fn main()
{
    let mut my_df = df::new();

    // filter starts a filter context
    // you can perform various operations until you get to an operation that
    // evaluates to a boolean (like less/greater/equality), which returns Ok()
    // if the operation is valid, Err() otherwise.
    my_df
        .filter()
            .add("some_col", 3)
            .less_than("some_col")?
}

