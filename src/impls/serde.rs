pub trait Biserialize: serde::Serialize + serde::Deserialize {}
impl<T: serde::Serialize + serde::Deserialize> Biserialize for T {}

pub trait Record: base::Record + Biserialize {}
impl<T: base::Record + serde::Serialize + serde::Deserialize {}

