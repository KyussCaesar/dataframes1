//! This tests use of the crate with CSV files, using the classic iris dataset.
//! Later, working with `csv` will be integrated into the crate proper.

extern crate df_rs;

extern crate csv;

#[macro_use] extern crate serde_derive;

#[derive(Clone, Debug, Deserialize)]
struct Record
{
    sepal_length : f32,
    sepal_width  : f32,
    petal_length : f32,
    petal_width  : f32,
    class        : String,
}

type DataFrame = df_rs::DataFrame<Record>;

#[test]
fn csv_test() -> csv::Result<()>
{
    let mut rdr = csv::Reader::from_path("tests/iris.csv")?;
    let mut df = DataFrame::new();
    for result in rdr.deserialize()
    {
        let record: Record = result?;
        df.push(record);
    }

    println!("{:?}", df);

    Ok(())
}


