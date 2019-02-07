pub struct Progress<T> {
    func: fn(T) -> ()
}

impl<T> Progress<T> {
    pub fn new(func: fn(T) -> ()) -> Progress<T> {
        Progress {
            func
        }
    }
    pub fn report(&self, value: T) -> () {
        (self.func)(value);
    }
}
