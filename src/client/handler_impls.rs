use super::callbacks::{NotificationHandler, ProcessHandler};
use client::ProcessScope;
use client::client::Client;
use jack_enums::Control;

/// A trivial handler that does nothing.
impl NotificationHandler for () {}

/// A trivial handler that does nothing.
impl ProcessHandler for () {
    /// Return `Control::Continue` so that the client stays activated.
    fn process(&mut self, _: &Client, _: &ProcessScope) -> Control {
        Control::Continue
    }
}

/// Wrap a closure that can handle the `process` callback. This is called every time data from ports
/// is available from JACK.
pub struct ClosureProcessHandler<F: 'static + Send + FnMut(&Client, &ProcessScope) -> Control> {
    pub process_fn: F,
}

impl<F> ClosureProcessHandler<F>
where
    F: 'static + Send + FnMut(&Client, &ProcessScope) -> Control,
{
    pub fn new(f: F) -> ClosureProcessHandler<F> {
        ClosureProcessHandler { process_fn: f }
    }
}

impl<F> ProcessHandler for ClosureProcessHandler<F>
where
    F: 'static + Send + FnMut(&Client, &ProcessScope) -> Control,
{
    fn process(&mut self, c: &Client, ps: &ProcessScope) -> Control {
        (self.process_fn)(c, ps)
    }
}
