use std::{ffi, slice};
use jack_sys as j;
use client::*;
use callbacks::ProcessScope;
use flags::*;
use enums::*;
use utils;
use std::marker::PhantomData;

/// The maximum length of a full Jack port name. Unlike the "C" Jack API,
/// this does not count the `NULL` character and corresponds to a string's
/// `.len()`.
///
/// The port's full name contains the owning client name concatenated with a
/// colon (:) followed by its short name.
///
/// This value is constant
pub fn port_name_size() -> usize {
    let s = unsafe { j::jack_port_name_size() - 1 };
    s as usize
}

/// The maximum length of a port type. Unlike the "C" Jack API, this does
/// not count the `NULL` character and corresponds to a string's `.len()`.
///
/// This value is constant.
pub fn port_type_size() -> usize {
    let s = unsafe { j::jack_port_type_size() - 1 };
    s as usize
}

lazy_static! {
    pub static ref PORT_NAME_SIZE: usize = port_name_size();
    pub static ref PORT_TYPE_SIZE: usize = port_type_size();
}

pub unsafe fn port_pointer<K: PortOwnershipKind>(port: &Port<K>) -> *mut j::jack_port_t {
    port.port
}

/// Converts a jack client handle and jack port handle in a `Port`. If either
/// `client` or `port` is `null`, then `None` is returned.
pub unsafe fn ptrs_to_port(client: *mut j::jack_client_t,
                           port: *mut j::jack_port_t)
                           -> Option<Port<Unowned>> {
    if client.is_null() || port.is_null() {
        None
    } else {
        Some(Port {
            port: port,
            kind: Unowned {},
        })
    }
}

pub unsafe fn port_to_owned<InKind: PortOwnershipKind, OutType: PortType>
    (port: Port<InKind>,
     client_id: ClientId)
     -> Port<Owned<OutType>> {
    Port {
        port: port.port,
        kind: Owned {
            client_id: client_id,
            _type: PhantomData,
        },
    }
}

/// Returns a pointer to the memory area associated with the specified
/// port. For an output port, it will be a memory area that can be written
/// to; for an input port, it will be an area containing the data from the
/// port's connection(s), or zero-filled. If there are multiple inbound
/// connections, the data will be mixed appropriately.
///
/// Do not cache the returned address across `process()` calls. Port buffers
/// have to be retrieved in each callback for proper functioning.
/// The provided port must be owned and valid for the lifetime of the result.
pub unsafe fn buffer(port: *mut j::jack_port_t, n_frames: u32) -> *mut ::libc::c_void {
    j::jack_port_get_buffer(port, n_frames)
}

pub trait PortOwnershipKind: Sized {}
pub unsafe trait PortType: Sized + Copy {
    type DataType: PortDataType;
    fn necessary_flags() -> PortFlags;
}
pub unsafe trait PortDataType: Sized + Copy {
    type BufferType: Sized;
    fn type_identifier() -> &'static str;
}

#[derive(Debug, Copy, Clone)]
pub struct Owned<PType: PortType> {
    // The id of the client that created the Port. This is used to ensure
    // that the port's buffers are only read or mutated in the process fn
    // of the client that owns the Port
    client_id: ClientId,

    // Marker type for compile-time information about how the port can be
    // used
    _type: PhantomData<PType>,
}
#[derive(Debug, Copy, Clone)]
pub struct Unowned;

impl<PType: PortType> PortOwnershipKind for Owned<PType> {}
impl PortOwnershipKind for Unowned {}

#[derive(Debug, Copy, Clone)]
pub struct Input<PDT: Sized + Copy> {
    data_type: PDT,
}

#[derive(Debug, Copy, Clone)]
pub struct Output<PDT: Sized + Copy> {
    data_type: PDT,
}

#[derive(Debug, Copy, Clone)]
pub struct UnknownOwned;

unsafe impl<PDT: PortDataType> PortType for Input<PDT> {
    type DataType = PDT;
    fn necessary_flags() -> PortFlags {
        IS_INPUT
    }
}
unsafe impl<PDT: PortDataType> PortType for Output<PDT> {
    type DataType = PDT;
    fn necessary_flags() -> PortFlags {
        IS_OUTPUT
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Audio {}

unsafe impl PortDataType for Audio {
    type BufferType = f32;
    fn type_identifier() -> &'static str {
        DEFAULT_AUDIO_TYPE
    }
}

// TODO(cramertj) finish implementing MIDI input tyupe
//
// #[derive(Debug, Copy, Clone)]
// pub struct Midi {}
//
// unsafe impl PortAudioType for Midi {
//    type BufferType =
// }

/// An endpoint to interact with Jack data streams, for audio, midi, etc...
#[derive(Debug)]
pub struct Port<Kind: PortOwnershipKind> {
    // The Jack port itself
    port: *mut j::jack_port_t,

