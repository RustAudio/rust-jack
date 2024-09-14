use crate::{Client, Control, NotificationHandler, ProcessHandler, ProcessScope};

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
#[deprecated = "Prefer using jack::contrib::ClosureProcessHandler directly."]
pub type ClosureProcessHandler<F> = crate::contrib::ClosureProcessHandler<(), F>;
