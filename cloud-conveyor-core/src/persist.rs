pub trait Persist<T> {
    fn persist(&mut self, instance: &T);
}
