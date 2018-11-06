pub trait Record: Clone {}
impl<T: Clone> Record for T {}

