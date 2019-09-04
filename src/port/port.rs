use jack_sys as j;
use libc;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::hash::{Hash, Hasher};
use std::marker::Sized;
use std::sync::Weak;
use std::{ffi, fmt, iter};

use Error;
use Frames;
use PortFlags;

lazy_static! {
    /// The maximum string length for port names.
    pub static ref PORT_NAME_SIZE: usize = unsafe { j::jack_port_name_size() - 1 } as usize;

    /// The maximum string length for jack type names.
    pub static ref PORT_TYPE_SIZE: usize = unsafe { j::jack_port_type_size() - 1 } as usize;
}

/// Defines the configuration for a certain port to JACK, ie 32 bit floating audio input, 8 bit raw
/// midi output, etc...
pub unsafe trait PortSpec: Sized {
    /// String used by JACK upon port creation to identify the port
    /// type.
    fn jack_port_type(&self) -> &str;

    /// Flags used by jack upon port creation.
    fn jack_flags(&self) -> PortFlags;

    /// Size used by jack upon port creation.
    fn jack_buffer_size(&self) -> libc::c_ulong;
}

/// An endpoint to interact with JACK data streams, for audio, midi, etc...
///
/// The `Port` struct contains mostly metadata and exposes data as raw pointers. For a better data
/// consumption/production API, see the `AudioInPort`, `AudioOutPort`, `MidiInPort`, and
/// `MidiOutPort`.
///
/// Most JACK functionality is exposed, including the raw pointers, but it should be possible to
/// create a client without the need for calling `unsafe` `Port` methods.
///
/// Also, ports can be compared and hashed using their raw pointers.
pub struct Port<PS> {
    spec: PS,
    client_ptr: *mut j::jack_client_t,
    port_ptr: *mut j::jack_port_t,
    client_life: Weak<()>,
}

unsafe impl<PS: PortSpec + Send> Send for Port<PS> {}
unsafe impl<PS: PortSpec + Sync> Sync for Port<PS> {}

impl<PS> Port<PS> {
    /// Returns the spec that was used to create this port.
    pub fn spec(&self) -> &PS {
        &self.spec
    }

    /// Return a copy of port as an unowned port that can still be used for
    /// querying information.
    pub fn clone_unowned(&self) -> Port<Unowned> {
        Port {
            spec: Unowned,
            client_ptr: self.client_ptr(),
            port_ptr: self.raw(),
            client_life: self.client_life.clone(),
        }
    }

    /// Returns the full name of the port, including the "client_name:" prefix.
    pub fn name(&self) -> Result<String, Error> {
        self.check_client_life()?;
        let s = unsafe {
            ffi::CStr::from_ptr(j::jack_port_name(self.raw()))
                .to_string_lossy()
                .to_string()
        };
        Ok(s)
    }

    /// Returns the short name of the port, it excludes the "client_name:"
    /// prefix.
    pub fn short_name(&self) -> Result<String, Error> {
        self.check_client_life()?;
        let s = unsafe {
            ffi::CStr::from_ptr(j::jack_port_short_name(self.raw()))
                .to_string_lossy()
                .to_string()
        };
        Ok(s)
    }

    /// The flags for the port. These are set when the port is registered with
    /// its client.
    pub fn flags(&self) -> PortFlags {
        let bits = unsafe { j::jack_port_flags(self.raw()) };
        PortFlags::from_bits(bits as j::Enum_JackPortFlags).unwrap()
    }

    /// The port type. JACK's built in types include `"32 bit float mono audio`" and `"8 bit raw
    /// midi"`. Custom types may also be used.
    pub fn port_type(&self) -> Result<String, Error> {
        self.check_client_life()?;
        let s = unsafe {
            ffi::CStr::from_ptr(j::jack_port_type(self.raw()))
                .to_string_lossy()
                .to_string()
        };
        Ok(s)
    }

    /// Number of ports connected to/from `&self`.
    pub fn connected_count(&self) -> Result<usize, Error> {
        self.check_client_life()?;
        let n = unsafe { j::jack_port_connected(self.raw()) };
        Ok(n as usize)
    }

