#[cfg(not(target_os = "windows"))]
pub type jack_native_thread_t = ::libc::pthread_t;
pub type jack_uuid_t = u64;
pub type jack_shmsize_t = i32;
pub type jack_nframes_t = u32;
pub type jack_time_t = u64;
pub type jack_intclient_t = u64;
pub enum Struct__jack_port {}
pub type jack_port_t = Struct__jack_port;
pub enum Struct__jack_client {}
pub type jack_client_t = Struct__jack_client;
pub type jack_port_id_t = u32;
pub type jack_port_type_id_t = u32;
pub type Enum_JackOptions = ::libc::c_uint;
pub const JackNullOption: ::libc::c_uint = 0;
pub const JackNoStartServer: ::libc::c_uint = 1;
pub const JackUseExactName: ::libc::c_uint = 2;
pub const JackServerName: ::libc::c_uint = 4;
pub const JackLoadName: ::libc::c_uint = 8;
pub const JackLoadInit: ::libc::c_uint = 16;
pub const JackSessionID: ::libc::c_uint = 32;
pub type jack_options_t = Enum_JackOptions;
pub type Enum_JackStatus = ::libc::c_uint;
pub const JackFailure: ::libc::c_uint = 1;
pub const JackInvalidOption: ::libc::c_uint = 2;
pub const JackNameNotUnique: ::libc::c_uint = 4;
pub const JackServerStarted: ::libc::c_uint = 8;
pub const JackServerFailed: ::libc::c_uint = 16;
pub const JackServerError: ::libc::c_uint = 32;
pub const JackNoSuchClient: ::libc::c_uint = 64;
pub const JackLoadFailure: ::libc::c_uint = 128;
pub const JackInitFailure: ::libc::c_uint = 256;
pub const JackShmFailure: ::libc::c_uint = 512;
pub const JackVersionError: ::libc::c_uint = 1024;
pub const JackBackendError: ::libc::c_uint = 2048;
pub const JackClientZombie: ::libc::c_uint = 4096;
pub type jack_status_t = Enum_JackStatus;
pub type Enum_JackLatencyCallbackMode = ::libc::c_uint;
pub const JackCaptureLatency: ::libc::c_uint = 0;
pub const JackPlaybackLatency: ::libc::c_uint = 1;
pub type jack_latency_callback_mode_t = Enum_JackLatencyCallbackMode;
pub type JackLatencyCallback = ::std::option::Option<
    unsafe extern "C" fn(mode: jack_latency_callback_mode_t, arg: *mut ::libc::c_void) -> (),