    // Owned<some PortType> or Unowned
    // This stores the type that describes how the port can be used
    kind: Kind,
}

// The `port` field is only used to query Jack, and the `client`
// field is only used to ensure that the port isn't accessed from
// non-owning clients. Neither is explicitly dereferenced, so it's
// safe to send them across thread boundaries (so long as Jack itself
// handles concurrent requests properly).
unsafe impl<Kind: PortOwnershipKind> Send for Port<Kind> {}

// It's only safe to change the given port as long as there are no other
// references to the port. The client must be the client used to create
// the port and must also be alive for the duration of the borrow.
// pub fn port_mut_pointer<'a, OKind: OwnedPortKind, T: JackHandler>(
// port: &'a mut Port<OKind>, owner_client: &'a mut Client) -> &'a mut j::jack_port_t {
// unsafe {
// assert_eq!(
// owner_client.client_ as *mut j::jack_client_t,
// port.client as *const j::jack_client_t);
// transmute(port.port)
// }
// }
//

impl<PDT: PortDataType> Port<Owned<Input<PDT>>> {
    pub fn input_buffer(&self, process_scope: &ProcessScope) -> &[PDT::BufferType] {
        unsafe {
            assert!(process_scope.client_equals(self.kind.client_id),
                    "Port buffers may only be from handler of the client that created the port.");
            let n_frames = process_scope.n_frames();
            let buffer = buffer(self.port, n_frames) as *const PDT::BufferType;
            slice::from_raw_parts(buffer, n_frames as usize)
        }
    }
}

impl<PDT: PortDataType> Port<Owned<Output<PDT>>> {
    pub fn output_buffer(&mut self, process_scope: &mut ProcessScope) -> &mut [PDT::BufferType] {
        unsafe {
            assert!(process_scope.client_equals(self.kind.client_id),
                    "Port buffers may only be from handler of the client that created the port.");
            let n_frames = process_scope.n_frames();
            let buffer = buffer(self.port, n_frames) as *mut PDT::BufferType;
            slice::from_raw_parts_mut(buffer, n_frames as usize)
        }
    }
}

// These functions mutate the Port, and should are be usable if we are
// the owner.
impl<Type: PortType> Port<Owned<Type>> {
    /// Remove the port from the client, disconnecting any existing connections.
    /// The port must have been created with the provided client.
    pub fn unregister(self, client: &mut Client) -> Result<(), JackErr> {
        let res = unsafe {
            assert!(client.id() == self.kind.client_id);
            j::jack_port_unregister(client.client_ptr(), self.port)
        };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::CallbackDeregistrationError),
        }
    }

    /// Set's the short name of the port. If the full name is longer than
    /// `Port::name_size()`, then it will be truncated.
    pub fn set_name(&mut self, short_name: &str) -> Result<(), JackErr> {
        let short_name = ffi::CString::new(short_name).unwrap();
        let res = unsafe { j::jack_port_set_name(self.port, short_name.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortNamingError),
        }
    }

    /// Sets `alias` as an alias for `self`.
    ///
    /// May be called at any time. If the alias is longer than
    /// `Client::name_size()`, it will be truncated.
    ///
    /// After a successful call, and until Jack exists, or the alias is unset,
    /// `alias` may be used as an alternate name for the port.
    ///
    /// Ports can have up to two aliases - if both are already set, this
    /// function will return an error.
    pub fn set_alias(&self, alias: &str) -> Result<(), JackErr> {
        let alias = ffi::CString::new(alias).unwrap();
        let res = unsafe { j::jack_port_set_alias(self.port, alias.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortAliasError),
        }
    }

    /// Remove `alias` as an alias for port. May be called at any time.
    ///
    /// After a successful call, `alias` can no longer be used as an alternate
    /// name for `self`.
    pub fn unset_alias(&self, alias: &str) -> Result<(), JackErr> {
        let alias = ffi::CString::new(alias).unwrap();
        let res = unsafe { j::jack_port_unset_alias(self.port, alias.as_ptr()) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortAliasError),
        }
    }

    /// Turn input monitoring for the port on or off.
    ///
    /// This only works if the port has the `CAN_MONITOR` flag set.
    pub fn request_monitor(&self, enable_monitor: bool) -> Result<(), JackErr> {
        let onoff = match enable_monitor {
            true => 1,
            false => 0,
        };
        let res = unsafe { j::jack_port_request_monitor(self.port, onoff) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortMonitorError),
        }
    }

    /// Perform the same function as `Client::disconnect_ports()`, but with a
    /// port handle instead.
    ///
    /// Avoids the name lookup inherent in the name-based version.
    ///
    /// Clients connecting their own ports are likely to use this function,
    /// while generic connection clients (e.g. patchbays) would use
    /// `Client::disconnect_ports()`.
    pub fn disconnect(self, client: &mut Client) -> Result<(), JackErr> {
        match unsafe {
            assert!(client.id() == self.kind.client_id);
            j::jack_port_disconnect(client.client_ptr(), self.port)
        } {
            0 => Ok(()),
            _ => Err(JackErr::PortDisconnectionError),
        }
    }
}

