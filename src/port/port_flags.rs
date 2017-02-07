// The MIT License (MIT)
//
// Copyright (c) 2017 Will Medrano (will.s.medrano@gmail.com)
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

use jack_sys as j;

bitflags! {
    /// Flags for specifying port options.
    pub flags PortFlags: j::Enum_JackPortFlags {
        /// The port can receive data.
        const IS_INPUT    = j::JackPortIsInput,

        /// Data can be read from the port.
        const IS_OUTPUT   = j::JackPortIsOutput,

        /// Port corresponds to some kind of physical I/O connector.
        const IS_PHYSICAL = j::JackPortIsPhysical,

        /// A call to `jack_port_request_monitor()` makes sense.
        ///
        /// # TODO implement
        ///
        /// Precisely what this means it dependent on the client. A typical result of it being
        /// called with `true` as the second argument is that data that would be available from an
        /// output port (with `IS_PHYSICAL` set) is sent to a physical output connector as well, so
        /// that it can be heard/seen/whatever.
        const CAN_MONITOR = j::JackPortCanMonitor,

        /// For an input port, the data received by the port will not be passed on or made available
        /// at any other port. For output, the data available at the port does not originate from
        /// any other port. Audio synthesizers, I/O hardware interface clients, HDR systems are
        /// examples of clients that would set this flag for their ports.
        const IS_TERMINAL = j::JackPortIsTerminal,
    }
}