    /// Returns `true` if the port is directly connected to a port with the
    /// name `port_name`.
    pub fn is_connected_to(&self, port_name: &str) -> Result<bool, Error> {
        self.check_client_life()?;
        let res = unsafe {
            let port_name = ffi::CString::new(port_name).unwrap();
            j::jack_port_connected_to(self.raw(), port_name.as_ptr())
        };
        match res {
            0 => Ok(false),
            _ => Ok(true),
        }
    }

    /// Get the alias names for `self`.
    ///
    /// Will return up to 2 strings.
    pub fn aliases(&self) -> Result<Vec<String>, Error> {
        self.check_client_life()?;
        let mut a: Vec<libc::c_char> = iter::repeat(0).take(*PORT_NAME_SIZE + 1).collect();
        let mut b = a.clone();
        unsafe {
            let mut ptrs: [*mut libc::c_char; 2] = [a.as_mut_ptr(), b.as_mut_ptr()];
            j::jack_port_get_aliases(self.raw(), ptrs.as_mut_ptr());
        };
        Ok([a, b]
            .iter()
            .map(|p| p.as_ptr())
            .map(|p| unsafe { ffi::CStr::from_ptr(p).to_string_lossy().into_owned() })
            .filter(|s| !s.is_empty())
            .collect())
    }

    /// Returns `true` if monitoring has been requested for `self`.
    pub fn is_monitoring_input(&self) -> Result<bool, Error> {
        self.check_client_life()?;
        match unsafe { j::jack_port_monitoring_input(self.raw()) } {
            0 => Ok(false),
            _ => Ok(true),
        }
    }

    /// Turn input monitoring for the port on or off.
    ///
    /// This only works if the port has the `CAN_MONITOR` flag set.
    pub fn request_monitor(&self, enable_monitor: bool) -> Result<(), Error> {
        self.check_client_life()?;
        let onoff = if enable_monitor { 1 } else { 0 };
        let res = unsafe { j::jack_port_request_monitor(self.raw(), onoff) };
        match res {
            0 => Ok(()),
            _ => Err(Error::PortMonitorError),
        }
    }

    /// If the `CAN_MONITOR` flag is set for the port, then input monitoring is turned on if it was
    /// off, and turns it off if only one request has been made to turn it on.  Otherwise it does
    /// nothing.
    pub fn ensure_monitor(&self, enable_monitor: bool) -> Result<(), Error> {
        self.check_client_life()?;
        let onoff = if enable_monitor { 1 } else { 0 };
        let res = unsafe { j::jack_port_ensure_monitor(self.raw(), onoff) };
        match res {
            0 => Ok(()),
            _ => Err(Error::PortMonitorError),
        }
    }

