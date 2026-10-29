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
