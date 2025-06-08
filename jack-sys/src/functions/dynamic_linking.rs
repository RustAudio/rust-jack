use crate::types::*;
pub unsafe fn jack_get_cycle_times(
    _client: *const jack_client_t,
    _current_frames: *mut jack_nframes_t,
    _current_usecs: *mut jack_time_t,
    _next_usecs: *mut jack_time_t,
    _period_usecs: *mut ::libc::c_float,
) -> Option<::libc::c_int> {
    None
}
pub unsafe fn jack_internal_client_new(
    _client_name: *const ::libc::c_char,
    _load_name: *const ::libc::c_char,
    _load_init: *const ::libc::c_char,
) -> Option<::libc::c_int> {
    None
}
pub unsafe fn jack_internal_client_close(_client_name: *const ::libc::c_char) -> Option<()> {
    None
}
pub unsafe fn jack_get_client_pid(_name: *const ::libc::c_char) -> Option<::libc::c_int> {
    None
}
pub unsafe fn jack_port_type_id(_port: *const jack_port_t) -> Option<jack_port_type_id_t> {
    None
}
pub unsafe fn jackctl_setup_signals(_flags: ::libc::c_uint) -> Option<*mut jackctl_sigmask_t> {
    None
}
pub unsafe fn jackctl_wait_signals(_signals: *mut jackctl_sigmask_t) -> Option<()> {
    None
}
pub unsafe fn jackctl_server_create(
    _on_device_acquire: ::std::option::Option<
        unsafe extern "C" fn(device_name: *const ::libc::c_char) -> u8,
    >,
    _on_device_release: ::std::option::Option<
        unsafe extern "C" fn(device_name: *const ::libc::c_char) -> (),
    >,
) -> Option<*mut jackctl_server_t> {
    None
}
pub unsafe fn jackctl_server_destroy(_server: *mut jackctl_server_t) -> Option<()> {
    None
}
pub unsafe fn jackctl_server_open(
    _server: *mut jackctl_server_t,
    _driver: *mut jackctl_driver_t,
) -> Option<u8> {
    None
}
pub unsafe fn jackctl_server_start(_server: *mut jackctl_server_t) -> Option<u8> {
    None
}
pub unsafe fn jackctl_server_stop(_server: *mut jackctl_server_t) -> Option<u8> {
    None
}
pub unsafe fn jackctl_server_close(_server: *mut jackctl_server_t) -> Option<u8> {
    None
}
pub unsafe fn jackctl_server_get_drivers_list(
    _server: *mut jackctl_server_t,
) -> Option<*const JSList> {
    None
}
pub unsafe fn jackctl_server_get_parameters(
    _server: *mut jackctl_server_t,
) -> Option<*const JSList> {
    None
}
pub unsafe fn jackctl_server_get_internals_list(
    _server: *mut jackctl_server_t,
) -> Option<*const JSList> {
    None
}
pub unsafe fn jackctl_server_load_internal(
    _server: *mut jackctl_server_t,
    _internal: *mut jackctl_internal_t,
) -> Option<u8> {
    None
}
pub unsafe fn jackctl_server_unload_internal(
    _server: *mut jackctl_server_t,
    _internal: *mut jackctl_internal_t,
) -> Option<u8> {
    None
}
pub unsafe fn jackctl_server_add_slave(
    _server: *mut jackctl_server_t,
    _driver: *mut jackctl_driver_t,
) -> Option<u8> {
    None
}
pub unsafe fn jackctl_server_remove_slave(
    _server: *mut jackctl_server_t,
    _driver: *mut jackctl_driver_t,
) -> Option<u8> {
    None
}
pub unsafe fn jackctl_server_switch_master(
    _server: *mut jackctl_server_t,
    _driver: *mut jackctl_driver_t,
) -> Option<u8> {
    None
}
pub unsafe fn jackctl_driver_get_name(
    _driver: *mut jackctl_driver_t,
) -> Option<*const ::libc::c_char> {
    None
}
pub unsafe fn jackctl_driver_get_type(
    _driver: *mut jackctl_driver_t,
) -> Option<jackctl_driver_type_t> {
    None
}
pub unsafe fn jackctl_driver_get_parameters(
    _driver: *mut jackctl_driver_t,
) -> Option<*const JSList> {
    None
}
pub unsafe fn jackctl_driver_params_parse(
    _driver: *mut jackctl_driver_t,
    _argc: ::libc::c_int,
    _argv: *mut *mut ::libc::c_char,
) -> Option<::libc::c_int> {
    None
}
pub unsafe fn jackctl_internal_get_name(
    _internal: *mut jackctl_internal_t,
) -> Option<*const ::libc::c_char> {
    None
}
pub unsafe fn jackctl_internal_get_parameters(
    _internal: *mut jackctl_internal_t,
) -> Option<*const JSList> {
    None
}
pub unsafe fn jackctl_parameter_get_name(
    _parameter: *mut jackctl_parameter_t,
) -> Option<*const ::libc::c_char> {
    None
}
pub unsafe fn jackctl_parameter_get_short_description(
    _parameter: *mut jackctl_parameter_t,
) -> Option<*const ::libc::c_char> {
    None
}
pub unsafe fn jackctl_parameter_get_long_description(
    _parameter: *mut jackctl_parameter_t,
) -> Option<*const ::libc::c_char> {
    None
}
pub unsafe fn jackctl_parameter_get_type(
    _parameter: *mut jackctl_parameter_t,
) -> Option<jackctl_param_type_t> {
    None
}
pub unsafe fn jackctl_parameter_get_id(
    _parameter: *mut jackctl_parameter_t,
) -> Option<::libc::c_char> {
    None
}
pub unsafe fn jackctl_parameter_is_set(_parameter: *mut jackctl_parameter_t) -> Option<u8> {
    None
}
pub unsafe fn jackctl_parameter_reset(_parameter: *mut jackctl_parameter_t) -> Option<u8> {
    None
}
pub unsafe fn jackctl_parameter_get_value(
    _parameter: *mut jackctl_parameter_t,
) -> Option<Union_jackctl_parameter_value> {
    None
}
pub unsafe fn jackctl_parameter_set_value(
    _parameter: *mut jackctl_parameter_t,
    _value_ptr: *const Union_jackctl_parameter_value,
) -> Option<u8> {
    None
}
pub unsafe fn jackctl_parameter_get_default_value(
    _parameter: *mut jackctl_parameter_t,
) -> Option<Union_jackctl_parameter_value> {
    None
}
pub unsafe fn jackctl_parameter_has_range_constraint(
    _parameter: *mut jackctl_parameter_t,
) -> Option<u8> {
    None
}
pub unsafe fn jackctl_parameter_has_enum_constraint(
    _parameter: *mut jackctl_parameter_t,
) -> Option<u8> {
    None
}
pub unsafe fn jackctl_parameter_get_enum_constraints_count(
    _parameter: *mut jackctl_parameter_t,
) -> Option<u32> {
    None
}
pub unsafe fn jackctl_parameter_get_enum_constraint_value(
    _parameter: *mut jackctl_parameter_t,
    _index: u32,
) -> Option<Union_jackctl_parameter_value> {
    None
}
pub unsafe fn jackctl_parameter_get_enum_constraint_description(
    _parameter: *mut jackctl_parameter_t,
    _index: u32,
) -> Option<*const ::libc::c_char> {
    None
}
pub unsafe fn jackctl_parameter_get_range_constraint(
    _parameter: *mut jackctl_parameter_t,
    _min_ptr: *mut Union_jackctl_parameter_value,
    _max_ptr: *mut Union_jackctl_parameter_value,
) -> Option<()> {
    None
}
pub unsafe fn jackctl_parameter_constraint_is_strict(
    _parameter: *mut jackctl_parameter_t,
) -> Option<u8> {
    None
}
pub unsafe fn jackctl_parameter_constraint_is_fake_value(
    _parameter: *mut jackctl_parameter_t,
) -> Option<u8> {
    None
}
pub unsafe fn jack_get_internal_client_name(
    _client: *mut jack_client_t,
    _intclient: jack_intclient_t,
) -> Option<*mut ::libc::c_char> {
    None
}
pub unsafe fn jack_internal_client_handle(
    _client: *mut jack_client_t,
    _client_name: *const ::libc::c_char,
    _status: *mut jack_status_t,
) -> Option<jack_intclient_t> {
    None
}
pub unsafe fn jack_internal_client_load(
    _client: *mut jack_client_t,
    _client_name: *const ::libc::c_char,
    _options: jack_options_t,
    _status: *mut jack_status_t,
    _load_name: *const ::libc::c_char,
    _load_init: *const ::libc::c_char,
) -> Option<jack_intclient_t> {
    None
}
pub unsafe fn jack_internal_client_unload(
    _client: *mut jack_client_t,
    _intclient: jack_intclient_t,
) -> Option<jack_status_t> {
    None
}
#[link(name = "jack")]
extern "C" {
    pub fn jack_release_timebase(client: *mut jack_client_t) -> ::libc::c_int;
    pub fn jack_set_sync_callback(
        client: *mut jack_client_t,
        sync_callback: JackSyncCallback,
        sync_arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_set_sync_timeout(client: *mut jack_client_t, timeout: jack_time_t)
        -> ::libc::c_int;
    pub fn jack_set_timebase_callback(
        client: *mut jack_client_t,
        conditional: ::libc::c_int,
        timebase_callback: TimebaseCallback,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_transport_locate(
        client: *mut jack_client_t,
        frame: jack_nframes_t,
    ) -> ::libc::c_int;
    pub fn jack_transport_query(
        client: *const jack_client_t,
        pos: *mut jack_position_t,
    ) -> jack_transport_state_t;
    pub fn jack_get_current_transport_frame(client: *const jack_client_t) -> jack_nframes_t;
    pub fn jack_transport_reposition(
        client: *mut jack_client_t,
        pos: *const jack_position_t,
    ) -> ::libc::c_int;
    pub fn jack_transport_start(client: *mut jack_client_t) -> ();
    pub fn jack_transport_stop(client: *mut jack_client_t) -> ();
    pub fn jack_get_transport_info(
        client: *mut jack_client_t,
        tinfo: *mut jack_transport_info_t,
    ) -> ();
    pub fn jack_set_transport_info(
        client: *mut jack_client_t,
        tinfo: *mut jack_transport_info_t,
    ) -> ();
    pub fn jack_client_open(
        client_name: *const ::libc::c_char,
        options: jack_options_t,
        status: *mut jack_status_t,
    ) -> *mut jack_client_t;
    pub fn jack_client_new(client_name: *const ::libc::c_char) -> *mut jack_client_t;
    pub fn jack_client_close(client: *mut jack_client_t) -> ::libc::c_int;
    pub fn jack_client_name_size() -> ::libc::c_int;
    pub fn jack_get_client_name(client: *mut jack_client_t) -> *mut ::libc::c_char;
    pub fn jack_get_uuid_for_client_name(
        client: *mut jack_client_t,
        client_name: *const ::libc::c_char,
    ) -> *mut ::libc::c_char;
    pub fn jack_get_client_name_by_uuid(
        client: *mut jack_client_t,
        client_uuid: *const ::libc::c_char,
    ) -> *mut ::libc::c_char;
    pub fn jack_activate(client: *mut jack_client_t) -> ::libc::c_int;
    pub fn jack_deactivate(client: *mut jack_client_t) -> ::libc::c_int;
    pub fn jack_is_realtime(client: *mut jack_client_t) -> ::libc::c_int;
    pub fn jack_thread_wait(client: *mut jack_client_t, status: ::libc::c_int) -> jack_nframes_t;
    pub fn jack_cycle_wait(client: *mut jack_client_t) -> jack_nframes_t;
    pub fn jack_cycle_signal(client: *mut jack_client_t, status: ::libc::c_int) -> ();
    pub fn jack_set_process_thread(
        client: *mut jack_client_t,
        thread_callback: JackThreadCallback,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_set_thread_init_callback(
        client: *mut jack_client_t,
        thread_init_callback: JackThreadInitCallback,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_on_shutdown(
        client: *mut jack_client_t,
        callback: JackShutdownCallback,
        arg: *mut ::libc::c_void,
    ) -> ();
    pub fn jack_on_info_shutdown(
        client: *mut jack_client_t,
        callback: JackInfoShutdownCallback,
        arg: *mut ::libc::c_void,
    ) -> ();
    pub fn jack_set_process_callback(
        client: *mut jack_client_t,
        process_callback: JackProcessCallback,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_set_freewheel_callback(
        client: *mut jack_client_t,
        freewheel_callback: JackFreewheelCallback,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_set_buffer_size_callback(
        client: *mut jack_client_t,
        bufsize_callback: JackBufferSizeCallback,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_get_sample_rate(client: *mut jack_client_t) -> ::libc::c_int;
    pub fn jack_set_sample_rate_callback(
        client: *mut jack_client_t,
        srate_callback: JackSampleRateCallback,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_set_client_registration_callback(
        client: *mut jack_client_t,
        registration_callback: JackClientRegistrationCallback,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_set_port_registration_callback(
        client: *mut jack_client_t,
        registration_callback: JackPortRegistrationCallback,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_set_port_connect_callback(
        client: *mut jack_client_t,
        connect_callback: JackPortConnectCallback,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_set_port_rename_callback(
        client: *mut jack_client_t,
        rename_callback: JackPortRenameCallback,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_set_graph_order_callback(
        client: *mut jack_client_t,
        graph_callback: JackGraphOrderCallback,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_set_xrun_callback(
        client: *mut jack_client_t,
        xrun_callback: JackXRunCallback,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_set_latency_callback(
        client: *mut jack_client_t,
        latency_callback: JackLatencyCallback,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_set_freewheel(client: *mut jack_client_t, onoff: ::libc::c_int) -> ::libc::c_int;
    pub fn jack_set_buffer_size(
        client: *mut jack_client_t,
        nframes: jack_nframes_t,
    ) -> ::libc::c_int;
    pub fn jack_get_buffer_size(client: *mut jack_client_t) -> jack_nframes_t;
    pub fn jack_engine_takeover_timebase(client: *mut jack_client_t) -> ::libc::c_int;
    pub fn jack_cpu_load(client: *mut jack_client_t) -> ::libc::c_float;
    pub fn jack_port_register(
        client: *mut jack_client_t,
        port_name: *const ::libc::c_char,
        port_type: *const ::libc::c_char,
        flags: ::libc::c_ulong,
        buffer_size: ::libc::c_ulong,
    ) -> *mut jack_port_t;
    pub fn jack_port_unregister(
        client: *mut jack_client_t,
        port: *mut jack_port_t,
    ) -> ::libc::c_int;
    pub fn jack_port_get_buffer(
        port: *mut jack_port_t,
        nframes: jack_nframes_t,
    ) -> *mut ::libc::c_void;
    pub fn jack_port_uuid(port: *mut jack_port_t) -> jack_uuid_t;
    pub fn jack_port_name(port: *mut jack_port_t) -> *const ::libc::c_char;
    pub fn jack_port_short_name(port: *mut jack_port_t) -> *const ::libc::c_char;
    pub fn jack_port_flags(port: *mut jack_port_t) -> ::libc::c_int;
    pub fn jack_port_type(port: *const jack_port_t) -> *const ::libc::c_char;
    pub fn jack_port_is_mine(
        client: *const jack_client_t,
        port: *const jack_port_t,
    ) -> ::libc::c_int;
    pub fn jack_port_connected(port: *const jack_port_t) -> ::libc::c_int;
    pub fn jack_port_connected_to(
        port: *const jack_port_t,
        port_name: *const ::libc::c_char,
    ) -> ::libc::c_int;
    pub fn jack_port_get_connections(port: *const jack_port_t) -> *mut *const ::libc::c_char;
    pub fn jack_port_get_all_connections(
        client: *const jack_client_t,
        port: *const jack_port_t,
    ) -> *mut *const ::libc::c_char;
    pub fn jack_port_tie(src: *mut jack_port_t, dst: *mut jack_port_t) -> ::libc::c_int;
    pub fn jack_port_untie(port: *mut jack_port_t) -> ::libc::c_int;
    pub fn jack_port_set_name(
        port: *mut jack_port_t,
        port_name: *const ::libc::c_char,
    ) -> ::libc::c_int;
    pub fn jack_port_set_alias(
        port: *mut jack_port_t,
        alias: *const ::libc::c_char,
    ) -> ::libc::c_int;
    pub fn jack_port_unset_alias(
        port: *mut jack_port_t,
        alias: *const ::libc::c_char,
    ) -> ::libc::c_int;
    pub fn jack_port_get_aliases(
        port: *const jack_port_t,
        aliases: *mut *mut ::libc::c_char,
    ) -> ::libc::c_int;
    pub fn jack_port_request_monitor(port: *mut jack_port_t, onoff: ::libc::c_int)
        -> ::libc::c_int;
    pub fn jack_port_request_monitor_by_name(
        client: *mut jack_client_t,
        port_name: *const ::libc::c_char,
        onoff: ::libc::c_int,
    ) -> ::libc::c_int;
    pub fn jack_port_ensure_monitor(port: *mut jack_port_t, onoff: ::libc::c_int) -> ::libc::c_int;
    pub fn jack_port_monitoring_input(port: *mut jack_port_t) -> ::libc::c_int;
    pub fn jack_connect(
        client: *mut jack_client_t,
        source_port: *const ::libc::c_char,
        destination_port: *const ::libc::c_char,
    ) -> ::libc::c_int;
    pub fn jack_disconnect(
        client: *mut jack_client_t,
        source_port: *const ::libc::c_char,
        destination_port: *const ::libc::c_char,
    ) -> ::libc::c_int;
    pub fn jack_port_disconnect(
        client: *mut jack_client_t,
        port: *mut jack_port_t,
    ) -> ::libc::c_int;
    pub fn jack_port_name_size() -> ::libc::c_int;
    pub fn jack_port_type_size() -> ::libc::c_int;
    pub fn jack_port_type_get_buffer_size(
        client: *mut jack_client_t,
        port_type: *const ::libc::c_char,
    ) -> ::libc::size_t;
    pub fn jack_port_set_latency(port: *mut jack_port_t, arg1: jack_nframes_t) -> ();
    pub fn jack_port_get_latency_range(
        port: *mut jack_port_t,
        mode: jack_latency_callback_mode_t,
        range: *mut jack_latency_range_t,
    ) -> ();
    pub fn jack_port_set_latency_range(
        port: *mut jack_port_t,
        mode: jack_latency_callback_mode_t,
        range: *mut jack_latency_range_t,
    ) -> ();
    pub fn jack_recompute_total_latencies(client: *mut jack_client_t) -> ::libc::c_int;
    pub fn jack_port_get_latency(port: *mut jack_port_t) -> jack_nframes_t;
    pub fn jack_port_get_total_latency(
        client: *mut jack_client_t,
        port: *mut jack_port_t,
    ) -> jack_nframes_t;
    pub fn jack_recompute_total_latency(
        arg1: *mut jack_client_t,
        port: *mut jack_port_t,
    ) -> ::libc::c_int;
    pub fn jack_get_ports(
        client: *mut jack_client_t,
        port_name_pattern: *const ::libc::c_char,
        type_name_pattern: *const ::libc::c_char,
        flags: ::libc::c_ulong,
    ) -> *mut *const ::libc::c_char;
    pub fn jack_port_by_name(
        client: *mut jack_client_t,
        port_name: *const ::libc::c_char,
    ) -> *mut jack_port_t;
    pub fn jack_port_by_id(client: *mut jack_client_t, port_id: jack_port_id_t)
        -> *mut jack_port_t;
    pub fn jack_frames_since_cycle_start(arg1: *const jack_client_t) -> jack_nframes_t;
    pub fn jack_frame_time(arg1: *const jack_client_t) -> jack_nframes_t;
    pub fn jack_last_frame_time(client: *const jack_client_t) -> jack_nframes_t;
    pub fn jack_frames_to_time(client: *const jack_client_t, arg1: jack_nframes_t) -> jack_time_t;
    pub fn jack_time_to_frames(client: *const jack_client_t, arg1: jack_time_t) -> jack_nframes_t;
    pub fn jack_get_time() -> jack_time_t;
    pub fn jack_set_error_function(
        func: ::std::option::Option<unsafe extern "C" fn(arg1: *const ::libc::c_char) -> ()>,
    ) -> ();
    pub fn jack_set_info_function(
        func: ::std::option::Option<unsafe extern "C" fn(arg1: *const ::libc::c_char) -> ()>,
    ) -> ();
    pub fn jack_free(ptr: *mut ::libc::c_void) -> ();
    pub fn jack_client_real_time_priority(arg1: *mut jack_client_t) -> ::libc::c_int;
    pub fn jack_client_max_real_time_priority(arg1: *mut jack_client_t) -> ::libc::c_int;
    pub fn jack_set_session_callback(
        client: *mut jack_client_t,
        session_callback: JackSessionCallback,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_session_reply(
        client: *mut jack_client_t,
        event: *mut jack_session_event_t,
    ) -> ::libc::c_int;
    pub fn jack_session_event_free(event: *mut jack_session_event_t) -> ();
    pub fn jack_client_get_uuid(client: *mut jack_client_t) -> *mut ::libc::c_char;
    pub fn jack_session_notify(
        client: *mut jack_client_t,
        target: *const ::libc::c_char,
        _type: jack_session_event_type_t,
        path: *const ::libc::c_char,
    ) -> *mut jack_session_command_t;
    pub fn jack_session_commands_free(cmds: *mut jack_session_command_t) -> ();
    pub fn jack_reserve_client_name(
        client: *mut jack_client_t,
        name: *const ::libc::c_char,
        uuid: *const ::libc::c_char,
    ) -> ::libc::c_int;
    pub fn jack_client_has_session_callback(
        client: *mut jack_client_t,
        client_name: *const ::libc::c_char,
    ) -> ::libc::c_int;
    pub fn jack_set_property(
        arg1: *mut jack_client_t,
        subject: jack_uuid_t,
        key: *const ::libc::c_char,
        value: *const ::libc::c_char,
        _type: *const ::libc::c_char,
    ) -> ::libc::c_int;
    pub fn jack_get_property(
        subject: jack_uuid_t,
        key: *const ::libc::c_char,
        value: *mut *mut ::libc::c_char,
        _type: *mut *mut ::libc::c_char,
    ) -> ::libc::c_int;
    pub fn jack_free_description(
        desc: *mut jack_description_t,
        free_description_itself: ::libc::c_int,
    ) -> ();
    pub fn jack_get_properties(
        subject: jack_uuid_t,
        desc: *mut jack_description_t,
    ) -> ::libc::c_int;
    pub fn jack_get_all_properties(descs: *mut *mut jack_description_t) -> ::libc::c_int;
    pub fn jack_remove_property(
        client: *mut jack_client_t,
        subject: jack_uuid_t,
        key: *const ::libc::c_char,
    ) -> ::libc::c_int;
    pub fn jack_remove_properties(
        client: *mut jack_client_t,
        subject: jack_uuid_t,
    ) -> ::libc::c_int;
    pub fn jack_remove_all_properties(client: *mut jack_client_t) -> ::libc::c_int;
    pub fn jack_set_property_change_callback(
        client: *mut jack_client_t,
        callback: JackPropertyChangeCallback,
        arg: *mut ::libc::c_void,
    ) -> ::libc::c_int;
    pub fn jack_get_max_delayed_usecs(client: *mut jack_client_t) -> ::libc::c_float;
    pub fn jack_get_xrun_delayed_usecs(client: *mut jack_client_t) -> ::libc::c_float;
    pub fn jack_reset_max_delayed_usecs(client: *mut jack_client_t) -> ();
    pub fn jack_midi_get_event_count(port_buffer: *mut ::libc::c_void) -> u32;
    pub fn jack_midi_event_get(
        event: *mut jack_midi_event_t,
        port_buffer: *mut ::libc::c_void,
        event_index: u32,
    ) -> ::libc::c_int;
    pub fn jack_midi_clear_buffer(port_buffer: *mut ::libc::c_void) -> ();
    pub fn jack_midi_max_event_size(port_buffer: *mut ::libc::c_void) -> ::libc::size_t;
    pub fn jack_midi_event_reserve(
        port_buffer: *mut ::libc::c_void,
        time: jack_nframes_t,
        data_size: ::libc::size_t,
    ) -> *mut jack_midi_data_t;
    pub fn jack_midi_event_write(
        port_buffer: *mut ::libc::c_void,
        time: jack_nframes_t,
        data: *const jack_midi_data_t,
        data_size: ::libc::size_t,
    ) -> ::libc::c_int;
    pub fn jack_midi_get_lost_event_count(port_buffer: *mut ::libc::c_void) -> u32;
    pub fn jack_ringbuffer_create(sz: ::libc::size_t) -> *mut jack_ringbuffer_t;
    pub fn jack_ringbuffer_free(rb: *mut jack_ringbuffer_t) -> ();
    pub fn jack_ringbuffer_get_read_vector(
        rb: *const jack_ringbuffer_t,
        vec: *mut jack_ringbuffer_data_t,
    ) -> ();
    pub fn jack_ringbuffer_get_write_vector(
        rb: *const jack_ringbuffer_t,
        vec: *mut jack_ringbuffer_data_t,
    ) -> ();
    pub fn jack_ringbuffer_read(
        rb: *mut jack_ringbuffer_t,
        dest: *mut ::libc::c_char,
        cnt: ::libc::size_t,
    ) -> ::libc::size_t;
    pub fn jack_ringbuffer_peek(
        rb: *mut jack_ringbuffer_t,
        dest: *mut ::libc::c_char,
        cnt: ::libc::size_t,
    ) -> ::libc::size_t;
    pub fn jack_ringbuffer_read_advance(rb: *mut jack_ringbuffer_t, cnt: ::libc::size_t) -> ();
    pub fn jack_ringbuffer_read_space(rb: *const jack_ringbuffer_t) -> ::libc::size_t;
    pub fn jack_ringbuffer_mlock(rb: *mut jack_ringbuffer_t) -> ::libc::c_int;
    pub fn jack_ringbuffer_reset(rb: *mut jack_ringbuffer_t) -> ();
    pub fn jack_ringbuffer_write(
        rb: *mut jack_ringbuffer_t,
        src: *const ::libc::c_char,
        cnt: ::libc::size_t,
    ) -> ::libc::size_t;
    pub fn jack_ringbuffer_write_advance(rb: *mut jack_ringbuffer_t, cnt: ::libc::size_t) -> ();
    pub fn jack_ringbuffer_write_space(rb: *const jack_ringbuffer_t) -> ::libc::size_t;
    pub fn jack_uuid_to_index(arg1: jack_uuid_t) -> u32;
    pub fn jack_uuid_compare(arg1: jack_uuid_t, arg2: jack_uuid_t) -> ::std::os::raw::c_int;
    pub fn jack_uuid_copy(dst: *mut jack_uuid_t, src: jack_uuid_t) -> ();
    pub fn jack_uuid_clear(arg1: *mut jack_uuid_t) -> ();
    pub fn jack_uuid_parse(
        buf: *const ::std::os::raw::c_char,
        arg1: *mut jack_uuid_t,
    ) -> ::std::os::raw::c_int;
    pub fn jack_uuid_unparse(arg1: jack_uuid_t, buf: *mut ::std::os::raw::c_char) -> ();
    pub fn jack_uuid_empty(arg1: jack_uuid_t) -> ::std::os::raw::c_int;
}