    /// Set's the short name of the port. If the full name is longer than `PORT_NAME_SIZE`, then it
    /// will be truncated.
    pub fn set_name(&mut self, short_name: &str) -> Result<(), Error> {
        self.check_client_life()?;
        let short_name = ffi::CString::new(short_name).unwrap();
        let res = unsafe { j::jack_port_set_name(self.raw(), short_name.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(Error::PortNamingError),
        }
    }

    /// Sets `alias` as an alias for `self`.
    ///
    /// May be called at any time. If the alias is longer than `PORT_NAME_SIZE`, it will be
    /// truncated.
    ///
    /// After a successful call, and until JACK exists, or the alias is unset, `alias` may be used
    /// as an alternate name for the port.
    ///
    /// Ports can have up to two aliases - if both are already set, this function will return an
    /// error.
    pub fn set_alias(&mut self, alias: &str) -> Result<(), Error> {
        self.check_client_life()?;
        let alias = ffi::CString::new(alias).unwrap();
        let res = unsafe { j::jack_port_set_alias(self.raw(), alias.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(Error::PortAliasError),
        }
    }

    /// Remove `alias` as an alias for port. May be called at any time.
    ///
    /// After a successful call, `alias` can no longer be used as an alternate name for `self`.
    pub fn unset_alias(&mut self, alias: &str) -> Result<(), Error> {
        self.check_client_life()?;
        let alias = ffi::CString::new(alias).unwrap();
        let res = unsafe { j::jack_port_unset_alias(self.raw(), alias.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(Error::PortAliasError),
        }
    }

    /// Create a Port from raw JACK pointers.
    ///
    /// This is mostly for use within the jack crate itself.
    pub unsafe fn from_raw(
        spec: PS,
        client_ptr: *mut j::jack_client_t,
        port_ptr: *mut j::jack_port_t,
        client_life: Weak<()>,
    ) -> Self {
        Port {
            spec,
            port_ptr,
            client_ptr,
            client_life,
        }
    }

    /// Obtain the client pointer that spawned this port.
    ///
    /// This is mostly for use within the jack crate itself.
    #[inline(always)]
    pub fn client_ptr(&self) -> *mut j::jack_client_t {
        self.client_ptr
    }

    /// Obtain the ffi port pointer.
    ///
    /// This is mostly for use within the jack crate itself.
    #[inline(always)]
    pub fn raw(&self) -> *mut j::jack_port_t {
        self.port_ptr
    }

    /// Obtain the buffer that the Port is holding. For standard audio and midi ports, consider
    /// using the `AudioInPort`, `AudioOutPort`, `MidiInPort`, or `MidiOutPort` adapter. For more
    /// custom data, consider implementing your own adapter that safely uses the `Port::buffer`
    /// method.
    #[inline(always)]
    pub unsafe fn buffer(&self, n_frames: Frames) -> *mut libc::c_void {
        // We don't check for life to improve performance in a very hot codepath.
        // self.check_client_life()?;
        j::jack_port_get_buffer(self.port_ptr, n_frames)
    }

    fn check_client_life(&self) -> Result<(), Error> {
        self.client_life
            .upgrade()
            .map(|_| ())
            .ok_or(Error::ClientIsNoLongerAlive)
    }
}

/// `PortSpec` for a port that holds has no readable or writeable data from JACK on the created
/// client. It can be used to connect ports or to obtain metadata.
#[derive(Debug, Default)]
pub struct Unowned;

unsafe impl PortSpec for Unowned {
    /// Panics on call since the `Unowned` spec can't be used to create ports.
    fn jack_port_type(&self) -> &str {
        ""
    }

    /// Panics on call since the `Unowned` spec can't be used to create ports.
    fn jack_flags(&self) -> PortFlags {
        PortFlags::empty()
    }

    /// Panics on call since the `Unowned` spec can't be used to create ports.
    fn jack_buffer_size(&self) -> libc::c_ulong {
        unreachable!()
    }
}

#[derive(Debug)]
struct PortInfo {
    name: String,
    connections: usize,
    port_type: String,
    port_flags: PortFlags,
    aliases: Vec<String>,
}

impl PortInfo {
    fn new<PS: PortSpec>(p: &Port<PS>) -> PortInfo {
        let s = p.spec();
        PortInfo {
            name: p
                .name()
                .unwrap_or_else(|_| String::from("client not alive")),
            connections: p.connected_count().unwrap_or(0),
            port_type: s.jack_port_type().to_owned(),
            port_flags: s.jack_flags(),
            aliases: p.aliases().unwrap_or_else(|_| Vec::new()),
        }
    }
}

impl<PS: PortSpec> fmt::Debug for Port<PS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "{:?}", PortInfo::new(self))
    }
}

impl<PS> PartialEq for Port<PS> {
    fn eq(&self, other: &Self) -> bool {
        self.port_ptr == other.port_ptr
    }
}

impl<PS> Eq for Port<PS> {}

impl<PS> PartialOrd for Port<PS> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<PS> Ord for Port<PS> {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.port_ptr < other.port_ptr {
            Ordering::Less
        } else if self.port_ptr > other.port_ptr {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl<PS> Hash for Port<PS> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.port_ptr.hash(state);
    }
}

impl Clone for Port<Unowned> {
    fn clone(&self) -> Self {
        self.clone_unowned()
    }
}
