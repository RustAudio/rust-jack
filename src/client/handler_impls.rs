use crate::{Client, Control, Frames, NotificationHandler, ProcessHandler, ProcessScope};

/// A trivial handler that does nothing.
impl NotificationHandler for () {}

/// A trivial handler that does nothing.
impl ProcessHandler for () {
    fn buffer_size(&mut self, _: &Client, _: Frames) -> Control {
        Control::Continue
    }

    /// Return `Control::Continue` so that the client stays activated.
    fn process(&mut self, _: &Client, _: &ProcessScope) -> Control {
        Control::Continue
    }
}
