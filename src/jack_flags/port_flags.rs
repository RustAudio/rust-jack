use jack_sys as j;

bitflags! {
    /// Flags for specifying port options.
    ///
    /// * `IS_INPUT` - The port can receive data.
    ///
    /// * `IS_OUTPUT` - Data can be read from the port.
    ///
    /// * `IS_PHYSICAL` - Port corresponds to some kind of physical I/O
    /// connector.
    ///
    /// * `CAN_MONITOR` - A call to `jack_port_request_monitor()` makes
    /// sense. TODO: implement. Precisely what this means it dependent on the
    /// client. A typical result of it being called with `true` as the second
    /// argument is that data that would be available from an output port (with
    /// `IS_PHYSICAL` set) is sent to a physical output connector as well, so
    /// that it can be heard/seen/whatever.
    ///
    /// * `IS_TERMINAL` - For an input port, the data received by the port will
    /// not be passed on or made available at any other port. For output, the
    /// data available at the port does not originate from any other port. Audio
    /// synthesizers, I/O hardware interface clients, HDR systems are examples
    /// of clients that would set this flag for their ports.
    pub flags PortFlags: u32 {
        const NO_PORT_FLAGS = 0,
        const IS_INPUT    = j::JackPortIsInput,
        const IS_OUTPUT   = j::JackPortIsOutput,
        const IS_PHYSICAL = j::JackPortIsPhysical,
        const CAN_MONITOR = j::JackPortCanMonitor,
        const IS_TERMINAL = j::JackPortIsTerminal,
    }
}
