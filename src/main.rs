use std::iter;
use std::sync::{Arc, Mutex};

use futures::executor::{self, Notify, Spawn};
use futures::sync::mpsc;
use futures::{stream, Async, Future, Sink, Stream};

struct Notifier<F>(Arc<Mutex<Option<Spawn<F>>>>);
impl<F> Clone for Notifier<F> {
    fn clone(&self) -> Notifier<F> {
        Notifier(Arc::clone(&self.0))
    }
}

impl<F> Notify for Notifier<F>
where
    F: Future<Item = (), Error = ()> + Send + 'static,
{
    fn notify(&self, id: usize) {
        let n = Arc::new(self.clone());
        let mut s = self.0.lock().unwrap();
        match s.as_mut().map(|spawn| spawn.poll_future_notify(&n, id)) {
            Some(Ok(Async::NotReady)) | None => {}
            _ => *s = None,
        }
    }
}

#[allow(dead_code)]
fn test_futures_1() {
    let (tx, rx) = mpsc::channel::<u64>(1);

    let s = stream::iter_ok::<_, ()>(iter::repeat(0u64).take(3));
    let f = tx
        .sink_map_err(|_| println!("sink error"))
        .send_all(s)
        .map(|_| println!("send suceess"));
    let spawn = Arc::new(Mutex::new(Some(executor::spawn(f))));
    Notifier(spawn).notify(0);

    println!("received: {}", rx.wait().count());
}

fn main() {
    test_futures_1();
}
