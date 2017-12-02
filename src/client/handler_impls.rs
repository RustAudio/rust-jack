use super::callbacks::{NotificationHandler, ProcessHandler};
use client::ProcessScope;
use client::base::Client;
use jack_enums::JackControl;

impl NotificationHandler for () {}
impl ProcessHandler for () {
    fn process(&mut self, _: &Client, _: &ProcessScope) -> JackControl {
        JackControl::Continue
    }
}

/// Wrap a closure that can handle the `process` callback. This is called every
/// time data from ports
/// is available from JACK.
pub struct ClosureProcessHandler<F: 'static + Send + FnMut(&Client, &ProcessScope) -> JackControl> {
    pub process_fn: F,
}

impl<F> ClosureProcessHandler<F>
where
    F: 'static + Send + FnMut(&Client, &ProcessScope) -> JackControl,
{
    pub fn new(f: F) -> ClosureProcessHandler<F> {
        ClosureProcessHandler { process_fn: f }
    }
}

impl<F> ProcessHandler for ClosureProcessHandler<F>
    where F: 'static + Send + FnMut(&Client, &ProcessScope) -> JackControl
{
    fn process(&mut self, c: &Client, ps: &ProcessScope) -> JackControl {
        (self.process_fn)(c, ps)
    }
}
