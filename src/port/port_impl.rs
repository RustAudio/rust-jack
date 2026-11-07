use jack_sys as j;
use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::marker::Sized;
use std::sync::atomic::{AtomicBool, Ordering as AtomicOrdering};
use std::sync::{Arc, LazyLock};
use std::{ffi, fmt, iter};

use crate::jack_utils::to_cstring;
use crate::{ClientHandle, Error, Frames, LatencyType, PortFlags};

/// The maximum string length for port names.
pub static PORT_NAME_SIZE: LazyLock<usize> =
    LazyLock::new(|| unsafe { j::jack_port_name_size() - 1 } as usize);

/// The maximum string length for jack type names.
pub static PORT_TYPE_SIZE: LazyLock<usize> =
    LazyLock::new(|| unsafe { j::jack_port_type_size() - 1 } as usize);

/// Defines the configuration for a certain port to JACK, ie 32 bit floating audio input, 8 bit raw
/// midi output, etc...
///
/// # Safety
/// This trait is unsafe because it may depend on interpretting raw bytes.
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
/// A port keeps its JACK client open, even after the original `Client` is dropped.
///
/// Also, ports can be compared and hashed using their raw pointers.
///
/// # Dangling ports
/// A port becomes invalid once it is unregistered, either through [`crate::Client::unregister_port`]
/// or by its owning client, or once the JACK server shuts down. Methods on an invalid port return
/// `Err(Error::InvalidPort)`. Unregistrations by other clients are only detected while this
/// port's client is activated, and only once JACK delivers the notification, so a port may be
/// gone for a short time before it is reported as invalid.
pub struct Port<PS> {
    spec: PS,
    port_ptr: *mut j::jack_port_t,
    state: Arc<PortState>,
    client: Arc<ClientHandle>,
}

/// Liveness shared by every [`Port`] value that refers to the same JACK port.
///
/// The flag only ever goes from valid to invalid. If JACK later reuses the pointer for a new
/// port, the new port gets a fresh `PortState`, so old handles stay invalid.
#[derive(Debug)]
pub(crate) struct PortState {
    valid: AtomicBool,
}

impl PortState {
    pub(crate) fn new() -> Self {
        PortState {
            valid: AtomicBool::new(true),
        }
    }

    #[inline(always)]
    pub(crate) fn is_valid(&self) -> bool {
        self.valid.load(AtomicOrdering::Acquire)
    }

