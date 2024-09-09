use bitflags::bitflags;
use jack_sys as j;

bitflags! {
    /// Flags for specifying port options.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub struct PortFlags: j::Enum_JackPortFlags {
        /// The port can receive data.
        const IS_INPUT    = j::JackPortIsInput;

        /// Data can be read from the port.
        const IS_OUTPUT   = j::JackPortIsOutput;

        /// Port corresponds to some kind of physical I/O connector.
        const IS_PHYSICAL = j::JackPortIsPhysical;

        /// A call to `jack_port_request_monitor()` makes sense.
        ///
        /// # TODO implement
        ///
        /// Precisely what this means it dependent on the client. A typical result of it being
        /// called with `true` as the second argument is that data that would be available from an
        /// output port (with `IS_PHYSICAL` set) is sent to a physical output connector as well, so
        /// that it can be heard/seen/whatever.
        const CAN_MONITOR = j::JackPortCanMonitor;

        /// For an input port, the data received by the port will not be passed on or made available
        /// at any other port. For output, the data available at the port does not originate from
        /// any other port. Audio synthesizers, I/O hardware interface clients, HDR systems are
        /// examples of clients that would set this flag for their ports.
        const IS_TERMINAL = j::JackPortIsTerminal;
    }
}
