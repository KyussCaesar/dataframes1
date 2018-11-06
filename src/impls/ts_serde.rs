pub trait Record: base::Record + ts::ThreadSafe + serial::Biserialize {}
impl<T: base::Record + ts::ThreadSafe + serial::Biserialize> Record for T {}

