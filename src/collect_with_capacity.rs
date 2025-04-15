pub trait CollectWithCapacity: Iterator {
    fn collect_with_capacity<T>(self, capacity: usize) -> Vec<Self::Item>
    where
        Self: Sized + Iterator<Item = T>,
    {
        let mut vec = Vec::with_capacity(capacity);
        vec.extend(self);
        vec
    }
}

impl<T: ?Sized> CollectWithCapacity for T where T: Iterator {}
