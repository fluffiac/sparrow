/// Builder struct that creates a "pipe" through a function.
///
/// ```
/// fn counter(data: &f64) -> String {
///    format!(r#"{{"event": "counter", "data": {:?}}}"#, data)
/// }
///
/// # fn example() {
/// let (tx, rx) = kanal::bounded(1);
///
/// let counter_pipe = pipe(counter).to(&tx);
/// counter_pipe(&3.14);
///
/// assert_eq!(rx.recv().unwrap(), r#"{"event": "counter", "data": 3.14}"#);    
/// # }
/// ```
pub struct Pipe<T> {
    f: fn(&T) -> String,
}

impl<T> Pipe<T> {
    pub fn to(self, chan: &kanal::Sender<String>) -> impl Fn(&T) {
        let chan = chan.clone();

        move |data| {
            let _ = chan.send((self.f)(data));
        }
    }
}

pub fn pipe<T>(f: fn(&T) -> String) -> Pipe<T> {
    Pipe { f }
}
