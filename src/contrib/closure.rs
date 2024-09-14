use crate::{Client, Control, Frames, ProcessHandler, ProcessScope};

/// Wrap a closure that can handle the `process` callback. This is called every time data from ports
/// is available from JACK.
pub struct ClosureProcessHandler<T, F> {
    pub state: T,
    pub callbacks: F,
}

impl<ProcessCallback> ClosureProcessHandler<(), ProcessCallback>
where
    ProcessCallback: 'static + Send + FnMut(&Client, &ProcessScope) -> Control,
{
    /// Create a new `jack::ProcessHandler` with the given process callback.
    ///
    /// ```rust
    /// // Run one cycle of processing
    /// let mut has_run = false;
    /// let handler = jack::contrib::ClosureProcessHandler::new(move |_client, _process_scope| {
    ///     if has_run {
    ///         jack::Control::Quit
    ///     } else {
    ///         has_run = true;
    ///         jack::Control::Continue
    ///     }
    /// });
    /// ```
    pub fn new(process_callback: ProcessCallback) -> Self {
        ClosureProcessHandler {
            state: (),
            callbacks: process_callback,
        }
    }
}

impl<ProcessCallback> ProcessHandler for ClosureProcessHandler<(), ProcessCallback>
where
    ProcessCallback: 'static + Send + FnMut(&Client, &ProcessScope) -> Control,
{
    fn process(&mut self, c: &Client, ps: &ProcessScope) -> Control {
        (self.callbacks)(c, ps)
    }
}

pub struct ProcessCallbacks<ProcessCallback, BufferCallback> {
    process: ProcessCallback,
    buffer: BufferCallback,
}

impl<T, ProcessCallback, BufferCallback>
    ClosureProcessHandler<T, ProcessCallbacks<ProcessCallback, BufferCallback>>
where
    T: Send,
    ProcessCallback: 'static + Send + FnMut(&mut T, &Client, &ProcessScope) -> Control,
    BufferCallback: 'static + Send + FnMut(&mut T, &Client, Frames) -> Control,
{
    /// Create a new `jack::ProcessHandler` with some state.
    ///
    /// ```rust
    /// // 1. Create the client.
    /// let (client, _status) = jack::Client::new("silence", jack::ClientOptions::default()).unwrap();
    ///
    /// // 2. Define the state.
    /// struct State{
    ///     silence: Vec<f32>,
    ///     output: jack::Port<jack::AudioOut>,
    /// }
    /// let state = State{
    ///     silence: Vec::new(),
    ///     output: client.register_port("out", jack::AudioOut::default()).unwrap(),
    /// };
    ///
    /// // 3. Define the state and closure.
    /// let process_callback = |state: &mut State, _: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
    ///     state.output.as_mut_slice(ps).copy_from_slice(state.silence.as_slice());
    ///     jack::Control::Continue
    /// };
    /// let buffer_callback = |state: &mut State, _: &jack::Client, len: jack::Frames| -> jack::Control {
    ///     state.silence = vec![0f32; len as usize];
    ///     jack::Control::Continue
    /// };
    ///
    /// // 4. Start processing.
    /// let process = jack::contrib::ClosureProcessHandler::with_state(state, process_callback, buffer_callback);
    /// let active_client = client.activate_async((), process).unwrap();
    /// ```
    pub fn with_state(
        state: T,
        process_callback: ProcessCallback,
        buffer_callback: BufferCallback,
    ) -> Self {
        ClosureProcessHandler {
            state,
            callbacks: ProcessCallbacks {
                process: process_callback,
                buffer: buffer_callback,
            },
        }
    }
}

impl<T, ProcessCallback, BufferCallback> ProcessHandler
    for ClosureProcessHandler<T, ProcessCallbacks<ProcessCallback, BufferCallback>>
where
    T: Send,
    ProcessCallback: 'static + Send + FnMut(&mut T, &Client, &ProcessScope) -> Control,
    BufferCallback: 'static + Send + FnMut(&mut T, &Client, Frames) -> Control,
{
    fn process(&mut self, c: &Client, ps: &ProcessScope) -> Control {
        (self.callbacks.process)(&mut self.state, c, ps)
    }

    fn buffer_size(&mut self, c: &Client, size: Frames) -> Control {
        (self.callbacks.buffer)(&mut self.state, c, size)
    }
}