    pub(crate) fn invalidate(&self) {
        self.valid.store(false, AtomicOrdering::Release);
    }
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
            port_ptr: self.port_ptr,
            state: Arc::clone(&self.state),
            client: self.client.clone(),
        }
    }

    /// Returns `false` if the port is known to be dangling, for example because it was
    /// unregistered or the JACK server shut down.
    ///
    /// A `true` result is best effort: the port may have been removed by another client in a way
    /// that has not been reported yet.
    #[inline(always)]
    pub fn is_valid(&self) -> bool {
        self.state.is_valid() && !self.client.is_shut_down()
    }

    /// Run `f` with the port pointer if the port is still valid.
    ///
    /// Call this immediately before the FFI call that uses the pointer, after any preparation
    /// such as string conversion, to keep the window between the check and the use small.
    #[inline(always)]
    pub(crate) fn with_ptr<T>(&self, f: impl FnOnce(*mut j::jack_port_t) -> T) -> Result<T, Error> {
        if self.is_valid() {
            Ok(f(self.port_ptr))
        } else {
            Err(Error::InvalidPort)
        }
    }

    /// Return `value` only if the port is still valid.
    ///
    /// Used after copying out memory that JACK owns, such as port names, in case the port was
    /// removed while the copy was being made.
    #[inline(always)]
    fn recheck<T>(&self, value: T) -> Result<T, Error> {
        if self.is_valid() {
            Ok(value)
        } else {
            Err(Error::InvalidPort)
        }
    }

    /// Returns the full name of the port, including the "client_name:" prefix.
    pub fn name(&self) -> Result<String, Error> {
        let s = self.with_ptr(|p| unsafe {
            ffi::CStr::from_ptr(j::jack_port_name(p))
                .to_string_lossy()
                .into_owned()
        })?;
        self.recheck(s)
    }

    /// Returns the short name of the port, it excludes the "client_name:"
    /// prefix.
    pub fn short_name(&self) -> Result<String, Error> {
        let s = self.with_ptr(|p| unsafe {
            ffi::CStr::from_ptr(j::jack_port_short_name(p))
                .to_string_lossy()
                .into_owned()
        })?;
        self.recheck(s)
    }

    /// The flags for the port. These are set when the port is registered with
    /// its client.
    pub fn flags(&self) -> Result<PortFlags, Error> {
        let bits = self.with_ptr(|p| unsafe { j::jack_port_flags(p) })?;
        Ok(PortFlags::from_bits(bits as j::Enum_JackPortFlags).unwrap())
    }

    /// The port type. JACK's built in types include `"32 bit float mono audio`" and `"8 bit raw
    /// midi"`. Custom types may also be used.
    pub fn port_type(&self) -> Result<String, Error> {
        let s = self.with_ptr(|p| unsafe {
            ffi::CStr::from_ptr(j::jack_port_type(p))
                .to_string_lossy()
                .into_owned()
        })?;
        self.recheck(s)
    }

    /// Number of ports connected to/from `&self`.
    pub fn connected_count(&self) -> Result<usize, Error> {
        let n = self.with_ptr(|p| unsafe { j::jack_port_connected(p) })?;
        Ok(n as usize)
    }

    /// Returns `true` if the port is directly connected to a port with the
    /// name `port_name`.
    pub fn is_connected_to(&self, port_name: &str) -> Result<bool, Error> {
        // No port name contains a NUL byte.
        let Ok(port_name) = ffi::CString::new(port_name) else {
            return Ok(false);
        };
        let res = self.with_ptr(|p| unsafe { j::jack_port_connected_to(p, port_name.as_ptr()) })?;
        match res {
            0 => Ok(false),
            _ => Ok(true),
        }
    }

    /// Get the alias names for `self`.
    ///
    /// Will return up to 2 strings.
    pub fn aliases(&self) -> Result<Vec<String>, Error> {
        let mut a: Vec<libc::c_char> = iter::repeat_n(0, *PORT_NAME_SIZE + 1).collect();
        let mut b = a.clone();
        let mut ptrs: [*mut libc::c_char; 2] = [a.as_mut_ptr(), b.as_mut_ptr()];
        self.with_ptr(|p| unsafe { j::jack_port_get_aliases(p, ptrs.as_mut_ptr()) })?;
        Ok([a, b]
            .iter()
            .map(|p| p.as_ptr())
            .map(|p| unsafe { ffi::CStr::from_ptr(p).to_string_lossy().into_owned() })
            .filter(|s| !s.is_empty())
            .collect())
    }

    /// Returns `true` if monitoring has been requested for `self`.
    pub fn is_monitoring_input(&self) -> Result<bool, Error> {
        match self.with_ptr(|p| unsafe { j::jack_port_monitoring_input(p) })? {
            0 => Ok(false),
            _ => Ok(true),
        }
    }

    /// Turn input monitoring for the port on or off.
    ///
    /// This only works if the port has the `CAN_MONITOR` flag set.
    pub fn request_monitor(&self, enable_monitor: bool) -> Result<(), Error> {
        let onoff = i32::from(enable_monitor);
        let res = self.with_ptr(|p| unsafe { j::jack_port_request_monitor(p, onoff) })?;
        match res {
            0 => Ok(()),
            _ => Err(Error::PortMonitorError),
        }
    }

    /// If the `CAN_MONITOR` flag is set for the port, then input monitoring is turned on if it was
    /// off, and turns it off if only one request has been made to turn it on.  Otherwise it does
    /// nothing.
    pub fn ensure_monitor(&self, enable_monitor: bool) -> Result<(), Error> {
        let onoff = i32::from(enable_monitor);
        let res = self.with_ptr(|p| unsafe { j::jack_port_ensure_monitor(p, onoff) })?;
        match res {
            0 => Ok(()),
            _ => Err(Error::PortMonitorError),
        }
    }

    /// Set's the short name of the port. If the full name is longer than `PORT_NAME_SIZE`, then it
    /// will be truncated.
    pub fn set_name(&mut self, short_name: &str) -> Result<(), Error> {
        let short_name = to_cstring(short_name)?;
        let res = self.with_ptr(|p| unsafe { j::jack_port_set_name(p, short_name.as_ptr()) })?;
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
        let alias = to_cstring(alias)?;
        let res = self.with_ptr(|p| unsafe { j::jack_port_set_alias(p, alias.as_ptr()) })?;
        match res {
            0 => Ok(()),
            _ => Err(Error::PortAliasError),
        }
    }

    /// Remove `alias` as an alias for port. May be called at any time.
    ///
    /// After a successful call, `alias` can no longer be used as an alternate name for `self`.
    pub fn unset_alias(&mut self, alias: &str) -> Result<(), Error> {
        let alias = to_cstring(alias)?;
        let res = self.with_ptr(|p| unsafe { j::jack_port_unset_alias(p, alias.as_ptr()) })?;
        match res {
            0 => Ok(()),
            _ => Err(Error::PortAliasError),
        }
    }

    /// Create a Port from a raw JACK port pointer and a client handle.
    ///
    /// This is mostly for use within the jack crate itself.
    ///
    /// # Safety
    /// The pointer must refer to a valid port, and `client` must be the handle
    /// of the client through which the port was obtained.
    pub unsafe fn from_raw(
        spec: PS,
        port_ptr: *mut j::jack_port_t,
        client: Arc<ClientHandle>,
    ) -> Self {
        let state = client.get_or_create_port_state(port_ptr);
        Port {
            spec,
            port_ptr,
            state,
            client,
        }
    }

    /// Obtain the client pointer that spawned this port.
    ///
    /// This is mostly for use within the jack crate itself.
    /// Keep this port or another strong reference to the client alive while using the pointer.
    #[inline(always)]
    pub fn client_ptr(&self) -> *mut j::jack_client_t {
        self.client.as_ptr()
    }

    /// Obtain the ffi port pointer.
    ///
    /// This is mostly for use within the jack crate itself. The pointer is returned even if the
    /// port is dangling; check [`Port::is_valid`] before passing it to JACK.
    #[inline(always)]
    pub fn raw(&self) -> *mut j::jack_port_t {
        self.port_ptr
    }

    /// Obtain the buffer that the Port is holding. For standard audio and midi ports, consider
    /// using the `AudioInPort`, `AudioOutPort`, `MidiInPort`, or `MidiOutPort` adapter. For more
    /// custom data, consider implementing your own adapter that safely uses the `Port::buffer`
    /// method.
    ///
    /// Returns a null pointer if the port is dangling.
    ///
    /// # Safety
    /// It is unsafe to extract a buffer. Prefer to use one of the type specific wrappers such as
    /// `.as_mut_slice()` for audio and `.midi_iter` for midi.
    #[inline(always)]
    pub unsafe fn buffer(&self, n_frames: Frames) -> *mut libc::c_void {
        self.with_ptr(|p| unsafe { j::jack_port_get_buffer(p, n_frames) })
            .unwrap_or(std::ptr::null_mut())
    }

    /// Set the minimum and maximum latencies defined by mode for port, in frames.
    /// The `range` argument is a tuple of `(min, max)`.
    ///
    /// See [Managing and determining latency](https://jackaudio.org/api/group__LatencyFunctions.html)
    /// for a description of what the latency values mean and some best practices. This function should
    /// **only** be used inside a latency callback.
    #[inline(always)]
    pub fn set_latency_range(
        &self,
        mode: LatencyType,
        range: (Frames, Frames),
    ) -> Result<(), Error> {
        let mut ffi_range = j::Struct__jack_latency_range {
            min: range.0,
            max: range.1,
        };
        self.with_ptr(|p| unsafe {
            j::jack_port_set_latency_range(p, mode.to_ffi(), &mut ffi_range)
        })
    }

    /// Returns a tuple of the minimum and maximum latencies defined by mode for port, in frames.
    ///
    /// See [Managing and determining latency](https://jackaudio.org/api/group__LatencyFunctions.html)
    /// for a description of what the latency values mean and some best practices. This is normally
    /// used in the LatencyCallback. and therefore safe to execute from callbacks.
    #[inline(always)]
    pub fn get_latency_range(&self, mode: LatencyType) -> Result<(Frames, Frames), Error> {
        let mut ffi_range = j::Struct__jack_latency_range { min: 0, max: 0 };
        self.with_ptr(|p| unsafe {
            j::jack_port_get_latency_range(p, mode.to_ffi(), &mut ffi_range)
        })?;
        Ok((ffi_range.min, ffi_range.max))
    }
}

