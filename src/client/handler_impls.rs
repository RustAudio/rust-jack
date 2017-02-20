use client::base::Client;
use client::ProcessScope;
use jack_enums::{JackControl, LatencyType};
use primitive_types as pt;
use client::client_status::ClientStatus;
use super::callbacks::{NotificationHandler, ProcessHandler};

impl NotificationHandler for () {}
impl ProcessHandler for () {
    fn process(&mut self, _: &Client, _: &ProcessScope) -> JackControl {
        JackControl::Continue
    }
}

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
    fn process(&mut self, c: &Client, ps: &ProcessScope) -> JackControl {
        (self.process_fn)(c, ps)
    }
}

/// Wrap closures that can handle notification callbacks.
///
/// For a skeleton handler, try:
/// ```
/// let notification_handler = ClosureNotificationHandler {
///     thread_init_fn: |_| {},
///     shutdown_fn: |_, _| {},
///     freewheel_fn: |_, _| {},
///     buffer_size_fn: |_, _| {JackControl::Continue},
///     sample_rate_fn: |_, _| {JackControl::Continue},
///     client_registration_fn: |_, _, _| {},
///     port_registration_fn: |_, _, _| {},
///     port_rename_fn: |_, _, _, _| {JackControl::Continue},
///     ports_connected_fn: |_, _, _, _| {},
///     graph_reorder_fn: |_| {JackControl::Continue},
///     xrun_fn: |_| {JackControl::Continue},
///     latency_fn: |_, _| {},
/// }
/// ```
pub struct ClosureNotificationHandler<TI, SD, FW, BS, SR, CR, PREG, PREN, PC, GR, XR, LA>
    where TI: 'static + Send + Fn(&Client),
          SD: 'static + Send + FnMut(ClientStatus, &str),
          FW: 'static + Send + FnMut(&Client, bool),
          BS: 'static + Send + FnMut(&Client, pt::JackFrames) -> JackControl,
          SR: 'static + Send + FnMut(&Client, pt::JackFrames) -> JackControl,
          CR: 'static + Send + FnMut(&Client, &str, bool),
          PREG: 'static + Send + FnMut(&Client, pt::JackPortId, bool),
          PREN: 'static + Send + FnMut(&Client, pt::JackPortId, &str, &str) -> JackControl,
          PC: 'static + Send + FnMut(&Client, pt::JackPortId, pt::JackPortId, bool),
          GR: 'static + Send + FnMut(&Client) -> JackControl,
          XR: 'static + Send + FnMut(&Client) -> JackControl,
          LA: 'static + Send + FnMut(&Client, LatencyType)
{
    pub thread_init_fn: TI,
    pub shutdown_fn: SD,
    pub freewheel_fn: FW,
    pub buffer_size_fn: BS,
    pub sample_rate_fn: SR,
    pub client_registration_fn: CR,
    pub port_registration_fn: PREG,
    pub port_rename_fn: PREN,
    pub ports_connected_fn: PC,
    pub graph_reorder_fn: GR,
    pub xrun_fn: XR,
    pub latency_fn: LA,
}

impl<TI, SD, FW, BS, SR, CR, PREG, PREN, PC, GR, XR, LA> NotificationHandler
    for ClosureNotificationHandler<TI, SD, FW, BS, SR, CR, PREG, PREN, PC, GR, XR, LA>
    where TI: 'static + Send + Fn(&Client),
          SD: 'static + Send + FnMut(ClientStatus, &str),
          FW: 'static + Send + FnMut(&Client, bool),
          BS: 'static + Send + FnMut(&Client, pt::JackFrames) -> JackControl,
          SR: 'static + Send + FnMut(&Client, pt::JackFrames) -> JackControl,
          CR: 'static + Send + FnMut(&Client, &str, bool),
          PREG: 'static + Send + FnMut(&Client, pt::JackPortId, bool),
          PREN: 'static + Send + FnMut(&Client, pt::JackPortId, &str, &str) -> JackControl,
          PC: 'static + Send + FnMut(&Client, pt::JackPortId, pt::JackPortId, bool),
          GR: 'static + Send + FnMut(&Client) -> JackControl,
          XR: 'static + Send + FnMut(&Client) -> JackControl,
          LA: 'static + Send + FnMut(&Client, LatencyType)
{
    fn thread_init(&self, c: &Client) {
        (self.thread_init_fn)(c)
    }

    fn shutdown(&mut self, status: ClientStatus, reason: &str) {
        (self.shutdown_fn)(status, reason)
    }

    fn freewheel(&mut self, c: &Client, enabled: bool) {
        (self.freewheel_fn)(c, enabled)
    }

    fn buffer_size(&mut self, c: &Client, sz: pt::JackFrames) -> JackControl {
        (self.buffer_size_fn)(c, sz)
    }

    fn sample_rate(&mut self, c: &Client, srate: pt::JackFrames) -> JackControl {
        (self.sample_rate_fn)(c, srate)
    }

    fn client_registration(&mut self, c: &Client, name: &str, is_reg: bool) {
        (self.client_registration_fn)(c, name, is_reg)
    }

    fn port_registration(&mut self, c: &Client, port_id: pt::JackPortId, is_reg: bool) {
        (self.port_registration_fn)(c, port_id, is_reg)
    }

    fn port_rename(&mut self, c: &Client, port_id: pt::JackPortId, old_name: &str, new_name: &str) -> JackControl {
        (self.port_rename_fn)(c, port_id, old_name, new_name)
    }

    fn ports_connected(&mut self, c: &Client, pa: pt::JackPortId, pb: pt::JackPortId, are_connected: bool) {
        (self.ports_connected_fn)(c, pa, pb, are_connected)
    }

    fn graph_reorder(&mut self, c: &Client) -> JackControl {
        (self.graph_reorder_fn)(c)
    }

    fn xrun(&mut self, c: &Client) -> JackControl {
        (self.xrun_fn)(c)
    }

    fn latency(&mut self, c: &Client, mode: LatencyType) {
        (self.latency_fn)(c, mode)
    }
}