impl<Kind: PortOwnershipKind> Port<Kind> {
    /// Returns the full name of the port, including the "client_name:" prefix.
    pub fn name<'a>(&'a self) -> &'a str {
        unsafe { ffi::CStr::from_ptr(j::jack_port_name(self.port)).to_str().unwrap() }
    }

    /// Returns the short name of the port, it excludes the "client_name:" prefix.
    pub fn short_name<'a>(&'a self) -> &'a str {
        unsafe { ffi::CStr::from_ptr(j::jack_port_short_name(self.port)).to_str().unwrap() }
    }

    /// Returns the uuid of the port as a u64.
    pub fn uuid(&self) -> u64 {
        unsafe { j::jack_port_uuid(self.port) }
    }

    /// The flags for the port. These are set when the port is registered with
    /// its client.
    pub fn flags(&self) -> PortFlags {
        let bits = unsafe { j::jack_port_flags(self.port) };
        PortFlags::from_bits(bits as u32).unwrap()
    }

    /// The port type. Jack's built in types include "32 bit float mono audio"
    /// and "8 bit raw midi". Custom types may also be used.
    pub fn port_type<'a>(&self) -> &'a str {
        unsafe { ffi::CStr::from_ptr(j::jack_port_type(self.port)).to_str().unwrap() }
    }

    /// Number of ports connected to/from
    pub fn connected_count(&self) -> usize {
        let n = unsafe { j::jack_port_connected(self.port) };
        n as usize
    }

    /// Returns `true` if the port is directly connected to a port with the name
    /// `port_name`.
    pub fn is_connected_to(&self, port_name: &str) -> bool {
        let res = unsafe {
            let port_name = ffi::CString::new(port_name).unwrap();
            j::jack_port_connected_to(self.port, port_name.as_ptr())
        };
        match res {
            0 => false,
            _ => true,
        }
    }

    /// Full port names to which `self` is connected to. This combines Jack's
    /// `jack_port_get_all_connections()` and `jack_port_get_connections()`. If
    /// the `client` from which `port` was spawned from is the owner, then it
    /// may be used in the graph reordered callback or else it should not be
    /// used.
    pub unsafe fn connections(&self, client: &Client) -> Vec<String> {
        let client_ptr = client.client_ptr();
        let connections_ptr = {
            let ptr = if j::jack_port_is_mine(client_ptr, self.port) == 1 {
                j::jack_port_get_connections(self.port)
            } else {
                j::jack_port_get_all_connections(client_ptr, self.port)
            };
            utils::collect_strs(ptr)
        };
        connections_ptr
    }

    /// Get the alias names for `self`.
    ///
    /// Will return a vector of strings of up to 2 elements.
    ///
    /// # TODO: Implement
    pub fn aliases(&self) -> Vec<String> {
        unimplemented!();
    }

    /// Returns `true` if monitoring has been requested for `self`.
    pub fn is_monitoring_input(&self) -> bool {
        match unsafe { j::jack_port_monitoring_input(self.port) } {
            0 => false,
            _ => true,
        }
    }

    /// If the `CAN_MONITOR` flag is set for the port, then input monitoring is
    /// turned on if it was off, and turns it off if only one request has been
    /// made to turn it on. Otherwise it does nothing.
    pub fn ensure_monitor(&self, enable_monitor: bool) -> Result<(), JackErr> {
        let onoff = match enable_monitor {
            true => 1,
            false => 0,
        };
        let res = unsafe { j::jack_port_ensure_monitor(self.port, onoff) };
        match res {
            0 => Ok(()),
            _ => Err(JackErr::PortMonitorError),
        }
    }
}