impl<PS> Port<PS> {
    /// Returns the fully-qualified name of all ports currently connected to this one
    /// Remarks: Not realtime safe
    pub fn get_connections(&self) -> Result<Vec<String>, Error> {
        // `jack_port_get_connections` returns memory owned by the caller, so no re-check is
        // needed after collecting it.
        let names = self.with_ptr(|p| unsafe { j::jack_port_get_connections(p) })?;
        Ok(unsafe { crate::jack_utils::collect_strs(names) })
    }

    /// Returns the UUIDs of all clients currently connected to this one
    /// Remarks: Not realtime safe
    pub fn get_connected_client_uuids(&self) -> Result<Vec<jack_sys::jack_uuid_t>, Error> {
        Ok(self
            .get_connections()?
            .into_iter()
            .map(|name| {
                let (client_name, _port) = name.split_once(':').unwrap();
                crate::Client::uuid_of_client_by_name_raw(self.client.as_ptr(), client_name)
                    .unwrap()
            })
            .collect())
    }
}

/// `PortSpec` for a port that holds has no readable or writeable data from JACK on the created
/// client. It can be used to connect ports or to obtain metadata.
///
/// `Unowned` describes ports obtained with `Client::port_by_name` or `Client::port_by_id`. It
/// cannot be used to register a port: `Client::register_port` returns
/// `Err(Error::PortRegistrationError)`.
#[derive(Debug, Default)]
pub struct Unowned;

