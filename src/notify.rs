use std::time::Duration;

pub trait Notify<E> {
    fn notify(&mut self, err: &E, duration: Duration);
}

impl<E, F> Notify<E> for F
where
    F: FnMut(&E, Duration),
{
    fn notify(&mut self, err: &E, duration: Duration) {
        self(err, duration)
    }
}