>;
#[repr(C, packed)]
#[derive(Copy)]
pub struct Struct__jack_latency_range {
    pub min: jack_nframes_t,
    pub max: jack_nframes_t,
}
impl ::std::clone::Clone for Struct__jack_latency_range {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::default::Default for Struct__jack_latency_range {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}
pub type jack_latency_range_t = Struct__jack_latency_range;
pub type JackProcessCallback = ::std::option::Option<
    unsafe extern "C" fn(nframes: jack_nframes_t, arg: *mut ::libc::c_void) -> ::libc::c_int,
>;
pub type JackThreadCallback =
    ::std::option::Option<unsafe extern "C" fn(arg: *mut ::libc::c_void) -> *mut ::libc::c_void>;
pub type JackThreadInitCallback =
    ::std::option::Option<unsafe extern "C" fn(arg: *mut ::libc::c_void) -> ()>;
pub type JackGraphOrderCallback =
    ::std::option::Option<unsafe extern "C" fn(arg: *mut ::libc::c_void) -> ::libc::c_int>;
pub type JackXRunCallback =
    ::std::option::Option<unsafe extern "C" fn(arg: *mut ::libc::c_void) -> ::libc::c_int>;
pub type JackBufferSizeCallback = ::std::option::Option<
    unsafe extern "C" fn(nframes: jack_nframes_t, arg: *mut ::libc::c_void) -> ::libc::c_int,
>;
pub type JackSampleRateCallback = ::std::option::Option<
    unsafe extern "C" fn(nframes: jack_nframes_t, arg: *mut ::libc::c_void) -> ::libc::c_int,
>;
pub type JackPortRegistrationCallback = ::std::option::Option<
    unsafe extern "C" fn(port: jack_port_id_t, arg1: ::libc::c_int, arg: *mut ::libc::c_void) -> (),
>;
pub type JackClientRegistrationCallback = ::std::option::Option<
    unsafe extern "C" fn(
        name: *const ::libc::c_char,
        arg1: ::libc::c_int,
        arg: *mut ::libc::c_void,
    ) -> (),
>;
pub type JackPortConnectCallback = ::std::option::Option<
    unsafe extern "C" fn(
        a: jack_port_id_t,
        b: jack_port_id_t,
        connect: ::libc::c_int,
        arg: *mut ::libc::c_void,
    ) -> (),
>;
pub type JackPortRenameCallback = ::std::option::Option<
    unsafe extern "C" fn(
        port: jack_port_id_t,
        old_name: *const ::libc::c_char,
        new_name: *const ::libc::c_char,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int,
>;
pub type JackFreewheelCallback = ::std::option::Option<
    unsafe extern "C" fn(starting: ::libc::c_int, arg: *mut ::libc::c_void) -> (),
>;
pub type JackShutdownCallback =
    ::std::option::Option<unsafe extern "C" fn(arg: *mut ::libc::c_void) -> ()>;
pub type JackInfoShutdownCallback = ::std::option::Option<
    unsafe extern "C" fn(
        code: jack_status_t,
        reason: *const ::libc::c_char,
        arg: *mut ::libc::c_void,
    ) -> (),
>;
pub type jack_default_audio_sample_t = ::libc::c_float;
pub type Enum_JackPortFlags = ::libc::c_uint;
pub const JackPortIsInput: ::libc::c_uint = 1;
pub const JackPortIsOutput: ::libc::c_uint = 2;
pub const JackPortIsPhysical: ::libc::c_uint = 4;
pub const JackPortCanMonitor: ::libc::c_uint = 8;
pub const JackPortIsTerminal: ::libc::c_uint = 16;
pub type Enum_Unnamed1 = ::libc::c_uint;
pub const JackTransportStopped: ::libc::c_uint = 0;
pub const JackTransportRolling: ::libc::c_uint = 1;
pub const JackTransportLooping: ::libc::c_uint = 2;
pub const JackTransportStarting: ::libc::c_uint = 3;
pub const JackTransportNetStarting: ::libc::c_uint = 4;
pub type jack_transport_state_t = Enum_Unnamed1;
pub type jack_unique_t = u64;
pub type Enum_Unnamed2 = ::libc::c_uint;
pub const JackPositionBBT: ::libc::c_uint = 16;
pub const JackPositionTimecode: ::libc::c_uint = 32;
pub const JackBBTFrameOffset: ::libc::c_uint = 64;
pub const JackAudioVideoRatio: ::libc::c_uint = 128;
pub const JackVideoFrameOffset: ::libc::c_uint = 256;
pub type jack_position_bits_t = Enum_Unnamed2;
#[repr(C, packed)]
#[derive(Copy)]
pub struct Struct__jack_position {
    pub unique_1: jack_unique_t,
    pub usecs: jack_time_t,
    pub frame_rate: jack_nframes_t,
    pub frame: jack_nframes_t,
    pub valid: jack_position_bits_t,
    pub bar: i32,
    pub beat: i32,
    pub tick: i32,
    pub bar_start_tick: ::libc::c_double,
    pub beats_per_bar: ::libc::c_float,
    pub beat_type: ::libc::c_float,
    pub ticks_per_beat: ::libc::c_double,
    pub beats_per_minute: ::libc::c_double,
    pub frame_time: ::libc::c_double,
    pub next_time: ::libc::c_double,
    pub bbt_offset: jack_nframes_t,
    pub audio_frames_per_video_frame: ::libc::c_float,
    pub video_offset: jack_nframes_t,
    pub padding: [i32; 7usize],
    pub unique_2: jack_unique_t,
}
impl ::std::clone::Clone for Struct__jack_position {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::default::Default for Struct__jack_position {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}
pub type jack_position_t = Struct__jack_position;
pub type JackSyncCallback = ::std::option::Option<
    unsafe extern "C" fn(
        state: jack_transport_state_t,
        pos: *mut jack_position_t,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int,
>;
pub type TimebaseCallback = ::std::option::Option<
    unsafe extern "C" fn(
        state: jack_transport_state_t,
        nframes: jack_nframes_t,
        pos: *mut jack_position_t,
        new_pos: ::libc::c_int,
        arg: *mut ::libc::c_void,
    ) -> (),
>;
pub type Enum_Unnamed3 = ::libc::c_uint;
pub const JackTransportState: ::libc::c_uint = 1;
pub const JackTransportPosition: ::libc::c_uint = 2;
pub const JackTransportLoop: ::libc::c_uint = 4;
pub const JackTransportSMPTE: ::libc::c_uint = 8;
pub const JackTransportBBT: ::libc::c_uint = 16;
pub type jack_transport_bits_t = Enum_Unnamed3;
#[repr(C)]
#[derive(Copy)]
pub struct Struct_Unnamed4 {
    pub frame_rate: jack_nframes_t,
    pub usecs: jack_time_t,
    pub valid: jack_transport_bits_t,
    pub transport_state: jack_transport_state_t,
    pub frame: jack_nframes_t,
    pub loop_start: jack_nframes_t,
    pub loop_end: jack_nframes_t,
    pub smpte_offset: ::libc::c_long,
    pub smpte_frame_rate: ::libc::c_float,
    pub bar: ::libc::c_int,
    pub beat: ::libc::c_int,
    pub tick: ::libc::c_int,
    pub bar_start_tick: ::libc::c_double,
    pub beats_per_bar: ::libc::c_float,
    pub beat_type: ::libc::c_float,
    pub ticks_per_beat: ::libc::c_double,
    pub beats_per_minute: ::libc::c_double,
}
impl ::std::clone::Clone for Struct_Unnamed4 {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::default::Default for Struct_Unnamed4 {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}
pub type jack_transport_info_t = Struct_Unnamed4;
#[cfg(not(target_os = "windows"))]
pub type jack_thread_creator_t = ::std::option::Option<
    unsafe extern "C" fn(
        arg1: *mut ::libc::pthread_t,
        arg2: *const ::libc::pthread_attr_t,
        function: ::std::option::Option<
            extern "C" fn(arg1: *mut ::libc::c_void) -> *mut ::libc::c_void,
        >,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int,
>;
pub type Enum_JackSessionEventType = ::libc::c_uint;
pub const JackSessionSave: ::libc::c_uint = 1;
pub const JackSessionSaveAndQuit: ::libc::c_uint = 2;
pub const JackSessionSaveTemplate: ::libc::c_uint = 3;
pub type jack_session_event_type_t = Enum_JackSessionEventType;
pub type Enum_JackSessionFlags = ::libc::c_uint;
pub const JackSessionSaveError: ::libc::c_uint = 1;
pub const JackSessionNeedTerminal: ::libc::c_uint = 2;
pub type jack_session_flags_t = Enum_JackSessionFlags;
#[repr(C)]
#[derive(Copy)]
pub struct Struct__jack_session_event {
    pub _type: jack_session_event_type_t,
    pub session_dir: *const ::libc::c_char,
    pub client_uuid: *const ::libc::c_char,
    pub command_line: *mut ::libc::c_char,
    pub flags: jack_session_flags_t,
    pub future: u32,
}
impl ::std::clone::Clone for Struct__jack_session_event {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::default::Default for Struct__jack_session_event {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}
pub type jack_session_event_t = Struct__jack_session_event;
pub type JackSessionCallback = ::std::option::Option<
    unsafe extern "C" fn(event: *mut jack_session_event_t, arg: *mut ::libc::c_void) -> (),
>;
#[repr(C)]
#[derive(Copy)]
pub struct Struct_Unnamed5 {
    pub uuid: *const ::libc::c_char,
    pub client_name: *const ::libc::c_char,
    pub command: *const ::libc::c_char,
    pub flags: jack_session_flags_t,
}
impl ::std::clone::Clone for Struct_Unnamed5 {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::default::Default for Struct_Unnamed5 {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}
pub type jack_session_command_t = Struct_Unnamed5;
pub type JSList = Struct__JSList;
pub type JCompareFunc = ::std::option::Option<
    unsafe extern "C" fn(a: *mut ::libc::c_void, b: *mut ::libc::c_void) -> ::libc::c_int,
>;
#[repr(C)]
#[derive(Copy)]
pub struct Struct__JSList {
    pub data: *mut ::libc::c_void,
    pub next: *mut JSList,
}
impl ::std::clone::Clone for Struct__JSList {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::default::Default for Struct__JSList {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}
pub type Enum_Unnamed6 = ::libc::c_uint;
pub const JackParamInt: ::libc::c_uint = 1;
pub const JackParamUInt: ::libc::c_uint = 2;
pub const JackParamChar: ::libc::c_uint = 3;
pub const JackParamString: ::libc::c_uint = 4;
pub const JackParamBool: ::libc::c_uint = 5;
pub type jackctl_param_type_t = Enum_Unnamed6;
pub type Enum_Unnamed7 = ::libc::c_uint;
pub const JackMaster: ::libc::c_uint = 1;
pub const JackSlave: ::libc::c_uint = 2;
pub type jackctl_driver_type_t = Enum_Unnamed7;
#[repr(C)]
#[derive(Copy)]
pub struct Union_jackctl_parameter_value {
    pub _bindgen_data_: [u32; 32usize],
}
impl Union_jackctl_parameter_value {
    pub unsafe fn ui(&mut self) -> *mut u32 {
        let raw: *mut u8 = ::std::mem::transmute(&self._bindgen_data_);
        ::std::mem::transmute(raw.offset(0))
    }
    pub unsafe fn i(&mut self) -> *mut i32 {
        let raw: *mut u8 = ::std::mem::transmute(&self._bindgen_data_);
        ::std::mem::transmute(raw.offset(0))
    }
    pub unsafe fn c(&mut self) -> *mut ::libc::c_char {
        let raw: *mut u8 = ::std::mem::transmute(&self._bindgen_data_);
        ::std::mem::transmute(raw.offset(0))
    }
    pub unsafe fn str(&mut self) -> *mut [::libc::c_char; 128usize] {
        let raw: *mut u8 = ::std::mem::transmute(&self._bindgen_data_);
        ::std::mem::transmute(raw.offset(0))
    }
    pub unsafe fn b(&mut self) -> *mut u8 {
        let raw: *mut u8 = ::std::mem::transmute(&self._bindgen_data_);
        ::std::mem::transmute(raw.offset(0))
    }
}
impl ::std::clone::Clone for Union_jackctl_parameter_value {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::default::Default for Union_jackctl_parameter_value {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}
pub enum Struct_jackctl_server {}
pub type jackctl_server_t = Struct_jackctl_server;
pub enum Struct_jackctl_driver {}
pub type jackctl_driver_t = Struct_jackctl_driver;
pub enum Struct_jackctl_internal {}
pub type jackctl_internal_t = Struct_jackctl_internal;
pub enum Struct_jackctl_parameter {}
pub type jackctl_parameter_t = Struct_jackctl_parameter;
pub enum Struct_jackctl_sigmask {}
pub type jackctl_sigmask_t = Struct_jackctl_sigmask;
#[repr(C)]
#[derive(Copy)]
pub struct Struct_Unnamed8 {
    pub key: *const ::libc::c_char,
    pub data: *const ::libc::c_char,
    pub _type: *const ::libc::c_char,
}
impl ::std::clone::Clone for Struct_Unnamed8 {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::default::Default for Struct_Unnamed8 {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}
pub type jack_property_t = Struct_Unnamed8;
#[repr(C)]
#[derive(Copy)]
pub struct Struct_Unnamed9 {
    pub subject: jack_uuid_t,
    pub property_cnt: u32,
    pub properties: *mut jack_property_t,
    pub property_size: u32,
}
impl ::std::clone::Clone for Struct_Unnamed9 {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::default::Default for Struct_Unnamed9 {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}
pub type jack_description_t = Struct_Unnamed9;
pub type Enum_Unnamed10 = ::libc::c_uint;
pub const PropertyCreated: ::libc::c_uint = 0;
pub const PropertyChanged: ::libc::c_uint = 1;
pub const PropertyDeleted: ::libc::c_uint = 2;
pub type jack_property_change_t = Enum_Unnamed10;
pub type JackPropertyChangeCallback = ::std::option::Option<
    unsafe extern "C" fn(
        subject: jack_uuid_t,
        key: *const ::libc::c_char,
        change: jack_property_change_t,
        arg: *mut ::libc::c_void,
    ) -> (),
>;
pub type jack_midi_data_t = ::libc::c_uchar;
#[repr(C)]
#[derive(Copy)]
pub struct Struct__jack_midi_event {
    pub time: jack_nframes_t,
    pub size: ::libc::size_t,
    pub buffer: *mut jack_midi_data_t,
}
impl ::std::clone::Clone for Struct__jack_midi_event {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::default::Default for Struct__jack_midi_event {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}
pub type jack_midi_event_t = Struct__jack_midi_event;
#[repr(C)]
#[derive(Copy)]
pub struct Struct_Unnamed11 {
    pub buf: *mut ::libc::c_char,
    pub len: ::libc::size_t,
}
impl ::std::clone::Clone for Struct_Unnamed11 {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::default::Default for Struct_Unnamed11 {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}
pub type jack_ringbuffer_data_t = Struct_Unnamed11;
#[repr(C)]
#[derive(Copy)]
pub struct Struct_Unnamed12 {
    pub buf: *mut ::libc::c_char,
    pub write_ptr: ::libc::size_t,
    pub read_ptr: ::libc::size_t,
    pub size: ::libc::size_t,
    pub size_mask: ::libc::size_t,
    pub mlocked: ::libc::c_int,
}
impl ::std::clone::Clone for Struct_Unnamed12 {
    fn clone(&self) -> Self {
        *self
    }
}
impl ::std::default::Default for Struct_Unnamed12 {
    fn default() -> Self {
        unsafe { ::std::mem::zeroed() }
    }
}
pub type jack_ringbuffer_t = Struct_Unnamed12;
extern "C" {
    pub static mut jack_error_callback:
        ::std::option::Option<unsafe extern "C" fn(msg: *const ::libc::c_char) -> ()>;
    pub static mut jack_info_callback:
        ::std::option::Option<unsafe extern "C" fn(msg: *const ::libc::c_char) -> ()>;
    pub static mut JACK_METADATA_PRETTY_NAME: *const ::libc::c_char;
    pub static mut JACK_METADATA_HARDWARE: *const ::libc::c_char;
    pub static mut JACK_METADATA_CONNECTED: *const ::libc::c_char;
    pub static mut JACK_METADATA_PORT_GROUP: *const ::libc::c_char;
    pub static mut JACK_METADATA_ICON_SMALL: *const ::libc::c_char;
    pub static mut JACK_METADATA_ICON_LARGE: *const ::libc::c_char;
}