unsafe impl PortSpec for Unowned {
    /// Empty, since the `Unowned` spec can't be used to create ports.
    fn jack_port_type(&self) -> &str {
        ""
    }

    /// Empty, since the `Unowned` spec can't be used to create ports.
    fn jack_flags(&self) -> PortFlags {
        PortFlags::empty()
    }

    /// Zero, since the `Unowned` spec can't be used to create ports.
    fn jack_buffer_size(&self) -> libc::c_ulong {
        0
    }
}

impl<PS: PortSpec> Debug for Port<PS> {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Port")
            .field("name", &self.name())
            .field("connections", &self.connected_count().unwrap_or(0))
            .field("port_type", &self.port_type())
            .field("port_flags", &self.flags())
            .field("aliases", &self.aliases().unwrap_or_else(|_| Vec::new()))
            .field("is_monitoring_input", &self.is_monitoring_input())
            .finish()
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
        self.port_ptr.cmp(&other.port_ptr)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AudioIn, AudioOut, Client, MidiOut, NotificationHandler, PortId, ProcessScope, RawMidi,
    };
    use std::sync::mpsc::{self, Sender};
    use std::time::{Duration, Instant};

    fn wait_until(mut condition: impl FnMut() -> bool) -> bool {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if condition() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        condition()
    }

    #[derive(Debug, PartialEq)]
    enum PortEvent {
        Registered(PortId),
        Removed {
            id: PortId,
            old_handle_invalid: bool,
            absent_by_id: bool,
            absent_by_name: bool,
        },
    }

    struct RegistrationObserver {
        name: String,
        tracked: Option<(PortId, Port<Unowned>)>,
        events: Sender<PortEvent>,
    }

    impl NotificationHandler for RegistrationObserver {
        fn port_registration(&mut self, client: &Client, id: PortId, register: bool) {
            if register {
                if let Some(port) = client.port_by_id(id) {
                    if port.name().as_deref() == Ok(self.name.as_str()) {
                        self.tracked = Some((id, port));
                        let _ = self.events.send(PortEvent::Registered(id));
                    }
                }
            } else if self
                .tracked
                .as_ref()
                .is_some_and(|(old_id, _)| *old_id == id)
            {
                let (_, old) = self.tracked.take().unwrap();
                // Report results to the test thread: panics in callbacks are caught by the crate.
                let event = PortEvent::Removed {
                    id,
                    old_handle_invalid: !old.is_valid() && old.name() == Err(Error::InvalidPort),
                    absent_by_id: client.port_by_id(id).is_none(),
                    absent_by_name: client.port_by_name(&self.name).is_none(),
                };
                drop(old);
                let _ = self.events.send(event);
            }
        }
    }

    #[test]
    fn ports_keep_client_alive_until_last_port_is_dropped() {
        let (client, _) = Client::new("", Default::default()).unwrap();
        let port = client
            .register_port("retained", AudioIn::default())
            .unwrap();
        let life = Arc::downgrade(&port.client);
        let name = port.name().unwrap();
        let unowned = client.port_by_name(&name).unwrap();
        let clone = port.clone_unowned();
        let ptr = client.raw();

        std::thread::spawn(move || drop(client)).join().unwrap();
        assert_eq!(port.name(), Ok(name.clone()));
        assert_eq!(port.client_ptr(), ptr);
        assert_eq!(unowned.client_ptr(), ptr);
        assert_eq!(clone.client_ptr(), ptr);

        drop(port);
        drop(unowned);
        assert_eq!(clone.name(), Ok(name));
        assert!(life.upgrade().is_some());

        drop(clone);
        assert!(life.upgrade().is_none());
    }

    #[test]
    fn unregistering_a_port_invalidates_every_handle_to_it() {
        let (client, _) = Client::new("", Default::default()).unwrap();
        let port = client.register_port("p", AudioIn::default()).unwrap();
        let name = port.name().unwrap();
        let clone = port.clone_unowned();
        let by_name = client.port_by_name(&name).unwrap();
        assert!(clone.is_valid() && by_name.is_valid());

        client.unregister_port(port).unwrap();
        for p in [&clone, &by_name] {
            assert!(!p.is_valid());
            assert_eq!(p.name(), Err(Error::InvalidPort));
            assert_eq!(p.flags(), Err(Error::InvalidPort));
            assert_eq!(p.get_connections(), Err(Error::InvalidPort));
            assert_eq!(
                p.get_latency_range(LatencyType::Capture),
                Err(Error::InvalidPort)
            );
            assert!(!client.is_mine(p));
            assert_eq!(client.disconnect(p), Err(Error::InvalidPort));
        }
        assert_eq!(
            client.unregister_port(clone),
            Err(Error::InvalidPort),
            "a dangling port must not reach jack_port_unregister"
        );
    }

    #[test]
    fn reregistering_a_port_name_gives_a_fresh_valid_port() {
        let (client, _) = Client::new("", Default::default()).unwrap();
        let port = client.register_port("p", AudioIn::default()).unwrap();
        let stale = port.clone_unowned();
        client.unregister_port(port).unwrap();

        let port = client.register_port("p", AudioIn::default()).unwrap();
        let fresh = client.port_by_name(&port.name().unwrap()).unwrap();
        assert!(fresh.is_valid());
        assert!(
            !stale.is_valid(),
            "an invalid port stays invalid, even if JACK reuses its pointer"
        );
    }

    #[test]
    fn removed_port_lookups_stay_absent_after_all_handles_are_dropped() {
        let (client, _) = Client::new("", Default::default()).unwrap();
        let name = format!("{}:p", client.name());
        let (events, received) = mpsc::channel();
        let client = client
            .activate_async(
                RegistrationObserver {
                    name: name.clone(),
                    tracked: None,
                    events,
                },
                (),
            )
            .unwrap();
        let client = client.as_client();
        let port = client.register_port("p", AudioIn::default()).unwrap();
        let PortEvent::Registered(id) = received.recv_timeout(Duration::from_secs(2)).unwrap()
        else {
            panic!("expected registration");
        };
        let stale = client.port_by_id(id).unwrap();
        client.unregister_port(port).unwrap();
        assert_eq!(
            received.recv_timeout(Duration::from_secs(2)).unwrap(),
            PortEvent::Removed {
                id,
                old_handle_invalid: true,
                absent_by_id: true,
                absent_by_name: true,
            }
        );

        for _ in 0..2 {
            assert!(client.port_by_id(id).is_none());
            assert!(client.port_by_name(&name).is_none());
        }
        drop(stale);
        assert!(client.port_by_id(id).is_none());
        assert!(client.port_by_name(&name).is_none());
    }

    #[test]
    fn failing_to_unregister_a_foreign_port_leaves_it_valid() {
        let (owner, _) = Client::new("", Default::default()).unwrap();
        let (other, _) = Client::new("", Default::default()).unwrap();
        let port = owner.register_port("p", AudioIn::default()).unwrap();
        // Some servers only publish ports of active clients.
        let _owner = owner.activate_async((), ()).unwrap();
        let name = port.name().unwrap();
        assert!(wait_until(|| other.port_by_name(&name).is_some()));
        let foreign = other.port_by_name(&name).unwrap();
        let kept = foreign.clone_unowned();

        assert_eq!(
            other.unregister_port(foreign),
            Err(Error::PortUnregistrationError)
        );
        assert!(kept.is_valid());
        assert_eq!(kept.name(), Ok(name));
    }

    #[test]
    fn active_client_detects_foreign_port_unregistration_and_reregistration() {
        let (owner, _) = Client::new("", Default::default()).unwrap();
        let name = format!("{}:p", owner.name());
        let owner = owner.activate_async((), ()).unwrap();
        let (observer, _) = Client::new("", Default::default()).unwrap();
        let (events, received) = mpsc::channel();
        let observer = observer
            .activate_async(
                RegistrationObserver {
                    name: name.clone(),
                    tracked: None,
                    events,
                },
                (),
            )
            .unwrap();
        let port = owner
            .as_client()
            .register_port("p", AudioIn::default())
            .unwrap();
        let PortEvent::Registered(id) = received.recv_timeout(Duration::from_secs(2)).unwrap()
        else {
            panic!("expected registration");
        };
        let foreign = observer.as_client().port_by_name(&name).unwrap();
        assert!(foreign.is_valid());

        owner.as_client().unregister_port(port).unwrap();
        assert_eq!(
            received.recv_timeout(Duration::from_secs(2)).unwrap(),
            PortEvent::Removed {
                id,
                old_handle_invalid: true,
                absent_by_id: true,
                absent_by_name: true,
            }
        );
        assert_eq!(foreign.name(), Err(Error::InvalidPort));
        assert!(observer.as_client().port_by_id(id).is_none());
        assert!(observer.as_client().port_by_name(&name).is_none());

        let _fresh = owner
            .as_client()
            .register_port("p", AudioIn::default())
            .unwrap();
        let PortEvent::Registered(fresh_id) =
            received.recv_timeout(Duration::from_secs(2)).unwrap()
        else {
            panic!("expected reregistration");
        };
        let fresh = observer.as_client().port_by_id(fresh_id).unwrap();
        assert!(fresh.is_valid());
        assert_eq!(fresh.name(), Ok(name.clone()));
        assert!(observer.as_client().port_by_name(&name).unwrap().is_valid());
        assert!(!foreign.is_valid());
    }

    #[test]
    fn dangling_ports_have_no_buffers() {
        let (client, _) = Client::new("", Default::default()).unwrap();
        let audio_in = client.register_port("in", AudioIn::default()).unwrap();
        let audio_out = client.register_port("out", AudioOut::default()).unwrap();
        let midi_out = client.register_port("midi", MidiOut::default()).unwrap();
        // Second handles to the same ports, which stay around after unregistration.
        let (stale_in, mut stale_out, mut stale_midi) = unsafe {
            (
                Port::from_raw(
                    AudioIn::default(),
                    audio_in.raw(),
                    Arc::clone(&audio_in.client),
                ),
                Port::from_raw(
                    AudioOut::default(),
                    audio_out.raw(),
                    Arc::clone(&audio_out.client),
                ),
                Port::from_raw(
                    MidiOut::default(),
                    midi_out.raw(),
                    Arc::clone(&midi_out.client),
                ),
            )
        };
        client.unregister_port(audio_in).unwrap();
        client.unregister_port(audio_out).unwrap();
        client.unregister_port(midi_out).unwrap();

        let ps = unsafe { ProcessScope::from_raw(64, client.raw()) };
        assert!(unsafe { stale_in.buffer(64) }.is_null());
        assert!(stale_in.as_slice(&ps).is_empty());
        assert!(stale_out.as_mut_slice(&ps).is_empty());
        let mut writer = stale_midi.writer(&ps);
        let msg = RawMidi {
            time: 0,
            bytes: &[0x90, 60, 100],
        };
        assert_eq!(writer.write(&msg), Err(Error::InvalidPort));
        assert_eq!(writer.lost_count(), 0);
        assert_eq!(writer.max_event_size(), 0);
    }
}
