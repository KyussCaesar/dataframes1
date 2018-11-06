pub trait ThreadSafe: Send + Sync {}
impl<T: Send + Sync> ThreadSafe for T {}

pub trait Record: base::Record + ThreadSafe {}
impl<T: base::Record + ThreadSafe> Record for T {}

