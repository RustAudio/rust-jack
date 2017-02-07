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
    /// Option flags for opening a JACK client.
    pub flags ClientOptions: j::Enum_JackOptions {
        /// Do not automatically start the JACK server when it is not already running. This option
        /// is always selected if `$JACK_NO_START_SERVER` is defined in the calling process
        /// environment.
        const NO_START_SERVER = j::JackNoStartServer,

        /// Use the exact client name requested. Otherwise, JACK
        /// automatically generates a unique one if needed.
        const USE_EXACT_NAME  = j::JackUseExactName,

        /// Open with optional `server_name` parameter.
        ///
        /// TODO: implement
        const SERVER_NAME     = j::JackServerName,

        /// Load internal client from optional `load_name`, otherwise use the `client_name`.
        ///
        /// TODO: wmedrano
        const LOAD_NAME       = j::JackLoadName,

        /// Pass optional `load_init` to `jack_initialize()` entry
        /// point of an internal client.
        ///
        /// TODO: implement
        const LOAD_INIT       = j::JackLoadInit,

        /// Pass a SessionID token. This allows the session manager to identify the client again.
        const SESSION_ID      = j::JackSessionID,
    }
}
