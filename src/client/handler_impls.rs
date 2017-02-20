use client::base::Client;
use client::ProcessScope;
use jack_enums::JackControl;
use super::callbacks::{NotificationHandler, ProcessHandler};

/// Wrap a closure that can handle the `process` callback. This is called every time data from ports
/// is available from JACK.
pub struct ClosureProcessHandler<F: 'static + Send + FnMut(&Client, &ProcessScope) -> JackControl> {
    pub process_fn: F,
}

impl<F> ClosureProcessHandler<F>
    where F: 'static + Send + FnMut(&Client, &ProcessScope) -> JackControl
{
    pub fn new(f: F) -> ClosureProcessHandler<F> {
        ClosureProcessHandler { process_fn: f }
    }
}

impl<F> ProcessHandler for ClosureProcessHandler<F>
    where F: 'static + Send + FnMut(&Client, &ProcessScope) -> JackControl
{
    #[allow(mutable_transmutes)]
    fn process(&mut self, c: &Client, ps: &ProcessScope) -> JackControl {
        // Casting to mut is safe because no other callbacks will accessing the `process` field.
        (self.process_fn)(c, ps)
    }
}

impl<F> NotificationHandler for ClosureProcessHandler<F>
    where F: 'static + Send + FnMut(&Client, &ProcessScope) -> JackControl
{
}
