use crate::types::*;
use lazy_static::lazy_static;

pub struct JackFunctions {
    jack_release_timebase_impl: unsafe extern "C" fn(*mut jack_client_t) -> ::libc::c_int,
    jack_get_cycle_times_impl: Option<
        unsafe extern "C" fn(
            *const jack_client_t,
            *mut jack_nframes_t,
            *mut jack_time_t,
            *mut jack_time_t,
            *mut ::libc::c_float,
        ) -> ::libc::c_int,
    >,
    jack_set_sync_callback_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackSyncCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_set_sync_timeout_impl:
        unsafe extern "C" fn(*mut jack_client_t, jack_time_t) -> ::libc::c_int,
    jack_set_timebase_callback_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        ::libc::c_int,
        TimebaseCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_transport_locate_impl:
        unsafe extern "C" fn(*mut jack_client_t, jack_nframes_t) -> ::libc::c_int,
    jack_transport_query_impl:
        unsafe extern "C" fn(*const jack_client_t, *mut jack_position_t) -> jack_transport_state_t,
    jack_get_current_transport_frame_impl:
        unsafe extern "C" fn(*const jack_client_t) -> jack_nframes_t,
    jack_transport_reposition_impl:
        unsafe extern "C" fn(*mut jack_client_t, *const jack_position_t) -> ::libc::c_int,
    jack_transport_start_impl: unsafe extern "C" fn(*mut jack_client_t) -> (),
    jack_transport_stop_impl: unsafe extern "C" fn(*mut jack_client_t) -> (),
    jack_get_transport_info_impl:
        unsafe extern "C" fn(*mut jack_client_t, *mut jack_transport_info_t) -> (),
    jack_set_transport_info_impl:
        unsafe extern "C" fn(*mut jack_client_t, *mut jack_transport_info_t) -> (),
    jack_client_open_impl: unsafe extern "C" fn(
        *const ::libc::c_char,
        jack_options_t,
        *mut jack_status_t,
    ) -> *mut jack_client_t,
    jack_client_new_impl: unsafe extern "C" fn(*const ::libc::c_char) -> *mut jack_client_t,
    jack_client_close_impl: unsafe extern "C" fn(*mut jack_client_t) -> ::libc::c_int,
    jack_client_name_size_impl: unsafe extern "C" fn() -> ::libc::c_int,
    jack_get_client_name_impl: unsafe extern "C" fn(*mut jack_client_t) -> *mut ::libc::c_char,
    jack_get_uuid_for_client_name_impl:
        unsafe extern "C" fn(*mut jack_client_t, *const ::libc::c_char) -> *mut ::libc::c_char,
    jack_get_client_name_by_uuid_impl:
        unsafe extern "C" fn(*mut jack_client_t, *const ::libc::c_char) -> *mut ::libc::c_char,
    jack_internal_client_new_impl: Option<
        unsafe extern "C" fn(
            *const ::libc::c_char,
            *const ::libc::c_char,
            *const ::libc::c_char,
        ) -> ::libc::c_int,
    >,
    jack_internal_client_close_impl: Option<unsafe extern "C" fn(*const ::libc::c_char) -> ()>,
    jack_activate_impl: unsafe extern "C" fn(*mut jack_client_t) -> ::libc::c_int,
    jack_deactivate_impl: unsafe extern "C" fn(*mut jack_client_t) -> ::libc::c_int,
    jack_get_client_pid_impl: Option<unsafe extern "C" fn(*const ::libc::c_char) -> ::libc::c_int>,
    jack_is_realtime_impl: unsafe extern "C" fn(*mut jack_client_t) -> ::libc::c_int,
    jack_thread_wait_impl:
        unsafe extern "C" fn(*mut jack_client_t, ::libc::c_int) -> jack_nframes_t,
    jack_cycle_wait_impl: unsafe extern "C" fn(*mut jack_client_t) -> jack_nframes_t,
    jack_cycle_signal_impl: unsafe extern "C" fn(*mut jack_client_t, ::libc::c_int) -> (),
    jack_set_process_thread_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackThreadCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_set_thread_init_callback_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackThreadInitCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_on_shutdown_impl:
        unsafe extern "C" fn(*mut jack_client_t, JackShutdownCallback, *mut ::libc::c_void) -> (),
    jack_on_info_shutdown_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackInfoShutdownCallback,
        *mut ::libc::c_void,
    ) -> (),
    jack_set_process_callback_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackProcessCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_set_freewheel_callback_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackFreewheelCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_set_buffer_size_callback_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackBufferSizeCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_get_sample_rate_impl: unsafe extern "C" fn(*mut jack_client_t) -> ::libc::c_int,
    jack_set_sample_rate_callback_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackSampleRateCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_set_client_registration_callback_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackClientRegistrationCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_set_port_registration_callback_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackPortRegistrationCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_set_port_connect_callback_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackPortConnectCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_set_port_rename_callback_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackPortRenameCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_set_graph_order_callback_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackGraphOrderCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_set_xrun_callback_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackXRunCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_set_latency_callback_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackLatencyCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_set_freewheel_impl:
        unsafe extern "C" fn(*mut jack_client_t, ::libc::c_int) -> ::libc::c_int,
    jack_set_buffer_size_impl:
        unsafe extern "C" fn(*mut jack_client_t, jack_nframes_t) -> ::libc::c_int,
    jack_get_buffer_size_impl: unsafe extern "C" fn(*mut jack_client_t) -> jack_nframes_t,
    jack_engine_takeover_timebase_impl: unsafe extern "C" fn(*mut jack_client_t) -> ::libc::c_int,
    jack_cpu_load_impl: unsafe extern "C" fn(*mut jack_client_t) -> ::libc::c_float,
    jack_port_register_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        *const ::libc::c_char,
        *const ::libc::c_char,
        ::libc::c_ulong,
        ::libc::c_ulong,
    ) -> *mut jack_port_t,
    jack_port_unregister_impl:
        unsafe extern "C" fn(*mut jack_client_t, *mut jack_port_t) -> ::libc::c_int,
    jack_port_get_buffer_impl:
        unsafe extern "C" fn(*mut jack_port_t, jack_nframes_t) -> *mut ::libc::c_void,
    jack_port_uuid_impl: unsafe extern "C" fn(*mut jack_port_t) -> jack_uuid_t,
    jack_port_name_impl: unsafe extern "C" fn(*mut jack_port_t) -> *const ::libc::c_char,
    jack_port_short_name_impl: unsafe extern "C" fn(*mut jack_port_t) -> *const ::libc::c_char,
    jack_port_flags_impl: unsafe extern "C" fn(*mut jack_port_t) -> ::libc::c_int,
    jack_port_type_impl: unsafe extern "C" fn(*const jack_port_t) -> *const ::libc::c_char,
    jack_port_type_id_impl: Option<unsafe extern "C" fn(*const jack_port_t) -> jack_port_type_id_t>,
    jack_port_is_mine_impl:
        unsafe extern "C" fn(*const jack_client_t, *const jack_port_t) -> ::libc::c_int,
    jack_port_connected_impl: unsafe extern "C" fn(*const jack_port_t) -> ::libc::c_int,
    jack_port_connected_to_impl:
        unsafe extern "C" fn(*const jack_port_t, *const ::libc::c_char) -> ::libc::c_int,
    jack_port_get_connections_impl:
        unsafe extern "C" fn(*const jack_port_t) -> *mut *const ::libc::c_char,
    jack_port_get_all_connections_impl: unsafe extern "C" fn(
        *const jack_client_t,
        *const jack_port_t,
    ) -> *mut *const ::libc::c_char,
    jack_port_tie_impl: unsafe extern "C" fn(*mut jack_port_t, *mut jack_port_t) -> ::libc::c_int,
    jack_port_untie_impl: unsafe extern "C" fn(*mut jack_port_t) -> ::libc::c_int,
    jack_port_set_name_impl:
        unsafe extern "C" fn(*mut jack_port_t, *const ::libc::c_char) -> ::libc::c_int,
    jack_port_set_alias_impl:
        unsafe extern "C" fn(*mut jack_port_t, *const ::libc::c_char) -> ::libc::c_int,
    jack_port_unset_alias_impl:
        unsafe extern "C" fn(*mut jack_port_t, *const ::libc::c_char) -> ::libc::c_int,
    jack_port_get_aliases_impl:
        unsafe extern "C" fn(*const jack_port_t, *mut *mut ::libc::c_char) -> ::libc::c_int,
    jack_port_request_monitor_impl:
        unsafe extern "C" fn(*mut jack_port_t, ::libc::c_int) -> ::libc::c_int,
    jack_port_request_monitor_by_name_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        *const ::libc::c_char,
        ::libc::c_int,
    ) -> ::libc::c_int,
    jack_port_ensure_monitor_impl:
        unsafe extern "C" fn(*mut jack_port_t, ::libc::c_int) -> ::libc::c_int,
    jack_port_monitoring_input_impl: unsafe extern "C" fn(*mut jack_port_t) -> ::libc::c_int,
    jack_connect_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        *const ::libc::c_char,
        *const ::libc::c_char,
    ) -> ::libc::c_int,
    jack_disconnect_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        *const ::libc::c_char,
        *const ::libc::c_char,
    ) -> ::libc::c_int,
    jack_port_disconnect_impl:
        unsafe extern "C" fn(*mut jack_client_t, *mut jack_port_t) -> ::libc::c_int,
    jack_port_name_size_impl: unsafe extern "C" fn() -> ::libc::c_int,
    jack_port_type_size_impl: unsafe extern "C" fn() -> ::libc::c_int,
    jack_port_type_get_buffer_size_impl:
        unsafe extern "C" fn(*mut jack_client_t, *const ::libc::c_char) -> ::libc::size_t,
    jack_port_set_latency_impl: unsafe extern "C" fn(*mut jack_port_t, jack_nframes_t) -> (),
    jack_port_get_latency_range_impl: unsafe extern "C" fn(
        *mut jack_port_t,
        jack_latency_callback_mode_t,
        *mut jack_latency_range_t,
    ) -> (),
    jack_port_set_latency_range_impl: unsafe extern "C" fn(
        *mut jack_port_t,
        jack_latency_callback_mode_t,
        *mut jack_latency_range_t,
    ) -> (),
    jack_recompute_total_latencies_impl: unsafe extern "C" fn(*mut jack_client_t) -> ::libc::c_int,
    jack_port_get_latency_impl: unsafe extern "C" fn(*mut jack_port_t) -> jack_nframes_t,
    jack_port_get_total_latency_impl:
        unsafe extern "C" fn(*mut jack_client_t, *mut jack_port_t) -> jack_nframes_t,
    jack_recompute_total_latency_impl:
        unsafe extern "C" fn(*mut jack_client_t, *mut jack_port_t) -> ::libc::c_int,
    jack_get_ports_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        *const ::libc::c_char,
        *const ::libc::c_char,
        ::libc::c_ulong,
    ) -> *mut *const ::libc::c_char,
    jack_port_by_name_impl:
        unsafe extern "C" fn(*mut jack_client_t, *const ::libc::c_char) -> *mut jack_port_t,
    jack_port_by_id_impl:
        unsafe extern "C" fn(*mut jack_client_t, jack_port_id_t) -> *mut jack_port_t,
    jack_frames_since_cycle_start_impl:
        unsafe extern "C" fn(*const jack_client_t) -> jack_nframes_t,
    jack_frame_time_impl: unsafe extern "C" fn(*const jack_client_t) -> jack_nframes_t,
    jack_last_frame_time_impl: unsafe extern "C" fn(*const jack_client_t) -> jack_nframes_t,
    jack_frames_to_time_impl:
        unsafe extern "C" fn(*const jack_client_t, jack_nframes_t) -> jack_time_t,
    jack_time_to_frames_impl:
        unsafe extern "C" fn(*const jack_client_t, jack_time_t) -> jack_nframes_t,
    jack_get_time_impl: unsafe extern "C" fn() -> jack_time_t,
    jack_set_error_function_impl: unsafe extern "C" fn(
        ::std::option::Option<unsafe extern "C" fn(arg1: *const ::libc::c_char) -> ()>,
    ) -> (),
    jack_set_info_function_impl: unsafe extern "C" fn(
        ::std::option::Option<unsafe extern "C" fn(arg1: *const ::libc::c_char) -> ()>,
    ) -> (),
    jack_free_impl: unsafe extern "C" fn(*mut ::libc::c_void) -> (),
    jack_client_real_time_priority_impl: unsafe extern "C" fn(*mut jack_client_t) -> ::libc::c_int,
    jack_client_max_real_time_priority_impl:
        unsafe extern "C" fn(*mut jack_client_t) -> ::libc::c_int,
    jack_set_session_callback_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackSessionCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_session_reply_impl:
        unsafe extern "C" fn(*mut jack_client_t, *mut jack_session_event_t) -> ::libc::c_int,
    jack_session_event_free_impl: unsafe extern "C" fn(*mut jack_session_event_t) -> (),
    jack_client_get_uuid_impl: unsafe extern "C" fn(*mut jack_client_t) -> *mut ::libc::c_char,
    jack_session_notify_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        *const ::libc::c_char,
        jack_session_event_type_t,
        *const ::libc::c_char,
    ) -> *mut jack_session_command_t,
    jack_session_commands_free_impl: unsafe extern "C" fn(*mut jack_session_command_t) -> (),
    jack_reserve_client_name_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        *const ::libc::c_char,
        *const ::libc::c_char,
    ) -> ::libc::c_int,
    jack_client_has_session_callback_impl:
        unsafe extern "C" fn(*mut jack_client_t, *const ::libc::c_char) -> ::libc::c_int,
    jackctl_setup_signals_impl:
        Option<unsafe extern "C" fn(::libc::c_uint) -> *mut jackctl_sigmask_t>,
    jackctl_wait_signals_impl: Option<unsafe extern "C" fn(*mut jackctl_sigmask_t) -> ()>,
    jackctl_server_create_impl: Option<
        unsafe extern "C" fn(
            ::std::option::Option<unsafe extern "C" fn(device_name: *const ::libc::c_char) -> u8>,
            ::std::option::Option<unsafe extern "C" fn(device_name: *const ::libc::c_char) -> ()>,
        ) -> *mut jackctl_server_t,
    >,
    jackctl_server_destroy_impl: Option<unsafe extern "C" fn(*mut jackctl_server_t) -> ()>,
    jackctl_server_open_impl:
        Option<unsafe extern "C" fn(*mut jackctl_server_t, *mut jackctl_driver_t) -> u8>,
    jackctl_server_start_impl: Option<unsafe extern "C" fn(*mut jackctl_server_t) -> u8>,
    jackctl_server_stop_impl: Option<unsafe extern "C" fn(*mut jackctl_server_t) -> u8>,
    jackctl_server_close_impl: Option<unsafe extern "C" fn(*mut jackctl_server_t) -> u8>,
    jackctl_server_get_drivers_list_impl:
        Option<unsafe extern "C" fn(*mut jackctl_server_t) -> *const JSList>,
    jackctl_server_get_parameters_impl:
        Option<unsafe extern "C" fn(*mut jackctl_server_t) -> *const JSList>,
    jackctl_server_get_internals_list_impl:
        Option<unsafe extern "C" fn(*mut jackctl_server_t) -> *const JSList>,
    jackctl_server_load_internal_impl:
        Option<unsafe extern "C" fn(*mut jackctl_server_t, *mut jackctl_internal_t) -> u8>,
    jackctl_server_unload_internal_impl:
        Option<unsafe extern "C" fn(*mut jackctl_server_t, *mut jackctl_internal_t) -> u8>,
    jackctl_server_add_slave_impl:
        Option<unsafe extern "C" fn(*mut jackctl_server_t, *mut jackctl_driver_t) -> u8>,
    jackctl_server_remove_slave_impl:
        Option<unsafe extern "C" fn(*mut jackctl_server_t, *mut jackctl_driver_t) -> u8>,
    jackctl_server_switch_master_impl:
        Option<unsafe extern "C" fn(*mut jackctl_server_t, *mut jackctl_driver_t) -> u8>,
    jackctl_driver_get_name_impl:
        Option<unsafe extern "C" fn(*mut jackctl_driver_t) -> *const ::libc::c_char>,
    jackctl_driver_get_type_impl:
        Option<unsafe extern "C" fn(*mut jackctl_driver_t) -> jackctl_driver_type_t>,
    jackctl_driver_get_parameters_impl:
        Option<unsafe extern "C" fn(*mut jackctl_driver_t) -> *const JSList>,
    jackctl_driver_params_parse_impl: Option<
        unsafe extern "C" fn(
            *mut jackctl_driver_t,
            ::libc::c_int,
            *mut *mut ::libc::c_char,
        ) -> ::libc::c_int,
    >,
    jackctl_internal_get_name_impl:
        Option<unsafe extern "C" fn(*mut jackctl_internal_t) -> *const ::libc::c_char>,
    jackctl_internal_get_parameters_impl:
        Option<unsafe extern "C" fn(*mut jackctl_internal_t) -> *const JSList>,
    jackctl_parameter_get_name_impl:
        Option<unsafe extern "C" fn(*mut jackctl_parameter_t) -> *const ::libc::c_char>,
    jackctl_parameter_get_short_description_impl:
        Option<unsafe extern "C" fn(*mut jackctl_parameter_t) -> *const ::libc::c_char>,
    jackctl_parameter_get_long_description_impl:
        Option<unsafe extern "C" fn(*mut jackctl_parameter_t) -> *const ::libc::c_char>,
    jackctl_parameter_get_type_impl:
        Option<unsafe extern "C" fn(*mut jackctl_parameter_t) -> jackctl_param_type_t>,
    jackctl_parameter_get_id_impl:
        Option<unsafe extern "C" fn(*mut jackctl_parameter_t) -> ::libc::c_char>,
    jackctl_parameter_is_set_impl: Option<unsafe extern "C" fn(*mut jackctl_parameter_t) -> u8>,
    jackctl_parameter_reset_impl: Option<unsafe extern "C" fn(*mut jackctl_parameter_t) -> u8>,
    jackctl_parameter_get_value_impl:
        Option<unsafe extern "C" fn(*mut jackctl_parameter_t) -> Union_jackctl_parameter_value>,
    jackctl_parameter_set_value_impl: Option<
        unsafe extern "C" fn(*mut jackctl_parameter_t, *const Union_jackctl_parameter_value) -> u8,
    >,
    jackctl_parameter_get_default_value_impl:
        Option<unsafe extern "C" fn(*mut jackctl_parameter_t) -> Union_jackctl_parameter_value>,
    jackctl_parameter_has_range_constraint_impl:
        Option<unsafe extern "C" fn(*mut jackctl_parameter_t) -> u8>,
    jackctl_parameter_has_enum_constraint_impl:
        Option<unsafe extern "C" fn(*mut jackctl_parameter_t) -> u8>,
    jackctl_parameter_get_enum_constraints_count_impl:
        Option<unsafe extern "C" fn(*mut jackctl_parameter_t) -> u32>,
    jackctl_parameter_get_enum_constraint_value_impl: Option<
        unsafe extern "C" fn(*mut jackctl_parameter_t, u32) -> Union_jackctl_parameter_value,
    >,
    jackctl_parameter_get_enum_constraint_description_impl:
        Option<unsafe extern "C" fn(*mut jackctl_parameter_t, u32) -> *const ::libc::c_char>,
    jackctl_parameter_get_range_constraint_impl: Option<
        unsafe extern "C" fn(
            *mut jackctl_parameter_t,
            *mut Union_jackctl_parameter_value,
            *mut Union_jackctl_parameter_value,
        ) -> (),
    >,
    jackctl_parameter_constraint_is_strict_impl:
        Option<unsafe extern "C" fn(*mut jackctl_parameter_t) -> u8>,
    jackctl_parameter_constraint_is_fake_value_impl:
        Option<unsafe extern "C" fn(*mut jackctl_parameter_t) -> u8>,
    jack_set_property_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        jack_uuid_t,
        *const ::libc::c_char,
        *const ::libc::c_char,
        *const ::libc::c_char,
    ) -> ::libc::c_int,
    jack_get_property_impl: unsafe extern "C" fn(
        jack_uuid_t,
        *const ::libc::c_char,
        *mut *mut ::libc::c_char,
        *mut *mut ::libc::c_char,
    ) -> ::libc::c_int,
    jack_free_description_impl: unsafe extern "C" fn(*mut jack_description_t, ::libc::c_int) -> (),
    jack_get_properties_impl:
        unsafe extern "C" fn(jack_uuid_t, *mut jack_description_t) -> ::libc::c_int,
    jack_get_all_properties_impl:
        unsafe extern "C" fn(*mut *mut jack_description_t) -> ::libc::c_int,
    jack_remove_property_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        jack_uuid_t,
        *const ::libc::c_char,
    ) -> ::libc::c_int,
    jack_remove_properties_impl:
        unsafe extern "C" fn(*mut jack_client_t, jack_uuid_t) -> ::libc::c_int,
    jack_remove_all_properties_impl: unsafe extern "C" fn(*mut jack_client_t) -> ::libc::c_int,
    jack_set_property_change_callback_impl: unsafe extern "C" fn(
        *mut jack_client_t,
        JackPropertyChangeCallback,
        *mut ::libc::c_void,
    ) -> ::libc::c_int,
    jack_get_internal_client_name_impl:
        Option<unsafe extern "C" fn(*mut jack_client_t, jack_intclient_t) -> *mut ::libc::c_char>,
    jack_internal_client_handle_impl: Option<
        unsafe extern "C" fn(
            *mut jack_client_t,
            *const ::libc::c_char,
            *mut jack_status_t,
        ) -> jack_intclient_t,
    >,
    jack_internal_client_load_impl: Option<
        unsafe extern "C" fn(
            *mut jack_client_t,
            *const ::libc::c_char,
            jack_options_t,
            *mut jack_status_t,
            *const ::libc::c_char,
            *const ::libc::c_char,
        ) -> jack_intclient_t,
    >,
    jack_internal_client_unload_impl:
        Option<unsafe extern "C" fn(*mut jack_client_t, jack_intclient_t) -> jack_status_t>,
    jack_get_max_delayed_usecs_impl: unsafe extern "C" fn(*mut jack_client_t) -> ::libc::c_float,
    jack_get_xrun_delayed_usecs_impl: unsafe extern "C" fn(*mut jack_client_t) -> ::libc::c_float,
    jack_reset_max_delayed_usecs_impl: unsafe extern "C" fn(*mut jack_client_t) -> (),
    jack_midi_get_event_count_impl: unsafe extern "C" fn(*mut ::libc::c_void) -> u32,
    jack_midi_event_get_impl:
        unsafe extern "C" fn(*mut jack_midi_event_t, *mut ::libc::c_void, u32) -> ::libc::c_int,
    jack_midi_clear_buffer_impl: unsafe extern "C" fn(*mut ::libc::c_void) -> (),
    jack_midi_max_event_size_impl: unsafe extern "C" fn(*mut ::libc::c_void) -> ::libc::size_t,
    jack_midi_event_reserve_impl: unsafe extern "C" fn(
        *mut ::libc::c_void,
        jack_nframes_t,
        ::libc::size_t,
    ) -> *mut jack_midi_data_t,
    jack_midi_event_write_impl: unsafe extern "C" fn(
        *mut ::libc::c_void,
        jack_nframes_t,
        *const jack_midi_data_t,
        ::libc::size_t,
    ) -> ::libc::c_int,
    jack_midi_get_lost_event_count_impl: unsafe extern "C" fn(*mut ::libc::c_void) -> u32,
    jack_ringbuffer_create_impl: unsafe extern "C" fn(::libc::size_t) -> *mut jack_ringbuffer_t,
    jack_ringbuffer_free_impl: unsafe extern "C" fn(*mut jack_ringbuffer_t) -> (),
    jack_ringbuffer_get_read_vector_impl:
        unsafe extern "C" fn(*const jack_ringbuffer_t, *mut jack_ringbuffer_data_t) -> (),
    jack_ringbuffer_get_write_vector_impl:
        unsafe extern "C" fn(*const jack_ringbuffer_t, *mut jack_ringbuffer_data_t) -> (),
    jack_ringbuffer_read_impl: unsafe extern "C" fn(
        *mut jack_ringbuffer_t,
        *mut ::libc::c_char,
        ::libc::size_t,
    ) -> ::libc::size_t,
    jack_ringbuffer_peek_impl: unsafe extern "C" fn(
        *mut jack_ringbuffer_t,
        *mut ::libc::c_char,
        ::libc::size_t,
    ) -> ::libc::size_t,
    jack_ringbuffer_read_advance_impl:
        unsafe extern "C" fn(*mut jack_ringbuffer_t, ::libc::size_t) -> (),
    jack_ringbuffer_read_space_impl:
        unsafe extern "C" fn(*const jack_ringbuffer_t) -> ::libc::size_t,
    jack_ringbuffer_mlock_impl: unsafe extern "C" fn(*mut jack_ringbuffer_t) -> ::libc::c_int,
    jack_ringbuffer_reset_impl: unsafe extern "C" fn(*mut jack_ringbuffer_t) -> (),
    jack_ringbuffer_write_impl: unsafe extern "C" fn(
        *mut jack_ringbuffer_t,
        *const ::libc::c_char,
        ::libc::size_t,
    ) -> ::libc::size_t,
    jack_ringbuffer_write_advance_impl:
        unsafe extern "C" fn(*mut jack_ringbuffer_t, ::libc::size_t) -> (),
    jack_ringbuffer_write_space_impl:
        unsafe extern "C" fn(*const jack_ringbuffer_t) -> ::libc::size_t,
    jack_uuid_to_index_impl: unsafe extern "C" fn(jack_uuid_t) -> u32,
    jack_uuid_compare_impl: unsafe extern "C" fn(jack_uuid_t, jack_uuid_t) -> ::std::os::raw::c_int,
    jack_uuid_copy_impl: unsafe extern "C" fn(*mut jack_uuid_t, jack_uuid_t) -> (),
    jack_uuid_clear_impl: unsafe extern "C" fn(*mut jack_uuid_t) -> (),
    jack_uuid_parse_impl: unsafe extern "C" fn(
        *const ::std::os::raw::c_char,
        *mut jack_uuid_t,
    ) -> ::std::os::raw::c_int,
    jack_uuid_unparse_impl: unsafe extern "C" fn(jack_uuid_t, *mut ::std::os::raw::c_char) -> (),
    jack_uuid_empty_impl: unsafe extern "C" fn(jack_uuid_t) -> ::std::os::raw::c_int,
}

lazy_static! {
    static ref FUNCTIONS: JackFunctions = unsafe {
        let library = crate::library().unwrap();
        let jack_release_timebase_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int>(
                b"jack_release_timebase",
            )
            .unwrap();
        let jack_release_timebase_impl = jack_release_timebase_impl.into_raw();
        let jack_release_timebase_impl = *jack_release_timebase_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int;
        let jack_get_cycle_times_impl = library
            .get::<unsafe extern "C" fn(
                client: *const jack_client_t,
                current_frames: *mut jack_nframes_t,
                current_usecs: *mut jack_time_t,
                next_usecs: *mut jack_time_t,
                period_usecs: *mut ::libc::c_float,
            ) -> ::libc::c_int>(b"jack_get_cycle_times")
            .ok();
        let jack_get_cycle_times_impl = jack_get_cycle_times_impl.map(|sym| sym.into_raw());
        let jack_get_cycle_times_impl = jack_get_cycle_times_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    client: *const jack_client_t,
                    current_frames: *mut jack_nframes_t,
                    current_usecs: *mut jack_time_t,
                    next_usecs: *mut jack_time_t,
                    period_usecs: *mut ::libc::c_float,
                ) -> ::libc::c_int
        });
        let jack_set_sync_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                sync_callback: JackSyncCallback,
                sync_arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_sync_callback")
            .unwrap();
        let jack_set_sync_callback_impl = jack_set_sync_callback_impl.into_raw();
        let jack_set_sync_callback_impl = *jack_set_sync_callback_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                sync_callback: JackSyncCallback,
                sync_arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_set_sync_timeout_impl =
            library
                .get::<unsafe extern "C" fn(
                    client: *mut jack_client_t,
                    timeout: jack_time_t,
                ) -> ::libc::c_int>(b"jack_set_sync_timeout")
                .unwrap();
        let jack_set_sync_timeout_impl = jack_set_sync_timeout_impl.into_raw();
        let jack_set_sync_timeout_impl = *jack_set_sync_timeout_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                timeout: jack_time_t,
            ) -> ::libc::c_int;
        let jack_set_timebase_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                conditional: ::libc::c_int,
                timebase_callback: TimebaseCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_timebase_callback")
            .unwrap();
        let jack_set_timebase_callback_impl = jack_set_timebase_callback_impl.into_raw();
        let jack_set_timebase_callback_impl = *jack_set_timebase_callback_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                conditional: ::libc::c_int,
                timebase_callback: TimebaseCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_transport_locate_impl =
            library
                .get::<unsafe extern "C" fn(
                    client: *mut jack_client_t,
                    frame: jack_nframes_t,
                ) -> ::libc::c_int>(b"jack_transport_locate")
                .unwrap();
        let jack_transport_locate_impl = jack_transport_locate_impl.into_raw();
        let jack_transport_locate_impl = *jack_transport_locate_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                frame: jack_nframes_t,
            ) -> ::libc::c_int;
        let jack_transport_query_impl = library
            .get::<unsafe extern "C" fn(
                client: *const jack_client_t,
                pos: *mut jack_position_t,
            ) -> jack_transport_state_t>(b"jack_transport_query")
            .unwrap();
        let jack_transport_query_impl = jack_transport_query_impl.into_raw();
        let jack_transport_query_impl = *jack_transport_query_impl.deref()
            as unsafe extern "C" fn(
                client: *const jack_client_t,
                pos: *mut jack_position_t,
            ) -> jack_transport_state_t;
        let jack_get_current_transport_frame_impl = library
            .get::<unsafe extern "C" fn(client: *const jack_client_t) -> jack_nframes_t>(
                b"jack_get_current_transport_frame",
            )
            .unwrap();
        let jack_get_current_transport_frame_impl =
            jack_get_current_transport_frame_impl.into_raw();
        let jack_get_current_transport_frame_impl = *jack_get_current_transport_frame_impl.deref()
            as unsafe extern "C" fn(client: *const jack_client_t) -> jack_nframes_t;
        let jack_transport_reposition_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                pos: *const jack_position_t,
            ) -> ::libc::c_int>(b"jack_transport_reposition")
            .unwrap();
        let jack_transport_reposition_impl = jack_transport_reposition_impl.into_raw();
        let jack_transport_reposition_impl = *jack_transport_reposition_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                pos: *const jack_position_t,
            ) -> ::libc::c_int;
        let jack_transport_start_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> ()>(b"jack_transport_start")
            .unwrap();
        let jack_transport_start_impl = jack_transport_start_impl.into_raw();
        let jack_transport_start_impl = *jack_transport_start_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> ();
        let jack_transport_stop_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> ()>(b"jack_transport_stop")
            .unwrap();
        let jack_transport_stop_impl = jack_transport_stop_impl.into_raw();
        let jack_transport_stop_impl = *jack_transport_stop_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> ();
        let jack_get_transport_info_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                tinfo: *mut jack_transport_info_t,
            ) -> ()>(b"jack_get_transport_info")
            .unwrap();
        let jack_get_transport_info_impl = jack_get_transport_info_impl.into_raw();
        let jack_get_transport_info_impl = *jack_get_transport_info_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                tinfo: *mut jack_transport_info_t,
            ) -> ();
        let jack_set_transport_info_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                tinfo: *mut jack_transport_info_t,
            ) -> ()>(b"jack_set_transport_info")
            .unwrap();
        let jack_set_transport_info_impl = jack_set_transport_info_impl.into_raw();
        let jack_set_transport_info_impl = *jack_set_transport_info_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                tinfo: *mut jack_transport_info_t,
            ) -> ();
        let jack_client_open_impl = library
            .get::<unsafe extern "C" fn(
                client_name: *const ::libc::c_char,
                options: jack_options_t,
                status: *mut jack_status_t,
            ) -> *mut jack_client_t>(b"jack_client_open")
            .unwrap();
        let jack_client_open_impl = jack_client_open_impl.into_raw();
        let jack_client_open_impl = *jack_client_open_impl.deref()
            as unsafe extern "C" fn(
                client_name: *const ::libc::c_char,
                options: jack_options_t,
                status: *mut jack_status_t,
            ) -> *mut jack_client_t;
        let jack_client_new_impl = library
            .get::<unsafe extern "C" fn(client_name: *const ::libc::c_char) -> *mut jack_client_t>(
                b"jack_client_new",
            )
            .unwrap();
        let jack_client_new_impl = jack_client_new_impl.into_raw();
        let jack_client_new_impl = *jack_client_new_impl.deref()
            as unsafe extern "C" fn(client_name: *const ::libc::c_char) -> *mut jack_client_t;
        let jack_client_close_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int>(
                b"jack_client_close",
            )
            .unwrap();
        let jack_client_close_impl = jack_client_close_impl.into_raw();
        let jack_client_close_impl = *jack_client_close_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int;
        let jack_client_name_size_impl = library
            .get::<unsafe extern "C" fn() -> ::libc::c_int>(b"jack_client_name_size")
            .unwrap();
        let jack_client_name_size_impl = jack_client_name_size_impl.into_raw();
        let jack_client_name_size_impl =
            *jack_client_name_size_impl.deref() as unsafe extern "C" fn() -> ::libc::c_int;
        let jack_get_client_name_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> *mut ::libc::c_char>(
                b"jack_get_client_name",
            )
            .unwrap();
        let jack_get_client_name_impl = jack_get_client_name_impl.into_raw();
        let jack_get_client_name_impl = *jack_get_client_name_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> *mut ::libc::c_char;
        let jack_get_uuid_for_client_name_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                client_name: *const ::libc::c_char,
            ) -> *mut ::libc::c_char>(b"jack_get_uuid_for_client_name")
            .unwrap();
        let jack_get_uuid_for_client_name_impl = jack_get_uuid_for_client_name_impl.into_raw();
        let jack_get_uuid_for_client_name_impl = *jack_get_uuid_for_client_name_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                client_name: *const ::libc::c_char,
            ) -> *mut ::libc::c_char;
        let jack_get_client_name_by_uuid_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                client_uuid: *const ::libc::c_char,
            ) -> *mut ::libc::c_char>(b"jack_get_client_name_by_uuid")
            .unwrap();
        let jack_get_client_name_by_uuid_impl = jack_get_client_name_by_uuid_impl.into_raw();
        let jack_get_client_name_by_uuid_impl = *jack_get_client_name_by_uuid_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                client_uuid: *const ::libc::c_char,
            ) -> *mut ::libc::c_char;
        let jack_internal_client_new_impl = library
            .get::<unsafe extern "C" fn(
                client_name: *const ::libc::c_char,
                load_name: *const ::libc::c_char,
                load_init: *const ::libc::c_char,
            ) -> ::libc::c_int>(b"jack_internal_client_new")
            .ok();
        let jack_internal_client_new_impl = jack_internal_client_new_impl.map(|sym| sym.into_raw());
        let jack_internal_client_new_impl = jack_internal_client_new_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    client_name: *const ::libc::c_char,
                    load_name: *const ::libc::c_char,
                    load_init: *const ::libc::c_char,
                ) -> ::libc::c_int
        });
        let jack_internal_client_close_impl = library
            .get::<unsafe extern "C" fn(client_name: *const ::libc::c_char) -> ()>(
                b"jack_internal_client_close",
            )
            .ok();
        let jack_internal_client_close_impl =
            jack_internal_client_close_impl.map(|sym| sym.into_raw());
        let jack_internal_client_close_impl = jack_internal_client_close_impl.map(|sym| {
            *sym.deref() as unsafe extern "C" fn(client_name: *const ::libc::c_char) -> ()
        });
        let jack_activate_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int>(
                b"jack_activate",
            )
            .unwrap();
        let jack_activate_impl = jack_activate_impl.into_raw();
        let jack_activate_impl = *jack_activate_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int;
        let jack_deactivate_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int>(
                b"jack_deactivate",
            )
            .unwrap();
        let jack_deactivate_impl = jack_deactivate_impl.into_raw();
        let jack_deactivate_impl = *jack_deactivate_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int;
        let jack_get_client_pid_impl = library
            .get::<unsafe extern "C" fn(name: *const ::libc::c_char) -> ::libc::c_int>(
                b"jack_get_client_pid",
            )
            .ok();
        let jack_get_client_pid_impl = jack_get_client_pid_impl.map(|sym| sym.into_raw());
        let jack_get_client_pid_impl = jack_get_client_pid_impl.map(|sym| {
            *sym.deref() as unsafe extern "C" fn(name: *const ::libc::c_char) -> ::libc::c_int
        });
        let jack_is_realtime_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int>(
                b"jack_is_realtime",
            )
            .unwrap();
        let jack_is_realtime_impl = jack_is_realtime_impl.into_raw();
        let jack_is_realtime_impl = *jack_is_realtime_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int;
        let jack_thread_wait_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                status: ::libc::c_int,
            ) -> jack_nframes_t>(b"jack_thread_wait")
            .unwrap();
        let jack_thread_wait_impl = jack_thread_wait_impl.into_raw();
        let jack_thread_wait_impl = *jack_thread_wait_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                status: ::libc::c_int,
            ) -> jack_nframes_t;
        let jack_cycle_wait_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> jack_nframes_t>(
                b"jack_cycle_wait",
            )
            .unwrap();
        let jack_cycle_wait_impl = jack_cycle_wait_impl.into_raw();
        let jack_cycle_wait_impl = *jack_cycle_wait_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> jack_nframes_t;
        let jack_cycle_signal_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t, status: ::libc::c_int) -> ()>(
                b"jack_cycle_signal",
            )
            .unwrap();
        let jack_cycle_signal_impl = jack_cycle_signal_impl.into_raw();
        let jack_cycle_signal_impl = *jack_cycle_signal_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t, status: ::libc::c_int) -> ();
        let jack_set_process_thread_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                thread_callback: JackThreadCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_process_thread")
            .unwrap();
        let jack_set_process_thread_impl = jack_set_process_thread_impl.into_raw();
        let jack_set_process_thread_impl = *jack_set_process_thread_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                thread_callback: JackThreadCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_set_thread_init_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                thread_init_callback: JackThreadInitCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_thread_init_callback")
            .unwrap();
        let jack_set_thread_init_callback_impl = jack_set_thread_init_callback_impl.into_raw();
        let jack_set_thread_init_callback_impl = *jack_set_thread_init_callback_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                thread_init_callback: JackThreadInitCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_on_shutdown_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                callback: JackShutdownCallback,
                arg: *mut ::libc::c_void,
            ) -> ()>(b"jack_on_shutdown")
            .unwrap();
        let jack_on_shutdown_impl = jack_on_shutdown_impl.into_raw();
        let jack_on_shutdown_impl = *jack_on_shutdown_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                callback: JackShutdownCallback,
                arg: *mut ::libc::c_void,
            ) -> ();
        let jack_on_info_shutdown_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                callback: JackInfoShutdownCallback,
                arg: *mut ::libc::c_void,
            ) -> ()>(b"jack_on_info_shutdown")
            .unwrap();
        let jack_on_info_shutdown_impl = jack_on_info_shutdown_impl.into_raw();
        let jack_on_info_shutdown_impl = *jack_on_info_shutdown_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                callback: JackInfoShutdownCallback,
                arg: *mut ::libc::c_void,
            ) -> ();
        let jack_set_process_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                process_callback: JackProcessCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_process_callback")
            .unwrap();
        let jack_set_process_callback_impl = jack_set_process_callback_impl.into_raw();
        let jack_set_process_callback_impl = *jack_set_process_callback_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                process_callback: JackProcessCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_set_freewheel_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                freewheel_callback: JackFreewheelCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_freewheel_callback")
            .unwrap();
        let jack_set_freewheel_callback_impl = jack_set_freewheel_callback_impl.into_raw();
        let jack_set_freewheel_callback_impl = *jack_set_freewheel_callback_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                freewheel_callback: JackFreewheelCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_set_buffer_size_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                bufsize_callback: JackBufferSizeCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_buffer_size_callback")
            .unwrap();
        let jack_set_buffer_size_callback_impl = jack_set_buffer_size_callback_impl.into_raw();
        let jack_set_buffer_size_callback_impl = *jack_set_buffer_size_callback_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                bufsize_callback: JackBufferSizeCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_get_sample_rate_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int>(
                b"jack_get_sample_rate",
            )
            .unwrap();
        let jack_get_sample_rate_impl = jack_get_sample_rate_impl.into_raw();
        let jack_get_sample_rate_impl = *jack_get_sample_rate_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int;
        let jack_set_sample_rate_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                srate_callback: JackSampleRateCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_sample_rate_callback")
            .unwrap();
        let jack_set_sample_rate_callback_impl = jack_set_sample_rate_callback_impl.into_raw();
        let jack_set_sample_rate_callback_impl = *jack_set_sample_rate_callback_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                srate_callback: JackSampleRateCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_set_client_registration_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                registration_callback: JackClientRegistrationCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_client_registration_callback")
            .unwrap();
        let jack_set_client_registration_callback_impl =
            jack_set_client_registration_callback_impl.into_raw();
        let jack_set_client_registration_callback_impl = *jack_set_client_registration_callback_impl
            .deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                registration_callback: JackClientRegistrationCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_set_port_registration_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                registration_callback: JackPortRegistrationCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_port_registration_callback")
            .unwrap();
        let jack_set_port_registration_callback_impl =
            jack_set_port_registration_callback_impl.into_raw();
        let jack_set_port_registration_callback_impl = *jack_set_port_registration_callback_impl
            .deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                registration_callback: JackPortRegistrationCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_set_port_connect_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                connect_callback: JackPortConnectCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_port_connect_callback")
            .unwrap();
        let jack_set_port_connect_callback_impl = jack_set_port_connect_callback_impl.into_raw();
        let jack_set_port_connect_callback_impl = *jack_set_port_connect_callback_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                connect_callback: JackPortConnectCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_set_port_rename_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                rename_callback: JackPortRenameCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_port_rename_callback")
            .unwrap();
        let jack_set_port_rename_callback_impl = jack_set_port_rename_callback_impl.into_raw();
        let jack_set_port_rename_callback_impl = *jack_set_port_rename_callback_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                rename_callback: JackPortRenameCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_set_graph_order_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                graph_callback: JackGraphOrderCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_graph_order_callback")
            .unwrap();
        let jack_set_graph_order_callback_impl = jack_set_graph_order_callback_impl.into_raw();
        let jack_set_graph_order_callback_impl = *jack_set_graph_order_callback_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                graph_callback: JackGraphOrderCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_set_xrun_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                xrun_callback: JackXRunCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_xrun_callback")
            .unwrap();
        let jack_set_xrun_callback_impl = jack_set_xrun_callback_impl.into_raw();
        let jack_set_xrun_callback_impl = *jack_set_xrun_callback_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                xrun_callback: JackXRunCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_set_latency_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                latency_callback: JackLatencyCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_latency_callback")
            .unwrap();
        let jack_set_latency_callback_impl = jack_set_latency_callback_impl.into_raw();
        let jack_set_latency_callback_impl = *jack_set_latency_callback_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                latency_callback: JackLatencyCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_set_freewheel_impl =
            library
                .get::<unsafe extern "C" fn(
                    client: *mut jack_client_t,
                    onoff: ::libc::c_int,
                ) -> ::libc::c_int>(b"jack_set_freewheel")
                .unwrap();
        let jack_set_freewheel_impl = jack_set_freewheel_impl.into_raw();
        let jack_set_freewheel_impl = *jack_set_freewheel_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                onoff: ::libc::c_int,
            ) -> ::libc::c_int;
        let jack_set_buffer_size_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                nframes: jack_nframes_t,
            ) -> ::libc::c_int>(b"jack_set_buffer_size")
            .unwrap();
        let jack_set_buffer_size_impl = jack_set_buffer_size_impl.into_raw();
        let jack_set_buffer_size_impl = *jack_set_buffer_size_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                nframes: jack_nframes_t,
            ) -> ::libc::c_int;
        let jack_get_buffer_size_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> jack_nframes_t>(
                b"jack_get_buffer_size",
            )
            .unwrap();
        let jack_get_buffer_size_impl = jack_get_buffer_size_impl.into_raw();
        let jack_get_buffer_size_impl = *jack_get_buffer_size_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> jack_nframes_t;
        let jack_engine_takeover_timebase_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int>(
                b"jack_engine_takeover_timebase",
            )
            .unwrap();
        let jack_engine_takeover_timebase_impl = jack_engine_takeover_timebase_impl.into_raw();
        let jack_engine_takeover_timebase_impl = *jack_engine_takeover_timebase_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int;
        let jack_cpu_load_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_float>(
                b"jack_cpu_load",
            )
            .unwrap();
        let jack_cpu_load_impl = jack_cpu_load_impl.into_raw();
        let jack_cpu_load_impl = *jack_cpu_load_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_float;
        let jack_port_register_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                port_name: *const ::libc::c_char,
                port_type: *const ::libc::c_char,
                flags: ::libc::c_ulong,
                buffer_size: ::libc::c_ulong,
            ) -> *mut jack_port_t>(b"jack_port_register")
            .unwrap();
        let jack_port_register_impl = jack_port_register_impl.into_raw();
        let jack_port_register_impl = *jack_port_register_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                port_name: *const ::libc::c_char,
                port_type: *const ::libc::c_char,
                flags: ::libc::c_ulong,
                buffer_size: ::libc::c_ulong,
            ) -> *mut jack_port_t;
        let jack_port_unregister_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                port: *mut jack_port_t,
            ) -> ::libc::c_int>(b"jack_port_unregister")
            .unwrap();
        let jack_port_unregister_impl = jack_port_unregister_impl.into_raw();
        let jack_port_unregister_impl = *jack_port_unregister_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                port: *mut jack_port_t,
            ) -> ::libc::c_int;
        let jack_port_get_buffer_impl = library
            .get::<unsafe extern "C" fn(
                port: *mut jack_port_t,
                nframes: jack_nframes_t,
            ) -> *mut ::libc::c_void>(b"jack_port_get_buffer")
            .unwrap();
        let jack_port_get_buffer_impl = jack_port_get_buffer_impl.into_raw();
        let jack_port_get_buffer_impl = *jack_port_get_buffer_impl.deref()
            as unsafe extern "C" fn(
                port: *mut jack_port_t,
                nframes: jack_nframes_t,
            ) -> *mut ::libc::c_void;
        let jack_port_uuid_impl = library
            .get::<unsafe extern "C" fn(port: *mut jack_port_t) -> jack_uuid_t>(b"jack_port_uuid")
            .unwrap();
        let jack_port_uuid_impl = jack_port_uuid_impl.into_raw();
        let jack_port_uuid_impl = *jack_port_uuid_impl.deref()
            as unsafe extern "C" fn(port: *mut jack_port_t) -> jack_uuid_t;
        let jack_port_name_impl = library
            .get::<unsafe extern "C" fn(port: *mut jack_port_t) -> *const ::libc::c_char>(
                b"jack_port_name",
            )
            .unwrap();
        let jack_port_name_impl = jack_port_name_impl.into_raw();
        let jack_port_name_impl = *jack_port_name_impl.deref()
            as unsafe extern "C" fn(port: *mut jack_port_t) -> *const ::libc::c_char;
        let jack_port_short_name_impl = library
            .get::<unsafe extern "C" fn(port: *mut jack_port_t) -> *const ::libc::c_char>(
                b"jack_port_short_name",
            )
            .unwrap();
        let jack_port_short_name_impl = jack_port_short_name_impl.into_raw();
        let jack_port_short_name_impl = *jack_port_short_name_impl.deref()
            as unsafe extern "C" fn(port: *mut jack_port_t) -> *const ::libc::c_char;
        let jack_port_flags_impl = library
            .get::<unsafe extern "C" fn(port: *mut jack_port_t) -> ::libc::c_int>(
                b"jack_port_flags",
            )
            .unwrap();
        let jack_port_flags_impl = jack_port_flags_impl.into_raw();
        let jack_port_flags_impl = *jack_port_flags_impl.deref()
            as unsafe extern "C" fn(port: *mut jack_port_t) -> ::libc::c_int;
        let jack_port_type_impl = library
            .get::<unsafe extern "C" fn(port: *const jack_port_t) -> *const ::libc::c_char>(
                b"jack_port_type",
            )
            .unwrap();
        let jack_port_type_impl = jack_port_type_impl.into_raw();
        let jack_port_type_impl = *jack_port_type_impl.deref()
            as unsafe extern "C" fn(port: *const jack_port_t) -> *const ::libc::c_char;
        let jack_port_type_id_impl = library
            .get::<unsafe extern "C" fn(port: *const jack_port_t) -> jack_port_type_id_t>(
                b"jack_port_type_id",
            )
            .ok();
        let jack_port_type_id_impl = jack_port_type_id_impl.map(|sym| sym.into_raw());
        let jack_port_type_id_impl = jack_port_type_id_impl.map(|sym| {
            *sym.deref() as unsafe extern "C" fn(port: *const jack_port_t) -> jack_port_type_id_t
        });
        let jack_port_is_mine_impl = library
            .get::<unsafe extern "C" fn(
                client: *const jack_client_t,
                port: *const jack_port_t,
            ) -> ::libc::c_int>(b"jack_port_is_mine")
            .unwrap();
        let jack_port_is_mine_impl = jack_port_is_mine_impl.into_raw();
        let jack_port_is_mine_impl = *jack_port_is_mine_impl.deref()
            as unsafe extern "C" fn(
                client: *const jack_client_t,
                port: *const jack_port_t,
            ) -> ::libc::c_int;
        let jack_port_connected_impl = library
            .get::<unsafe extern "C" fn(port: *const jack_port_t) -> ::libc::c_int>(
                b"jack_port_connected",
            )
            .unwrap();
        let jack_port_connected_impl = jack_port_connected_impl.into_raw();
        let jack_port_connected_impl = *jack_port_connected_impl.deref()
            as unsafe extern "C" fn(port: *const jack_port_t) -> ::libc::c_int;
        let jack_port_connected_to_impl = library
            .get::<unsafe extern "C" fn(
                port: *const jack_port_t,
                port_name: *const ::libc::c_char,
            ) -> ::libc::c_int>(b"jack_port_connected_to")
            .unwrap();
        let jack_port_connected_to_impl = jack_port_connected_to_impl.into_raw();
        let jack_port_connected_to_impl = *jack_port_connected_to_impl.deref()
            as unsafe extern "C" fn(
                port: *const jack_port_t,
                port_name: *const ::libc::c_char,
            ) -> ::libc::c_int;
        let jack_port_get_connections_impl = library
            .get::<unsafe extern "C" fn(port: *const jack_port_t) -> *mut *const ::libc::c_char>(
                b"jack_port_get_connections",
            )
            .unwrap();
        let jack_port_get_connections_impl = jack_port_get_connections_impl.into_raw();
        let jack_port_get_connections_impl = *jack_port_get_connections_impl.deref()
            as unsafe extern "C" fn(port: *const jack_port_t) -> *mut *const ::libc::c_char;
        let jack_port_get_all_connections_impl = library
            .get::<unsafe extern "C" fn(
                client: *const jack_client_t,
                port: *const jack_port_t,
            ) -> *mut *const ::libc::c_char>(b"jack_port_get_all_connections")
            .unwrap();
        let jack_port_get_all_connections_impl = jack_port_get_all_connections_impl.into_raw();
        let jack_port_get_all_connections_impl = *jack_port_get_all_connections_impl.deref()
            as unsafe extern "C" fn(
                client: *const jack_client_t,
                port: *const jack_port_t,
            ) -> *mut *const ::libc::c_char;
        let jack_port_tie_impl = library.get::<unsafe extern "C" fn(src: *mut jack_port_t, dst: *mut jack_port_t) -> ::libc::c_int>(b"jack_port_tie").unwrap();
        let jack_port_tie_impl = jack_port_tie_impl.into_raw();
        let jack_port_tie_impl = *jack_port_tie_impl.deref()
            as unsafe extern "C" fn(src: *mut jack_port_t, dst: *mut jack_port_t) -> ::libc::c_int;
        let jack_port_untie_impl = library
            .get::<unsafe extern "C" fn(port: *mut jack_port_t) -> ::libc::c_int>(
                b"jack_port_untie",
            )
            .unwrap();
        let jack_port_untie_impl = jack_port_untie_impl.into_raw();
        let jack_port_untie_impl = *jack_port_untie_impl.deref()
            as unsafe extern "C" fn(port: *mut jack_port_t) -> ::libc::c_int;
        let jack_port_set_name_impl = library
            .get::<unsafe extern "C" fn(
                port: *mut jack_port_t,
                port_name: *const ::libc::c_char,
            ) -> ::libc::c_int>(b"jack_port_set_name")
            .unwrap();
        let jack_port_set_name_impl = jack_port_set_name_impl.into_raw();
        let jack_port_set_name_impl = *jack_port_set_name_impl.deref()
            as unsafe extern "C" fn(
                port: *mut jack_port_t,
                port_name: *const ::libc::c_char,
            ) -> ::libc::c_int;
        let jack_port_set_alias_impl = library
            .get::<unsafe extern "C" fn(
                port: *mut jack_port_t,
                alias: *const ::libc::c_char,
            ) -> ::libc::c_int>(b"jack_port_set_alias")
            .unwrap();
        let jack_port_set_alias_impl = jack_port_set_alias_impl.into_raw();
        let jack_port_set_alias_impl = *jack_port_set_alias_impl.deref()
            as unsafe extern "C" fn(
                port: *mut jack_port_t,
                alias: *const ::libc::c_char,
            ) -> ::libc::c_int;
        let jack_port_unset_alias_impl = library
            .get::<unsafe extern "C" fn(
                port: *mut jack_port_t,
                alias: *const ::libc::c_char,
            ) -> ::libc::c_int>(b"jack_port_unset_alias")
            .unwrap();
        let jack_port_unset_alias_impl = jack_port_unset_alias_impl.into_raw();
        let jack_port_unset_alias_impl = *jack_port_unset_alias_impl.deref()
            as unsafe extern "C" fn(
                port: *mut jack_port_t,
                alias: *const ::libc::c_char,
            ) -> ::libc::c_int;
        let jack_port_get_aliases_impl = library
            .get::<unsafe extern "C" fn(
                port: *const jack_port_t,
                aliases: *mut *mut ::libc::c_char,
            ) -> ::libc::c_int>(b"jack_port_get_aliases")
            .unwrap();
        let jack_port_get_aliases_impl = jack_port_get_aliases_impl.into_raw();
        let jack_port_get_aliases_impl = *jack_port_get_aliases_impl.deref()
            as unsafe extern "C" fn(
                port: *const jack_port_t,
                aliases: *mut *mut ::libc::c_char,
            ) -> ::libc::c_int;
        let jack_port_request_monitor_impl = library.get::<unsafe extern "C" fn(port: *mut jack_port_t, onoff: ::libc::c_int) -> ::libc::c_int>(b"jack_port_request_monitor").unwrap();
        let jack_port_request_monitor_impl = jack_port_request_monitor_impl.into_raw();
        let jack_port_request_monitor_impl = *jack_port_request_monitor_impl.deref()
            as unsafe extern "C" fn(port: *mut jack_port_t, onoff: ::libc::c_int) -> ::libc::c_int;
        let jack_port_request_monitor_by_name_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                port_name: *const ::libc::c_char,
                onoff: ::libc::c_int,
            ) -> ::libc::c_int>(b"jack_port_request_monitor_by_name")
            .unwrap();
        let jack_port_request_monitor_by_name_impl =
            jack_port_request_monitor_by_name_impl.into_raw();
        let jack_port_request_monitor_by_name_impl = *jack_port_request_monitor_by_name_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                port_name: *const ::libc::c_char,
                onoff: ::libc::c_int,
            ) -> ::libc::c_int;
        let jack_port_ensure_monitor_impl = library.get::<unsafe extern "C" fn(port: *mut jack_port_t, onoff: ::libc::c_int) -> ::libc::c_int>(b"jack_port_ensure_monitor").unwrap();
        let jack_port_ensure_monitor_impl = jack_port_ensure_monitor_impl.into_raw();
        let jack_port_ensure_monitor_impl = *jack_port_ensure_monitor_impl.deref()
            as unsafe extern "C" fn(port: *mut jack_port_t, onoff: ::libc::c_int) -> ::libc::c_int;
        let jack_port_monitoring_input_impl = library
            .get::<unsafe extern "C" fn(port: *mut jack_port_t) -> ::libc::c_int>(
                b"jack_port_monitoring_input",
            )
            .unwrap();
        let jack_port_monitoring_input_impl = jack_port_monitoring_input_impl.into_raw();
        let jack_port_monitoring_input_impl = *jack_port_monitoring_input_impl.deref()
            as unsafe extern "C" fn(port: *mut jack_port_t) -> ::libc::c_int;
        let jack_connect_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                source_port: *const ::libc::c_char,
                destination_port: *const ::libc::c_char,
            ) -> ::libc::c_int>(b"jack_connect")
            .unwrap();
        let jack_connect_impl = jack_connect_impl.into_raw();
        let jack_connect_impl = *jack_connect_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                source_port: *const ::libc::c_char,
                destination_port: *const ::libc::c_char,
            ) -> ::libc::c_int;
        let jack_disconnect_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                source_port: *const ::libc::c_char,
                destination_port: *const ::libc::c_char,
            ) -> ::libc::c_int>(b"jack_disconnect")
            .unwrap();
        let jack_disconnect_impl = jack_disconnect_impl.into_raw();
        let jack_disconnect_impl = *jack_disconnect_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                source_port: *const ::libc::c_char,
                destination_port: *const ::libc::c_char,
            ) -> ::libc::c_int;
        let jack_port_disconnect_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                port: *mut jack_port_t,
            ) -> ::libc::c_int>(b"jack_port_disconnect")
            .unwrap();
        let jack_port_disconnect_impl = jack_port_disconnect_impl.into_raw();
        let jack_port_disconnect_impl = *jack_port_disconnect_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                port: *mut jack_port_t,
            ) -> ::libc::c_int;
        let jack_port_name_size_impl = library
            .get::<unsafe extern "C" fn() -> ::libc::c_int>(b"jack_port_name_size")
            .unwrap();
        let jack_port_name_size_impl = jack_port_name_size_impl.into_raw();
        let jack_port_name_size_impl =
            *jack_port_name_size_impl.deref() as unsafe extern "C" fn() -> ::libc::c_int;
        let jack_port_type_size_impl = library
            .get::<unsafe extern "C" fn() -> ::libc::c_int>(b"jack_port_type_size")
            .unwrap();
        let jack_port_type_size_impl = jack_port_type_size_impl.into_raw();
        let jack_port_type_size_impl =
            *jack_port_type_size_impl.deref() as unsafe extern "C" fn() -> ::libc::c_int;
        let jack_port_type_get_buffer_size_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                port_type: *const ::libc::c_char,
            ) -> ::libc::size_t>(b"jack_port_type_get_buffer_size")
            .unwrap();
        let jack_port_type_get_buffer_size_impl = jack_port_type_get_buffer_size_impl.into_raw();
        let jack_port_type_get_buffer_size_impl = *jack_port_type_get_buffer_size_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                port_type: *const ::libc::c_char,
            ) -> ::libc::size_t;
        let jack_port_set_latency_impl = library
            .get::<unsafe extern "C" fn(port: *mut jack_port_t, arg1: jack_nframes_t) -> ()>(
                b"jack_port_set_latency",
            )
            .unwrap();
        let jack_port_set_latency_impl = jack_port_set_latency_impl.into_raw();
        let jack_port_set_latency_impl = *jack_port_set_latency_impl.deref()
            as unsafe extern "C" fn(port: *mut jack_port_t, arg1: jack_nframes_t) -> ();
        let jack_port_get_latency_range_impl = library
            .get::<unsafe extern "C" fn(
                port: *mut jack_port_t,
                mode: jack_latency_callback_mode_t,
                range: *mut jack_latency_range_t,
            ) -> ()>(b"jack_port_get_latency_range")
            .unwrap();
        let jack_port_get_latency_range_impl = jack_port_get_latency_range_impl.into_raw();
        let jack_port_get_latency_range_impl = *jack_port_get_latency_range_impl.deref()
            as unsafe extern "C" fn(
                port: *mut jack_port_t,
                mode: jack_latency_callback_mode_t,
                range: *mut jack_latency_range_t,
            ) -> ();
        let jack_port_set_latency_range_impl = library
            .get::<unsafe extern "C" fn(
                port: *mut jack_port_t,
                mode: jack_latency_callback_mode_t,
                range: *mut jack_latency_range_t,
            ) -> ()>(b"jack_port_set_latency_range")
            .unwrap();
        let jack_port_set_latency_range_impl = jack_port_set_latency_range_impl.into_raw();
        let jack_port_set_latency_range_impl = *jack_port_set_latency_range_impl.deref()
            as unsafe extern "C" fn(
                port: *mut jack_port_t,
                mode: jack_latency_callback_mode_t,
                range: *mut jack_latency_range_t,
            ) -> ();
        let jack_recompute_total_latencies_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int>(
                b"jack_recompute_total_latencies",
            )
            .unwrap();
        let jack_recompute_total_latencies_impl = jack_recompute_total_latencies_impl.into_raw();
        let jack_recompute_total_latencies_impl = *jack_recompute_total_latencies_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int;
        let jack_port_get_latency_impl = library
            .get::<unsafe extern "C" fn(port: *mut jack_port_t) -> jack_nframes_t>(
                b"jack_port_get_latency",
            )
            .unwrap();
        let jack_port_get_latency_impl = jack_port_get_latency_impl.into_raw();
        let jack_port_get_latency_impl = *jack_port_get_latency_impl.deref()
            as unsafe extern "C" fn(port: *mut jack_port_t) -> jack_nframes_t;
        let jack_port_get_total_latency_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                port: *mut jack_port_t,
            ) -> jack_nframes_t>(b"jack_port_get_total_latency")
            .unwrap();
        let jack_port_get_total_latency_impl = jack_port_get_total_latency_impl.into_raw();
        let jack_port_get_total_latency_impl = *jack_port_get_total_latency_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                port: *mut jack_port_t,
            ) -> jack_nframes_t;
        let jack_recompute_total_latency_impl =
            library
                .get::<unsafe extern "C" fn(
                    arg1: *mut jack_client_t,
                    port: *mut jack_port_t,
                ) -> ::libc::c_int>(b"jack_recompute_total_latency")
                .unwrap();
        let jack_recompute_total_latency_impl = jack_recompute_total_latency_impl.into_raw();
        let jack_recompute_total_latency_impl = *jack_recompute_total_latency_impl.deref()
            as unsafe extern "C" fn(
                arg1: *mut jack_client_t,
                port: *mut jack_port_t,
            ) -> ::libc::c_int;
        let jack_get_ports_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                port_name_pattern: *const ::libc::c_char,
                type_name_pattern: *const ::libc::c_char,
                flags: ::libc::c_ulong,
            ) -> *mut *const ::libc::c_char>(b"jack_get_ports")
            .unwrap();
        let jack_get_ports_impl = jack_get_ports_impl.into_raw();
        let jack_get_ports_impl = *jack_get_ports_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                port_name_pattern: *const ::libc::c_char,
                type_name_pattern: *const ::libc::c_char,
                flags: ::libc::c_ulong,
            ) -> *mut *const ::libc::c_char;
        let jack_port_by_name_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                port_name: *const ::libc::c_char,
            ) -> *mut jack_port_t>(b"jack_port_by_name")
            .unwrap();
        let jack_port_by_name_impl = jack_port_by_name_impl.into_raw();
        let jack_port_by_name_impl = *jack_port_by_name_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                port_name: *const ::libc::c_char,
            ) -> *mut jack_port_t;
        let jack_port_by_id_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                port_id: jack_port_id_t,
            ) -> *mut jack_port_t>(b"jack_port_by_id")
            .unwrap();
        let jack_port_by_id_impl = jack_port_by_id_impl.into_raw();
        let jack_port_by_id_impl = *jack_port_by_id_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                port_id: jack_port_id_t,
            ) -> *mut jack_port_t;
        let jack_frames_since_cycle_start_impl = library
            .get::<unsafe extern "C" fn(arg1: *const jack_client_t) -> jack_nframes_t>(
                b"jack_frames_since_cycle_start",
            )
            .unwrap();
        let jack_frames_since_cycle_start_impl = jack_frames_since_cycle_start_impl.into_raw();
        let jack_frames_since_cycle_start_impl = *jack_frames_since_cycle_start_impl.deref()
            as unsafe extern "C" fn(arg1: *const jack_client_t) -> jack_nframes_t;
        let jack_frame_time_impl = library
            .get::<unsafe extern "C" fn(arg1: *const jack_client_t) -> jack_nframes_t>(
                b"jack_frame_time",
            )
            .unwrap();
        let jack_frame_time_impl = jack_frame_time_impl.into_raw();
        let jack_frame_time_impl = *jack_frame_time_impl.deref()
            as unsafe extern "C" fn(arg1: *const jack_client_t) -> jack_nframes_t;
        let jack_last_frame_time_impl = library
            .get::<unsafe extern "C" fn(client: *const jack_client_t) -> jack_nframes_t>(
                b"jack_last_frame_time",
            )
            .unwrap();
        let jack_last_frame_time_impl = jack_last_frame_time_impl.into_raw();
        let jack_last_frame_time_impl = *jack_last_frame_time_impl.deref()
            as unsafe extern "C" fn(client: *const jack_client_t) -> jack_nframes_t;
        let jack_frames_to_time_impl =
            library
                .get::<unsafe extern "C" fn(
                    client: *const jack_client_t,
                    arg1: jack_nframes_t,
                ) -> jack_time_t>(b"jack_frames_to_time")
                .unwrap();
        let jack_frames_to_time_impl = jack_frames_to_time_impl.into_raw();
        let jack_frames_to_time_impl = *jack_frames_to_time_impl.deref()
            as unsafe extern "C" fn(
                client: *const jack_client_t,
                arg1: jack_nframes_t,
            ) -> jack_time_t;
        let jack_time_to_frames_impl =
            library
                .get::<unsafe extern "C" fn(
                    client: *const jack_client_t,
                    arg1: jack_time_t,
                ) -> jack_nframes_t>(b"jack_time_to_frames")
                .unwrap();
        let jack_time_to_frames_impl = jack_time_to_frames_impl.into_raw();
        let jack_time_to_frames_impl = *jack_time_to_frames_impl.deref()
            as unsafe extern "C" fn(
                client: *const jack_client_t,
                arg1: jack_time_t,
            ) -> jack_nframes_t;
        let jack_get_time_impl = library
            .get::<unsafe extern "C" fn() -> jack_time_t>(b"jack_get_time")
            .unwrap();
        let jack_get_time_impl = jack_get_time_impl.into_raw();
        let jack_get_time_impl =
            *jack_get_time_impl.deref() as unsafe extern "C" fn() -> jack_time_t;
        let jack_set_error_function_impl =
            library
                .get::<unsafe extern "C" fn(
                    func: ::std::option::Option<
                        unsafe extern "C" fn(arg1: *const ::libc::c_char) -> (),
                    >,
                ) -> ()>(b"jack_set_error_function")
                .unwrap();
        let jack_set_error_function_impl = jack_set_error_function_impl.into_raw();
        let jack_set_error_function_impl = *jack_set_error_function_impl.deref()
            as unsafe extern "C" fn(
                func: ::std::option::Option<unsafe extern "C" fn(arg1: *const ::libc::c_char) -> ()>,
            ) -> ();
        let jack_set_info_function_impl =
            library
                .get::<unsafe extern "C" fn(
                    func: ::std::option::Option<
                        unsafe extern "C" fn(arg1: *const ::libc::c_char) -> (),
                    >,
                ) -> ()>(b"jack_set_info_function")
                .unwrap();
        let jack_set_info_function_impl = jack_set_info_function_impl.into_raw();
        let jack_set_info_function_impl = *jack_set_info_function_impl.deref()
            as unsafe extern "C" fn(
                func: ::std::option::Option<unsafe extern "C" fn(arg1: *const ::libc::c_char) -> ()>,
            ) -> ();
        let jack_free_impl = library
            .get::<unsafe extern "C" fn(ptr: *mut ::libc::c_void) -> ()>(b"jack_free")
            .unwrap();
        let jack_free_impl = jack_free_impl.into_raw();
        let jack_free_impl =
            *jack_free_impl.deref() as unsafe extern "C" fn(ptr: *mut ::libc::c_void) -> ();
        let jack_client_real_time_priority_impl = library
            .get::<unsafe extern "C" fn(arg1: *mut jack_client_t) -> ::libc::c_int>(
                b"jack_client_real_time_priority",
            )
            .unwrap();
        let jack_client_real_time_priority_impl = jack_client_real_time_priority_impl.into_raw();
        let jack_client_real_time_priority_impl = *jack_client_real_time_priority_impl.deref()
            as unsafe extern "C" fn(arg1: *mut jack_client_t) -> ::libc::c_int;
        let jack_client_max_real_time_priority_impl = library
            .get::<unsafe extern "C" fn(arg1: *mut jack_client_t) -> ::libc::c_int>(
                b"jack_client_max_real_time_priority",
            )
            .unwrap();
        let jack_client_max_real_time_priority_impl =
            jack_client_max_real_time_priority_impl.into_raw();
        let jack_client_max_real_time_priority_impl = *jack_client_max_real_time_priority_impl
            .deref()
            as unsafe extern "C" fn(arg1: *mut jack_client_t) -> ::libc::c_int;
        let jack_set_session_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                session_callback: JackSessionCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_session_callback")
            .unwrap();
        let jack_set_session_callback_impl = jack_set_session_callback_impl.into_raw();
        let jack_set_session_callback_impl = *jack_set_session_callback_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                session_callback: JackSessionCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_session_reply_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                event: *mut jack_session_event_t,
            ) -> ::libc::c_int>(b"jack_session_reply")
            .unwrap();
        let jack_session_reply_impl = jack_session_reply_impl.into_raw();
        let jack_session_reply_impl = *jack_session_reply_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                event: *mut jack_session_event_t,
            ) -> ::libc::c_int;
        let jack_session_event_free_impl = library
            .get::<unsafe extern "C" fn(event: *mut jack_session_event_t) -> ()>(
                b"jack_session_event_free",
            )
            .unwrap();
        let jack_session_event_free_impl = jack_session_event_free_impl.into_raw();
        let jack_session_event_free_impl = *jack_session_event_free_impl.deref()
            as unsafe extern "C" fn(event: *mut jack_session_event_t) -> ();
        let jack_client_get_uuid_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> *mut ::libc::c_char>(
                b"jack_client_get_uuid",
            )
            .unwrap();
        let jack_client_get_uuid_impl = jack_client_get_uuid_impl.into_raw();
        let jack_client_get_uuid_impl = *jack_client_get_uuid_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> *mut ::libc::c_char;
        let jack_session_notify_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                target: *const ::libc::c_char,
                _type: jack_session_event_type_t,
                path: *const ::libc::c_char,
            ) -> *mut jack_session_command_t>(b"jack_session_notify")
            .unwrap();
        let jack_session_notify_impl = jack_session_notify_impl.into_raw();
        let jack_session_notify_impl = *jack_session_notify_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                target: *const ::libc::c_char,
                _type: jack_session_event_type_t,
                path: *const ::libc::c_char,
            ) -> *mut jack_session_command_t;
        let jack_session_commands_free_impl = library
            .get::<unsafe extern "C" fn(cmds: *mut jack_session_command_t) -> ()>(
                b"jack_session_commands_free",
            )
            .unwrap();
        let jack_session_commands_free_impl = jack_session_commands_free_impl.into_raw();
        let jack_session_commands_free_impl = *jack_session_commands_free_impl.deref()
            as unsafe extern "C" fn(cmds: *mut jack_session_command_t) -> ();
        let jack_reserve_client_name_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                name: *const ::libc::c_char,
                uuid: *const ::libc::c_char,
            ) -> ::libc::c_int>(b"jack_reserve_client_name")
            .unwrap();
        let jack_reserve_client_name_impl = jack_reserve_client_name_impl.into_raw();
        let jack_reserve_client_name_impl = *jack_reserve_client_name_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                name: *const ::libc::c_char,
                uuid: *const ::libc::c_char,
            ) -> ::libc::c_int;
        let jack_client_has_session_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                client_name: *const ::libc::c_char,
            ) -> ::libc::c_int>(b"jack_client_has_session_callback")
            .unwrap();
        let jack_client_has_session_callback_impl =
            jack_client_has_session_callback_impl.into_raw();
        let jack_client_has_session_callback_impl = *jack_client_has_session_callback_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                client_name: *const ::libc::c_char,
            ) -> ::libc::c_int;
        let jackctl_setup_signals_impl = library
            .get::<unsafe extern "C" fn(flags: ::libc::c_uint) -> *mut jackctl_sigmask_t>(
                b"jackctl_setup_signals",
            )
            .ok();
        let jackctl_setup_signals_impl = jackctl_setup_signals_impl.map(|sym| sym.into_raw());
        let jackctl_setup_signals_impl = jackctl_setup_signals_impl.map(|sym| {
            *sym.deref() as unsafe extern "C" fn(flags: ::libc::c_uint) -> *mut jackctl_sigmask_t
        });
        let jackctl_wait_signals_impl = library
            .get::<unsafe extern "C" fn(signals: *mut jackctl_sigmask_t) -> ()>(
                b"jackctl_wait_signals",
            )
            .ok();
        let jackctl_wait_signals_impl = jackctl_wait_signals_impl.map(|sym| sym.into_raw());
        let jackctl_wait_signals_impl = jackctl_wait_signals_impl
            .map(|sym| *sym.deref() as unsafe extern "C" fn(signals: *mut jackctl_sigmask_t) -> ());
        let jackctl_server_create_impl = library
            .get::<unsafe extern "C" fn(
                on_device_acquire: ::std::option::Option<
                    unsafe extern "C" fn(device_name: *const ::libc::c_char) -> u8,
                >,
                on_device_release: ::std::option::Option<
                    unsafe extern "C" fn(device_name: *const ::libc::c_char) -> (),
                >,
            ) -> *mut jackctl_server_t>(b"jackctl_server_create")
            .ok();
        let jackctl_server_create_impl = jackctl_server_create_impl.map(|sym| sym.into_raw());
        let jackctl_server_create_impl = jackctl_server_create_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    on_device_acquire: ::std::option::Option<
                        unsafe extern "C" fn(device_name: *const ::libc::c_char) -> u8,
                    >,
                    on_device_release: ::std::option::Option<
                        unsafe extern "C" fn(device_name: *const ::libc::c_char) -> (),
                    >,
                ) -> *mut jackctl_server_t
        });
        let jackctl_server_destroy_impl = library
            .get::<unsafe extern "C" fn(server: *mut jackctl_server_t) -> ()>(
                b"jackctl_server_destroy",
            )
            .ok();
        let jackctl_server_destroy_impl = jackctl_server_destroy_impl.map(|sym| sym.into_raw());
        let jackctl_server_destroy_impl = jackctl_server_destroy_impl
            .map(|sym| *sym.deref() as unsafe extern "C" fn(server: *mut jackctl_server_t) -> ());
        let jackctl_server_open_impl =
            library
                .get::<unsafe extern "C" fn(
                    server: *mut jackctl_server_t,
                    driver: *mut jackctl_driver_t,
                ) -> u8>(b"jackctl_server_open")
                .ok();
        let jackctl_server_open_impl = jackctl_server_open_impl.map(|sym| sym.into_raw());
        let jackctl_server_open_impl = jackctl_server_open_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    server: *mut jackctl_server_t,
                    driver: *mut jackctl_driver_t,
                ) -> u8
        });
        let jackctl_server_start_impl = library
            .get::<unsafe extern "C" fn(server: *mut jackctl_server_t) -> u8>(
                b"jackctl_server_start",
            )
            .ok();
        let jackctl_server_start_impl = jackctl_server_start_impl.map(|sym| sym.into_raw());
        let jackctl_server_start_impl = jackctl_server_start_impl
            .map(|sym| *sym.deref() as unsafe extern "C" fn(server: *mut jackctl_server_t) -> u8);
        let jackctl_server_stop_impl = library
            .get::<unsafe extern "C" fn(server: *mut jackctl_server_t) -> u8>(
                b"jackctl_server_stop",
            )
            .ok();
        let jackctl_server_stop_impl = jackctl_server_stop_impl.map(|sym| sym.into_raw());
        let jackctl_server_stop_impl = jackctl_server_stop_impl
            .map(|sym| *sym.deref() as unsafe extern "C" fn(server: *mut jackctl_server_t) -> u8);
        let jackctl_server_close_impl = library
            .get::<unsafe extern "C" fn(server: *mut jackctl_server_t) -> u8>(
                b"jackctl_server_close",
            )
            .ok();
        let jackctl_server_close_impl = jackctl_server_close_impl.map(|sym| sym.into_raw());
        let jackctl_server_close_impl = jackctl_server_close_impl
            .map(|sym| *sym.deref() as unsafe extern "C" fn(server: *mut jackctl_server_t) -> u8);
        let jackctl_server_get_drivers_list_impl = library
            .get::<unsafe extern "C" fn(server: *mut jackctl_server_t) -> *const JSList>(
                b"jackctl_server_get_drivers_list",
            )
            .ok();
        let jackctl_server_get_drivers_list_impl =
            jackctl_server_get_drivers_list_impl.map(|sym| sym.into_raw());
        let jackctl_server_get_drivers_list_impl =
            jackctl_server_get_drivers_list_impl.map(|sym| {
                *sym.deref() as unsafe extern "C" fn(server: *mut jackctl_server_t) -> *const JSList
            });
        let jackctl_server_get_parameters_impl = library
            .get::<unsafe extern "C" fn(server: *mut jackctl_server_t) -> *const JSList>(
                b"jackctl_server_get_parameters",
            )
            .ok();
        let jackctl_server_get_parameters_impl =
            jackctl_server_get_parameters_impl.map(|sym| sym.into_raw());
        let jackctl_server_get_parameters_impl = jackctl_server_get_parameters_impl.map(|sym| {
            *sym.deref() as unsafe extern "C" fn(server: *mut jackctl_server_t) -> *const JSList
        });
        let jackctl_server_get_internals_list_impl = library
            .get::<unsafe extern "C" fn(server: *mut jackctl_server_t) -> *const JSList>(
                b"jackctl_server_get_internals_list",
            )
            .ok();
        let jackctl_server_get_internals_list_impl =
            jackctl_server_get_internals_list_impl.map(|sym| sym.into_raw());
        let jackctl_server_get_internals_list_impl =
            jackctl_server_get_internals_list_impl.map(|sym| {
                *sym.deref() as unsafe extern "C" fn(server: *mut jackctl_server_t) -> *const JSList
            });
        let jackctl_server_load_internal_impl = library
            .get::<unsafe extern "C" fn(
                server: *mut jackctl_server_t,
                internal: *mut jackctl_internal_t,
            ) -> u8>(b"jackctl_server_load_internal")
            .ok();
        let jackctl_server_load_internal_impl =
            jackctl_server_load_internal_impl.map(|sym| sym.into_raw());
        let jackctl_server_load_internal_impl = jackctl_server_load_internal_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    server: *mut jackctl_server_t,
                    internal: *mut jackctl_internal_t,
                ) -> u8
        });
        let jackctl_server_unload_internal_impl = library
            .get::<unsafe extern "C" fn(
                server: *mut jackctl_server_t,
                internal: *mut jackctl_internal_t,
            ) -> u8>(b"jackctl_server_unload_internal")
            .ok();
        let jackctl_server_unload_internal_impl =
            jackctl_server_unload_internal_impl.map(|sym| sym.into_raw());
        let jackctl_server_unload_internal_impl = jackctl_server_unload_internal_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    server: *mut jackctl_server_t,
                    internal: *mut jackctl_internal_t,
                ) -> u8
        });
        let jackctl_server_add_slave_impl =
            library
                .get::<unsafe extern "C" fn(
                    server: *mut jackctl_server_t,
                    driver: *mut jackctl_driver_t,
                ) -> u8>(b"jackctl_server_add_slave")
                .ok();
        let jackctl_server_add_slave_impl = jackctl_server_add_slave_impl.map(|sym| sym.into_raw());
        let jackctl_server_add_slave_impl = jackctl_server_add_slave_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    server: *mut jackctl_server_t,
                    driver: *mut jackctl_driver_t,
                ) -> u8
        });
        let jackctl_server_remove_slave_impl =
            library
                .get::<unsafe extern "C" fn(
                    server: *mut jackctl_server_t,
                    driver: *mut jackctl_driver_t,
                ) -> u8>(b"jackctl_server_remove_slave")
                .ok();
        let jackctl_server_remove_slave_impl =
            jackctl_server_remove_slave_impl.map(|sym| sym.into_raw());
        let jackctl_server_remove_slave_impl = jackctl_server_remove_slave_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    server: *mut jackctl_server_t,
                    driver: *mut jackctl_driver_t,
                ) -> u8
        });
        let jackctl_server_switch_master_impl =
            library
                .get::<unsafe extern "C" fn(
                    server: *mut jackctl_server_t,
                    driver: *mut jackctl_driver_t,
                ) -> u8>(b"jackctl_server_switch_master")
                .ok();
        let jackctl_server_switch_master_impl =
            jackctl_server_switch_master_impl.map(|sym| sym.into_raw());
        let jackctl_server_switch_master_impl = jackctl_server_switch_master_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    server: *mut jackctl_server_t,
                    driver: *mut jackctl_driver_t,
                ) -> u8
        });
        let jackctl_driver_get_name_impl = library
            .get::<unsafe extern "C" fn(driver: *mut jackctl_driver_t) -> *const ::libc::c_char>(
                b"jackctl_driver_get_name",
            )
            .ok();
        let jackctl_driver_get_name_impl = jackctl_driver_get_name_impl.map(|sym| sym.into_raw());
        let jackctl_driver_get_name_impl = jackctl_driver_get_name_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(driver: *mut jackctl_driver_t) -> *const ::libc::c_char
        });
        let jackctl_driver_get_type_impl = library
            .get::<unsafe extern "C" fn(driver: *mut jackctl_driver_t) -> jackctl_driver_type_t>(
                b"jackctl_driver_get_type",
            )
            .ok();
        let jackctl_driver_get_type_impl = jackctl_driver_get_type_impl.map(|sym| sym.into_raw());
        let jackctl_driver_get_type_impl = jackctl_driver_get_type_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(driver: *mut jackctl_driver_t) -> jackctl_driver_type_t
        });
        let jackctl_driver_get_parameters_impl = library
            .get::<unsafe extern "C" fn(driver: *mut jackctl_driver_t) -> *const JSList>(
                b"jackctl_driver_get_parameters",
            )
            .ok();
        let jackctl_driver_get_parameters_impl =
            jackctl_driver_get_parameters_impl.map(|sym| sym.into_raw());
        let jackctl_driver_get_parameters_impl = jackctl_driver_get_parameters_impl.map(|sym| {
            *sym.deref() as unsafe extern "C" fn(driver: *mut jackctl_driver_t) -> *const JSList
        });
        let jackctl_driver_params_parse_impl = library
            .get::<unsafe extern "C" fn(
                driver: *mut jackctl_driver_t,
                argc: ::libc::c_int,
                argv: *mut *mut ::libc::c_char,
            ) -> ::libc::c_int>(b"jackctl_driver_params_parse")
            .ok();
        let jackctl_driver_params_parse_impl =
            jackctl_driver_params_parse_impl.map(|sym| sym.into_raw());
        let jackctl_driver_params_parse_impl = jackctl_driver_params_parse_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    driver: *mut jackctl_driver_t,
                    argc: ::libc::c_int,
                    argv: *mut *mut ::libc::c_char,
                ) -> ::libc::c_int
        });
        let jackctl_internal_get_name_impl = library.get::<unsafe extern "C" fn(internal: *mut jackctl_internal_t) -> *const ::libc::c_char>(b"jackctl_internal_get_name").ok();
        let jackctl_internal_get_name_impl =
            jackctl_internal_get_name_impl.map(|sym| sym.into_raw());
        let jackctl_internal_get_name_impl = jackctl_internal_get_name_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(internal: *mut jackctl_internal_t) -> *const ::libc::c_char
        });
        let jackctl_internal_get_parameters_impl = library
            .get::<unsafe extern "C" fn(internal: *mut jackctl_internal_t) -> *const JSList>(
                b"jackctl_internal_get_parameters",
            )
            .ok();
        let jackctl_internal_get_parameters_impl =
            jackctl_internal_get_parameters_impl.map(|sym| sym.into_raw());
        let jackctl_internal_get_parameters_impl =
            jackctl_internal_get_parameters_impl.map(|sym| {
                *sym.deref()
                    as unsafe extern "C" fn(internal: *mut jackctl_internal_t) -> *const JSList
            });
        let jackctl_parameter_get_name_impl = library.get::<unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> *const ::libc::c_char>(b"jackctl_parameter_get_name").ok();
        let jackctl_parameter_get_name_impl =
            jackctl_parameter_get_name_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_get_name_impl = jackctl_parameter_get_name_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    parameter: *mut jackctl_parameter_t,
                ) -> *const ::libc::c_char
        });
        let jackctl_parameter_get_short_description_impl = library.get::<unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> *const ::libc::c_char>(b"jackctl_parameter_get_short_description").ok();
        let jackctl_parameter_get_short_description_impl =
            jackctl_parameter_get_short_description_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_get_short_description_impl =
            jackctl_parameter_get_short_description_impl.map(|sym| {
                *sym.deref()
                    as unsafe extern "C" fn(
                        parameter: *mut jackctl_parameter_t,
                    ) -> *const ::libc::c_char
            });
        let jackctl_parameter_get_long_description_impl = library.get::<unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> *const ::libc::c_char>(b"jackctl_parameter_get_long_description").ok();
        let jackctl_parameter_get_long_description_impl =
            jackctl_parameter_get_long_description_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_get_long_description_impl =
            jackctl_parameter_get_long_description_impl.map(|sym| {
                *sym.deref()
                    as unsafe extern "C" fn(
                        parameter: *mut jackctl_parameter_t,
                    ) -> *const ::libc::c_char
            });
        let jackctl_parameter_get_type_impl = library.get::<unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> jackctl_param_type_t>(b"jackctl_parameter_get_type").ok();
        let jackctl_parameter_get_type_impl =
            jackctl_parameter_get_type_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_get_type_impl = jackctl_parameter_get_type_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> jackctl_param_type_t
        });
        let jackctl_parameter_get_id_impl = library
            .get::<unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> ::libc::c_char>(
                b"jackctl_parameter_get_id",
            )
            .ok();
        let jackctl_parameter_get_id_impl = jackctl_parameter_get_id_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_get_id_impl = jackctl_parameter_get_id_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> ::libc::c_char
        });
        let jackctl_parameter_is_set_impl = library
            .get::<unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> u8>(
                b"jackctl_parameter_is_set",
            )
            .ok();
        let jackctl_parameter_is_set_impl = jackctl_parameter_is_set_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_is_set_impl = jackctl_parameter_is_set_impl.map(|sym| {
            *sym.deref() as unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> u8
        });
        let jackctl_parameter_reset_impl = library
            .get::<unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> u8>(
                b"jackctl_parameter_reset",
            )
            .ok();
        let jackctl_parameter_reset_impl = jackctl_parameter_reset_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_reset_impl = jackctl_parameter_reset_impl.map(|sym| {
            *sym.deref() as unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> u8
        });
        let jackctl_parameter_get_value_impl = library
            .get::<unsafe extern "C" fn(
                parameter: *mut jackctl_parameter_t,
            ) -> Union_jackctl_parameter_value>(b"jackctl_parameter_get_value")
            .ok();
        let jackctl_parameter_get_value_impl =
            jackctl_parameter_get_value_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_get_value_impl = jackctl_parameter_get_value_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    parameter: *mut jackctl_parameter_t,
                ) -> Union_jackctl_parameter_value
        });
        let jackctl_parameter_set_value_impl = library
            .get::<unsafe extern "C" fn(
                parameter: *mut jackctl_parameter_t,
                value_ptr: *const Union_jackctl_parameter_value,
            ) -> u8>(b"jackctl_parameter_set_value")
            .ok();
        let jackctl_parameter_set_value_impl =
            jackctl_parameter_set_value_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_set_value_impl = jackctl_parameter_set_value_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    parameter: *mut jackctl_parameter_t,
                    value_ptr: *const Union_jackctl_parameter_value,
                ) -> u8
        });
        let jackctl_parameter_get_default_value_impl = library
            .get::<unsafe extern "C" fn(
                parameter: *mut jackctl_parameter_t,
            ) -> Union_jackctl_parameter_value>(b"jackctl_parameter_get_default_value")
            .ok();
        let jackctl_parameter_get_default_value_impl =
            jackctl_parameter_get_default_value_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_get_default_value_impl = jackctl_parameter_get_default_value_impl
            .map(|sym| {
                *sym.deref()
                    as unsafe extern "C" fn(
                        parameter: *mut jackctl_parameter_t,
                    ) -> Union_jackctl_parameter_value
            });
        let jackctl_parameter_has_range_constraint_impl = library
            .get::<unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> u8>(
                b"jackctl_parameter_has_range_constraint",
            )
            .ok();
        let jackctl_parameter_has_range_constraint_impl =
            jackctl_parameter_has_range_constraint_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_has_range_constraint_impl =
            jackctl_parameter_has_range_constraint_impl.map(|sym| {
                *sym.deref() as unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> u8
            });
        let jackctl_parameter_has_enum_constraint_impl = library
            .get::<unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> u8>(
                b"jackctl_parameter_has_enum_constraint",
            )
            .ok();
        let jackctl_parameter_has_enum_constraint_impl =
            jackctl_parameter_has_enum_constraint_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_has_enum_constraint_impl = jackctl_parameter_has_enum_constraint_impl
            .map(|sym| {
                *sym.deref() as unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> u8
            });
        let jackctl_parameter_get_enum_constraints_count_impl = library
            .get::<unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> u32>(
                b"jackctl_parameter_get_enum_constraints_count",
            )
            .ok();
        let jackctl_parameter_get_enum_constraints_count_impl =
            jackctl_parameter_get_enum_constraints_count_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_get_enum_constraints_count_impl =
            jackctl_parameter_get_enum_constraints_count_impl.map(|sym| {
                *sym.deref() as unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> u32
            });
        let jackctl_parameter_get_enum_constraint_value_impl = library
            .get::<unsafe extern "C" fn(
                parameter: *mut jackctl_parameter_t,
                index: u32,
            ) -> Union_jackctl_parameter_value>(
                b"jackctl_parameter_get_enum_constraint_value"
            )
            .ok();
        let jackctl_parameter_get_enum_constraint_value_impl =
            jackctl_parameter_get_enum_constraint_value_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_get_enum_constraint_value_impl =
            jackctl_parameter_get_enum_constraint_value_impl.map(|sym| {
                *sym.deref()
                    as unsafe extern "C" fn(
                        parameter: *mut jackctl_parameter_t,
                        index: u32,
                    ) -> Union_jackctl_parameter_value
            });
        let jackctl_parameter_get_enum_constraint_description_impl = library
            .get::<unsafe extern "C" fn(
                parameter: *mut jackctl_parameter_t,
                index: u32,
            ) -> *const ::libc::c_char>(
                b"jackctl_parameter_get_enum_constraint_description"
            )
            .ok();
        let jackctl_parameter_get_enum_constraint_description_impl =
            jackctl_parameter_get_enum_constraint_description_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_get_enum_constraint_description_impl =
            jackctl_parameter_get_enum_constraint_description_impl.map(|sym| {
                *sym.deref()
                    as unsafe extern "C" fn(
                        parameter: *mut jackctl_parameter_t,
                        index: u32,
                    ) -> *const ::libc::c_char
            });
        let jackctl_parameter_get_range_constraint_impl = library
            .get::<unsafe extern "C" fn(
                parameter: *mut jackctl_parameter_t,
                min_ptr: *mut Union_jackctl_parameter_value,
                max_ptr: *mut Union_jackctl_parameter_value,
            ) -> ()>(b"jackctl_parameter_get_range_constraint")
            .ok();
        let jackctl_parameter_get_range_constraint_impl =
            jackctl_parameter_get_range_constraint_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_get_range_constraint_impl =
            jackctl_parameter_get_range_constraint_impl.map(|sym| {
                *sym.deref()
                    as unsafe extern "C" fn(
                        parameter: *mut jackctl_parameter_t,
                        min_ptr: *mut Union_jackctl_parameter_value,
                        max_ptr: *mut Union_jackctl_parameter_value,
                    ) -> ()
            });
        let jackctl_parameter_constraint_is_strict_impl = library
            .get::<unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> u8>(
                b"jackctl_parameter_constraint_is_strict",
            )
            .ok();
        let jackctl_parameter_constraint_is_strict_impl =
            jackctl_parameter_constraint_is_strict_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_constraint_is_strict_impl =
            jackctl_parameter_constraint_is_strict_impl.map(|sym| {
                *sym.deref() as unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> u8
            });
        let jackctl_parameter_constraint_is_fake_value_impl = library
            .get::<unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> u8>(
                b"jackctl_parameter_constraint_is_fake_value",
            )
            .ok();
        let jackctl_parameter_constraint_is_fake_value_impl =
            jackctl_parameter_constraint_is_fake_value_impl.map(|sym| sym.into_raw());
        let jackctl_parameter_constraint_is_fake_value_impl =
            jackctl_parameter_constraint_is_fake_value_impl.map(|sym| {
                *sym.deref() as unsafe extern "C" fn(parameter: *mut jackctl_parameter_t) -> u8
            });
        let jack_set_property_impl = library
            .get::<unsafe extern "C" fn(
                arg1: *mut jack_client_t,
                subject: jack_uuid_t,
                key: *const ::libc::c_char,
                value: *const ::libc::c_char,
                _type: *const ::libc::c_char,
            ) -> ::libc::c_int>(b"jack_set_property")
            .unwrap();
        let jack_set_property_impl = jack_set_property_impl.into_raw();
        let jack_set_property_impl = *jack_set_property_impl.deref()
            as unsafe extern "C" fn(
                arg1: *mut jack_client_t,
                subject: jack_uuid_t,
                key: *const ::libc::c_char,
                value: *const ::libc::c_char,
                _type: *const ::libc::c_char,
            ) -> ::libc::c_int;
        let jack_get_property_impl = library
            .get::<unsafe extern "C" fn(
                subject: jack_uuid_t,
                key: *const ::libc::c_char,
                value: *mut *mut ::libc::c_char,
                _type: *mut *mut ::libc::c_char,
            ) -> ::libc::c_int>(b"jack_get_property")
            .unwrap();
        let jack_get_property_impl = jack_get_property_impl.into_raw();
        let jack_get_property_impl = *jack_get_property_impl.deref()
            as unsafe extern "C" fn(
                subject: jack_uuid_t,
                key: *const ::libc::c_char,
                value: *mut *mut ::libc::c_char,
                _type: *mut *mut ::libc::c_char,
            ) -> ::libc::c_int;
        let jack_free_description_impl = library
            .get::<unsafe extern "C" fn(
                desc: *mut jack_description_t,
                free_description_itself: ::libc::c_int,
            ) -> ()>(b"jack_free_description")
            .unwrap();
        let jack_free_description_impl = jack_free_description_impl.into_raw();
        let jack_free_description_impl = *jack_free_description_impl.deref()
            as unsafe extern "C" fn(
                desc: *mut jack_description_t,
                free_description_itself: ::libc::c_int,
            ) -> ();
        let jack_get_properties_impl = library
            .get::<unsafe extern "C" fn(
                subject: jack_uuid_t,
                desc: *mut jack_description_t,
            ) -> ::libc::c_int>(b"jack_get_properties")
            .unwrap();
        let jack_get_properties_impl = jack_get_properties_impl.into_raw();
        let jack_get_properties_impl = *jack_get_properties_impl.deref()
            as unsafe extern "C" fn(
                subject: jack_uuid_t,
                desc: *mut jack_description_t,
            ) -> ::libc::c_int;
        let jack_get_all_properties_impl = library
            .get::<unsafe extern "C" fn(descs: *mut *mut jack_description_t) -> ::libc::c_int>(
                b"jack_get_all_properties",
            )
            .unwrap();
        let jack_get_all_properties_impl = jack_get_all_properties_impl.into_raw();
        let jack_get_all_properties_impl = *jack_get_all_properties_impl.deref()
            as unsafe extern "C" fn(descs: *mut *mut jack_description_t) -> ::libc::c_int;
        let jack_remove_property_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                subject: jack_uuid_t,
                key: *const ::libc::c_char,
            ) -> ::libc::c_int>(b"jack_remove_property")
            .unwrap();
        let jack_remove_property_impl = jack_remove_property_impl.into_raw();
        let jack_remove_property_impl = *jack_remove_property_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                subject: jack_uuid_t,
                key: *const ::libc::c_char,
            ) -> ::libc::c_int;
        let jack_remove_properties_impl =
            library
                .get::<unsafe extern "C" fn(
                    client: *mut jack_client_t,
                    subject: jack_uuid_t,
                ) -> ::libc::c_int>(b"jack_remove_properties")
                .unwrap();
        let jack_remove_properties_impl = jack_remove_properties_impl.into_raw();
        let jack_remove_properties_impl = *jack_remove_properties_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                subject: jack_uuid_t,
            ) -> ::libc::c_int;
        let jack_remove_all_properties_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int>(
                b"jack_remove_all_properties",
            )
            .unwrap();
        let jack_remove_all_properties_impl = jack_remove_all_properties_impl.into_raw();
        let jack_remove_all_properties_impl = *jack_remove_all_properties_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_int;
        let jack_set_property_change_callback_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                callback: JackPropertyChangeCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int>(b"jack_set_property_change_callback")
            .unwrap();
        let jack_set_property_change_callback_impl =
            jack_set_property_change_callback_impl.into_raw();
        let jack_set_property_change_callback_impl = *jack_set_property_change_callback_impl.deref()
            as unsafe extern "C" fn(
                client: *mut jack_client_t,
                callback: JackPropertyChangeCallback,
                arg: *mut ::libc::c_void,
            ) -> ::libc::c_int;
        let jack_get_internal_client_name_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                intclient: jack_intclient_t,
            ) -> *mut ::libc::c_char>(b"jack_get_internal_client_name")
            .ok();
        let jack_get_internal_client_name_impl =
            jack_get_internal_client_name_impl.map(|sym| sym.into_raw());
        let jack_get_internal_client_name_impl = jack_get_internal_client_name_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    client: *mut jack_client_t,
                    intclient: jack_intclient_t,
                ) -> *mut ::libc::c_char
        });
        let jack_internal_client_handle_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                client_name: *const ::libc::c_char,
                status: *mut jack_status_t,
            ) -> jack_intclient_t>(b"jack_internal_client_handle")
            .ok();
        let jack_internal_client_handle_impl =
            jack_internal_client_handle_impl.map(|sym| sym.into_raw());
        let jack_internal_client_handle_impl = jack_internal_client_handle_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    client: *mut jack_client_t,
                    client_name: *const ::libc::c_char,
                    status: *mut jack_status_t,
                ) -> jack_intclient_t
        });
        let jack_internal_client_load_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                client_name: *const ::libc::c_char,
                options: jack_options_t,
                status: *mut jack_status_t,
                load_name: *const ::libc::c_char,
                load_init: *const ::libc::c_char,
            ) -> jack_intclient_t>(b"jack_internal_client_load")
            .ok();
        let jack_internal_client_load_impl =
            jack_internal_client_load_impl.map(|sym| sym.into_raw());
        let jack_internal_client_load_impl = jack_internal_client_load_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    client: *mut jack_client_t,
                    client_name: *const ::libc::c_char,
                    options: jack_options_t,
                    status: *mut jack_status_t,
                    load_name: *const ::libc::c_char,
                    load_init: *const ::libc::c_char,
                ) -> jack_intclient_t
        });
        let jack_internal_client_unload_impl = library
            .get::<unsafe extern "C" fn(
                client: *mut jack_client_t,
                intclient: jack_intclient_t,
            ) -> jack_status_t>(b"jack_internal_client_unload")
            .ok();
        let jack_internal_client_unload_impl =
            jack_internal_client_unload_impl.map(|sym| sym.into_raw());
        let jack_internal_client_unload_impl = jack_internal_client_unload_impl.map(|sym| {
            *sym.deref()
                as unsafe extern "C" fn(
                    client: *mut jack_client_t,
                    intclient: jack_intclient_t,
                ) -> jack_status_t
        });
        let jack_get_max_delayed_usecs_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_float>(
                b"jack_get_max_delayed_usecs",
            )
            .unwrap();
        let jack_get_max_delayed_usecs_impl = jack_get_max_delayed_usecs_impl.into_raw();
        let jack_get_max_delayed_usecs_impl = *jack_get_max_delayed_usecs_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_float;
        let jack_get_xrun_delayed_usecs_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_float>(
                b"jack_get_xrun_delayed_usecs",
            )
            .unwrap();
        let jack_get_xrun_delayed_usecs_impl = jack_get_xrun_delayed_usecs_impl.into_raw();
        let jack_get_xrun_delayed_usecs_impl = *jack_get_xrun_delayed_usecs_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> ::libc::c_float;
        let jack_reset_max_delayed_usecs_impl = library
            .get::<unsafe extern "C" fn(client: *mut jack_client_t) -> ()>(
                b"jack_reset_max_delayed_usecs",
            )
            .unwrap();
        let jack_reset_max_delayed_usecs_impl = jack_reset_max_delayed_usecs_impl.into_raw();
        let jack_reset_max_delayed_usecs_impl = *jack_reset_max_delayed_usecs_impl.deref()
            as unsafe extern "C" fn(client: *mut jack_client_t) -> ();
        let jack_midi_get_event_count_impl = library
            .get::<unsafe extern "C" fn(port_buffer: *mut ::libc::c_void) -> u32>(
                b"jack_midi_get_event_count",
            )
            .unwrap();
        let jack_midi_get_event_count_impl = jack_midi_get_event_count_impl.into_raw();
        let jack_midi_get_event_count_impl = *jack_midi_get_event_count_impl.deref()
            as unsafe extern "C" fn(port_buffer: *mut ::libc::c_void) -> u32;
        let jack_midi_event_get_impl = library
            .get::<unsafe extern "C" fn(
                event: *mut jack_midi_event_t,
                port_buffer: *mut ::libc::c_void,
                event_index: u32,
            ) -> ::libc::c_int>(b"jack_midi_event_get")
            .unwrap();
        let jack_midi_event_get_impl = jack_midi_event_get_impl.into_raw();
        let jack_midi_event_get_impl = *jack_midi_event_get_impl.deref()
            as unsafe extern "C" fn(
                event: *mut jack_midi_event_t,
                port_buffer: *mut ::libc::c_void,
                event_index: u32,
            ) -> ::libc::c_int;
        let jack_midi_clear_buffer_impl = library
            .get::<unsafe extern "C" fn(port_buffer: *mut ::libc::c_void) -> ()>(
                b"jack_midi_clear_buffer",
            )
            .unwrap();
        let jack_midi_clear_buffer_impl = jack_midi_clear_buffer_impl.into_raw();
        let jack_midi_clear_buffer_impl = *jack_midi_clear_buffer_impl.deref()
            as unsafe extern "C" fn(port_buffer: *mut ::libc::c_void) -> ();
        let jack_midi_max_event_size_impl = library
            .get::<unsafe extern "C" fn(port_buffer: *mut ::libc::c_void) -> ::libc::size_t>(
                b"jack_midi_max_event_size",
            )
            .unwrap();
        let jack_midi_max_event_size_impl = jack_midi_max_event_size_impl.into_raw();
        let jack_midi_max_event_size_impl = *jack_midi_max_event_size_impl.deref()
            as unsafe extern "C" fn(port_buffer: *mut ::libc::c_void) -> ::libc::size_t;
        let jack_midi_event_reserve_impl = library
            .get::<unsafe extern "C" fn(
                port_buffer: *mut ::libc::c_void,
                time: jack_nframes_t,
                data_size: ::libc::size_t,
            ) -> *mut jack_midi_data_t>(b"jack_midi_event_reserve")
            .unwrap();
        let jack_midi_event_reserve_impl = jack_midi_event_reserve_impl.into_raw();
        let jack_midi_event_reserve_impl = *jack_midi_event_reserve_impl.deref()
            as unsafe extern "C" fn(
                port_buffer: *mut ::libc::c_void,
                time: jack_nframes_t,
                data_size: ::libc::size_t,
            ) -> *mut jack_midi_data_t;
        let jack_midi_event_write_impl = library
            .get::<unsafe extern "C" fn(
                port_buffer: *mut ::libc::c_void,
                time: jack_nframes_t,
                data: *const jack_midi_data_t,
                data_size: ::libc::size_t,
            ) -> ::libc::c_int>(b"jack_midi_event_write")
            .unwrap();
        let jack_midi_event_write_impl = jack_midi_event_write_impl.into_raw();
        let jack_midi_event_write_impl = *jack_midi_event_write_impl.deref()
            as unsafe extern "C" fn(
                port_buffer: *mut ::libc::c_void,
                time: jack_nframes_t,
                data: *const jack_midi_data_t,
                data_size: ::libc::size_t,
            ) -> ::libc::c_int;
        let jack_midi_get_lost_event_count_impl = library
            .get::<unsafe extern "C" fn(port_buffer: *mut ::libc::c_void) -> u32>(
                b"jack_midi_get_lost_event_count",
            )
            .unwrap();
        let jack_midi_get_lost_event_count_impl = jack_midi_get_lost_event_count_impl.into_raw();
        let jack_midi_get_lost_event_count_impl = *jack_midi_get_lost_event_count_impl.deref()
            as unsafe extern "C" fn(port_buffer: *mut ::libc::c_void) -> u32;
        let jack_ringbuffer_create_impl = library
            .get::<unsafe extern "C" fn(sz: ::libc::size_t) -> *mut jack_ringbuffer_t>(
                b"jack_ringbuffer_create",
            )
            .unwrap();
        let jack_ringbuffer_create_impl = jack_ringbuffer_create_impl.into_raw();
        let jack_ringbuffer_create_impl = *jack_ringbuffer_create_impl.deref()
            as unsafe extern "C" fn(sz: ::libc::size_t) -> *mut jack_ringbuffer_t;
        let jack_ringbuffer_free_impl = library
            .get::<unsafe extern "C" fn(rb: *mut jack_ringbuffer_t) -> ()>(b"jack_ringbuffer_free")
            .unwrap();
        let jack_ringbuffer_free_impl = jack_ringbuffer_free_impl.into_raw();
        let jack_ringbuffer_free_impl = *jack_ringbuffer_free_impl.deref()
            as unsafe extern "C" fn(rb: *mut jack_ringbuffer_t) -> ();
        let jack_ringbuffer_get_read_vector_impl = library
            .get::<unsafe extern "C" fn(
                rb: *const jack_ringbuffer_t,
                vec: *mut jack_ringbuffer_data_t,
            ) -> ()>(b"jack_ringbuffer_get_read_vector")
            .unwrap();
        let jack_ringbuffer_get_read_vector_impl = jack_ringbuffer_get_read_vector_impl.into_raw();
        let jack_ringbuffer_get_read_vector_impl = *jack_ringbuffer_get_read_vector_impl.deref()
            as unsafe extern "C" fn(
                rb: *const jack_ringbuffer_t,
                vec: *mut jack_ringbuffer_data_t,
            ) -> ();
        let jack_ringbuffer_get_write_vector_impl = library
            .get::<unsafe extern "C" fn(
                rb: *const jack_ringbuffer_t,
                vec: *mut jack_ringbuffer_data_t,
            ) -> ()>(b"jack_ringbuffer_get_write_vector")
            .unwrap();
        let jack_ringbuffer_get_write_vector_impl =
            jack_ringbuffer_get_write_vector_impl.into_raw();
        let jack_ringbuffer_get_write_vector_impl = *jack_ringbuffer_get_write_vector_impl.deref()
            as unsafe extern "C" fn(
                rb: *const jack_ringbuffer_t,
                vec: *mut jack_ringbuffer_data_t,
            ) -> ();
        let jack_ringbuffer_read_impl = library
            .get::<unsafe extern "C" fn(
                rb: *mut jack_ringbuffer_t,
                dest: *mut ::libc::c_char,
                cnt: ::libc::size_t,
            ) -> ::libc::size_t>(b"jack_ringbuffer_read")
            .unwrap();
        let jack_ringbuffer_read_impl = jack_ringbuffer_read_impl.into_raw();
        let jack_ringbuffer_read_impl = *jack_ringbuffer_read_impl.deref()
            as unsafe extern "C" fn(
                rb: *mut jack_ringbuffer_t,
                dest: *mut ::libc::c_char,
                cnt: ::libc::size_t,
            ) -> ::libc::size_t;
        let jack_ringbuffer_peek_impl = library
            .get::<unsafe extern "C" fn(
                rb: *mut jack_ringbuffer_t,
                dest: *mut ::libc::c_char,
                cnt: ::libc::size_t,
            ) -> ::libc::size_t>(b"jack_ringbuffer_peek")
            .unwrap();
        let jack_ringbuffer_peek_impl = jack_ringbuffer_peek_impl.into_raw();
        let jack_ringbuffer_peek_impl = *jack_ringbuffer_peek_impl.deref()
            as unsafe extern "C" fn(
                rb: *mut jack_ringbuffer_t,
                dest: *mut ::libc::c_char,
                cnt: ::libc::size_t,
            ) -> ::libc::size_t;
        let jack_ringbuffer_read_advance_impl = library
            .get::<unsafe extern "C" fn(rb: *mut jack_ringbuffer_t, cnt: ::libc::size_t) -> ()>(
                b"jack_ringbuffer_read_advance",
            )
            .unwrap();
        let jack_ringbuffer_read_advance_impl = jack_ringbuffer_read_advance_impl.into_raw();
        let jack_ringbuffer_read_advance_impl = *jack_ringbuffer_read_advance_impl.deref()
            as unsafe extern "C" fn(rb: *mut jack_ringbuffer_t, cnt: ::libc::size_t) -> ();
        let jack_ringbuffer_read_space_impl = library
            .get::<unsafe extern "C" fn(rb: *const jack_ringbuffer_t) -> ::libc::size_t>(
                b"jack_ringbuffer_read_space",
            )
            .unwrap();
        let jack_ringbuffer_read_space_impl = jack_ringbuffer_read_space_impl.into_raw();
        let jack_ringbuffer_read_space_impl = *jack_ringbuffer_read_space_impl.deref()
            as unsafe extern "C" fn(rb: *const jack_ringbuffer_t) -> ::libc::size_t;
        let jack_ringbuffer_mlock_impl = library
            .get::<unsafe extern "C" fn(rb: *mut jack_ringbuffer_t) -> ::libc::c_int>(
                b"jack_ringbuffer_mlock",
            )
            .unwrap();
        let jack_ringbuffer_mlock_impl = jack_ringbuffer_mlock_impl.into_raw();
        let jack_ringbuffer_mlock_impl = *jack_ringbuffer_mlock_impl.deref()
            as unsafe extern "C" fn(rb: *mut jack_ringbuffer_t) -> ::libc::c_int;
        let jack_ringbuffer_reset_impl = library
            .get::<unsafe extern "C" fn(rb: *mut jack_ringbuffer_t) -> ()>(b"jack_ringbuffer_reset")
            .unwrap();
        let jack_ringbuffer_reset_impl = jack_ringbuffer_reset_impl.into_raw();
        let jack_ringbuffer_reset_impl = *jack_ringbuffer_reset_impl.deref()
            as unsafe extern "C" fn(rb: *mut jack_ringbuffer_t) -> ();
        let jack_ringbuffer_write_impl = library
            .get::<unsafe extern "C" fn(
                rb: *mut jack_ringbuffer_t,
                src: *const ::libc::c_char,
                cnt: ::libc::size_t,
            ) -> ::libc::size_t>(b"jack_ringbuffer_write")
            .unwrap();
        let jack_ringbuffer_write_impl = jack_ringbuffer_write_impl.into_raw();
        let jack_ringbuffer_write_impl = *jack_ringbuffer_write_impl.deref()
            as unsafe extern "C" fn(
                rb: *mut jack_ringbuffer_t,
                src: *const ::libc::c_char,
                cnt: ::libc::size_t,
            ) -> ::libc::size_t;
        let jack_ringbuffer_write_advance_impl = library
            .get::<unsafe extern "C" fn(rb: *mut jack_ringbuffer_t, cnt: ::libc::size_t) -> ()>(
                b"jack_ringbuffer_write_advance",
            )
            .unwrap();
        let jack_ringbuffer_write_advance_impl = jack_ringbuffer_write_advance_impl.into_raw();
        let jack_ringbuffer_write_advance_impl = *jack_ringbuffer_write_advance_impl.deref()
            as unsafe extern "C" fn(rb: *mut jack_ringbuffer_t, cnt: ::libc::size_t) -> ();
        let jack_ringbuffer_write_space_impl = library
            .get::<unsafe extern "C" fn(rb: *const jack_ringbuffer_t) -> ::libc::size_t>(
                b"jack_ringbuffer_write_space",
            )
            .unwrap();
        let jack_ringbuffer_write_space_impl = jack_ringbuffer_write_space_impl.into_raw();
        let jack_ringbuffer_write_space_impl = *jack_ringbuffer_write_space_impl.deref()
            as unsafe extern "C" fn(rb: *const jack_ringbuffer_t) -> ::libc::size_t;
        let jack_uuid_to_index_impl = library
            .get::<unsafe extern "C" fn(arg1: jack_uuid_t) -> u32>(b"jack_uuid_to_index")
            .unwrap();
        let jack_uuid_to_index_impl = jack_uuid_to_index_impl.into_raw();
        let jack_uuid_to_index_impl =
            *jack_uuid_to_index_impl.deref() as unsafe extern "C" fn(arg1: jack_uuid_t) -> u32;
        let jack_uuid_compare_impl = library.get::<unsafe extern "C" fn(arg1: jack_uuid_t, arg2: jack_uuid_t) -> ::std::os::raw::c_int>(b"jack_uuid_compare").unwrap();
        let jack_uuid_compare_impl = jack_uuid_compare_impl.into_raw();
        let jack_uuid_compare_impl = *jack_uuid_compare_impl.deref()
            as unsafe extern "C" fn(arg1: jack_uuid_t, arg2: jack_uuid_t) -> ::std::os::raw::c_int;
        let jack_uuid_copy_impl = library
            .get::<unsafe extern "C" fn(dst: *mut jack_uuid_t, src: jack_uuid_t) -> ()>(
                b"jack_uuid_copy",
            )
            .unwrap();
        let jack_uuid_copy_impl = jack_uuid_copy_impl.into_raw();
        let jack_uuid_copy_impl = *jack_uuid_copy_impl.deref()
            as unsafe extern "C" fn(dst: *mut jack_uuid_t, src: jack_uuid_t) -> ();
        let jack_uuid_clear_impl = library
            .get::<unsafe extern "C" fn(arg1: *mut jack_uuid_t) -> ()>(b"jack_uuid_clear")
            .unwrap();
        let jack_uuid_clear_impl = jack_uuid_clear_impl.into_raw();
        let jack_uuid_clear_impl =
            *jack_uuid_clear_impl.deref() as unsafe extern "C" fn(arg1: *mut jack_uuid_t) -> ();
        let jack_uuid_parse_impl = library
            .get::<unsafe extern "C" fn(
                buf: *const ::std::os::raw::c_char,
                arg1: *mut jack_uuid_t,
            ) -> ::std::os::raw::c_int>(b"jack_uuid_parse")
            .unwrap();
        let jack_uuid_parse_impl = jack_uuid_parse_impl.into_raw();
        let jack_uuid_parse_impl = *jack_uuid_parse_impl.deref()
            as unsafe extern "C" fn(
                buf: *const ::std::os::raw::c_char,
                arg1: *mut jack_uuid_t,
            ) -> ::std::os::raw::c_int;
        let jack_uuid_unparse_impl = library
            .get::<unsafe extern "C" fn(arg1: jack_uuid_t, buf: *mut ::std::os::raw::c_char) -> ()>(
                b"jack_uuid_unparse",
            )
            .unwrap();
        let jack_uuid_unparse_impl = jack_uuid_unparse_impl.into_raw();
        let jack_uuid_unparse_impl = *jack_uuid_unparse_impl.deref()
            as unsafe extern "C" fn(arg1: jack_uuid_t, buf: *mut ::std::os::raw::c_char) -> ();
        let jack_uuid_empty_impl = library
            .get::<unsafe extern "C" fn(arg1: jack_uuid_t) -> ::std::os::raw::c_int>(
                b"jack_uuid_empty",
            )
            .unwrap();
        let jack_uuid_empty_impl = jack_uuid_empty_impl.into_raw();
        let jack_uuid_empty_impl = *jack_uuid_empty_impl.deref()
            as unsafe extern "C" fn(arg1: jack_uuid_t) -> ::std::os::raw::c_int;
        JackFunctions {
            jack_release_timebase_impl,
            jack_get_cycle_times_impl,
            jack_set_sync_callback_impl,
            jack_set_sync_timeout_impl,
            jack_set_timebase_callback_impl,
            jack_transport_locate_impl,
            jack_transport_query_impl,
            jack_get_current_transport_frame_impl,
            jack_transport_reposition_impl,
            jack_transport_start_impl,
            jack_transport_stop_impl,
            jack_get_transport_info_impl,
            jack_set_transport_info_impl,
            jack_client_open_impl,
            jack_client_new_impl,
            jack_client_close_impl,
            jack_client_name_size_impl,
            jack_get_client_name_impl,
            jack_get_uuid_for_client_name_impl,
            jack_get_client_name_by_uuid_impl,
            jack_internal_client_new_impl,
            jack_internal_client_close_impl,
            jack_activate_impl,
            jack_deactivate_impl,
            jack_get_client_pid_impl,
            jack_is_realtime_impl,
            jack_thread_wait_impl,
            jack_cycle_wait_impl,
            jack_cycle_signal_impl,
            jack_set_process_thread_impl,
            jack_set_thread_init_callback_impl,
            jack_on_shutdown_impl,
            jack_on_info_shutdown_impl,
            jack_set_process_callback_impl,
            jack_set_freewheel_callback_impl,
            jack_set_buffer_size_callback_impl,
            jack_get_sample_rate_impl,
            jack_set_sample_rate_callback_impl,
            jack_set_client_registration_callback_impl,
            jack_set_port_registration_callback_impl,
            jack_set_port_connect_callback_impl,
            jack_set_port_rename_callback_impl,
            jack_set_graph_order_callback_impl,
            jack_set_xrun_callback_impl,
            jack_set_latency_callback_impl,
            jack_set_freewheel_impl,
            jack_set_buffer_size_impl,
            jack_get_buffer_size_impl,
            jack_engine_takeover_timebase_impl,
            jack_cpu_load_impl,
            jack_port_register_impl,
            jack_port_unregister_impl,
            jack_port_get_buffer_impl,
            jack_port_uuid_impl,
            jack_port_name_impl,
            jack_port_short_name_impl,
            jack_port_flags_impl,
            jack_port_type_impl,
            jack_port_type_id_impl,
            jack_port_is_mine_impl,
            jack_port_connected_impl,
            jack_port_connected_to_impl,
            jack_port_get_connections_impl,
            jack_port_get_all_connections_impl,
            jack_port_tie_impl,
            jack_port_untie_impl,
            jack_port_set_name_impl,
            jack_port_set_alias_impl,
            jack_port_unset_alias_impl,
            jack_port_get_aliases_impl,
            jack_port_request_monitor_impl,
            jack_port_request_monitor_by_name_impl,
            jack_port_ensure_monitor_impl,
            jack_port_monitoring_input_impl,
            jack_connect_impl,
            jack_disconnect_impl,
            jack_port_disconnect_impl,
            jack_port_name_size_impl,
            jack_port_type_size_impl,
            jack_port_type_get_buffer_size_impl,
            jack_port_set_latency_impl,
            jack_port_get_latency_range_impl,
            jack_port_set_latency_range_impl,
            jack_recompute_total_latencies_impl,
            jack_port_get_latency_impl,
            jack_port_get_total_latency_impl,
            jack_recompute_total_latency_impl,
            jack_get_ports_impl,
            jack_port_by_name_impl,
            jack_port_by_id_impl,
            jack_frames_since_cycle_start_impl,
            jack_frame_time_impl,
            jack_last_frame_time_impl,
            jack_frames_to_time_impl,
            jack_time_to_frames_impl,
            jack_get_time_impl,
            jack_set_error_function_impl,
            jack_set_info_function_impl,
            jack_free_impl,
            jack_client_real_time_priority_impl,
            jack_client_max_real_time_priority_impl,
            jack_set_session_callback_impl,
            jack_session_reply_impl,
            jack_session_event_free_impl,
            jack_client_get_uuid_impl,
            jack_session_notify_impl,
            jack_session_commands_free_impl,
            jack_reserve_client_name_impl,
            jack_client_has_session_callback_impl,
            jackctl_setup_signals_impl,
            jackctl_wait_signals_impl,
            jackctl_server_create_impl,
            jackctl_server_destroy_impl,
            jackctl_server_open_impl,
            jackctl_server_start_impl,
            jackctl_server_stop_impl,
            jackctl_server_close_impl,
            jackctl_server_get_drivers_list_impl,
            jackctl_server_get_parameters_impl,
            jackctl_server_get_internals_list_impl,
            jackctl_server_load_internal_impl,
            jackctl_server_unload_internal_impl,
            jackctl_server_add_slave_impl,
            jackctl_server_remove_slave_impl,
            jackctl_server_switch_master_impl,
            jackctl_driver_get_name_impl,
            jackctl_driver_get_type_impl,
            jackctl_driver_get_parameters_impl,
            jackctl_driver_params_parse_impl,
            jackctl_internal_get_name_impl,
            jackctl_internal_get_parameters_impl,
            jackctl_parameter_get_name_impl,
            jackctl_parameter_get_short_description_impl,
            jackctl_parameter_get_long_description_impl,
            jackctl_parameter_get_type_impl,
            jackctl_parameter_get_id_impl,
            jackctl_parameter_is_set_impl,
            jackctl_parameter_reset_impl,
            jackctl_parameter_get_value_impl,
            jackctl_parameter_set_value_impl,
            jackctl_parameter_get_default_value_impl,
            jackctl_parameter_has_range_constraint_impl,
            jackctl_parameter_has_enum_constraint_impl,
            jackctl_parameter_get_enum_constraints_count_impl,
            jackctl_parameter_get_enum_constraint_value_impl,
            jackctl_parameter_get_enum_constraint_description_impl,
            jackctl_parameter_get_range_constraint_impl,
            jackctl_parameter_constraint_is_strict_impl,
            jackctl_parameter_constraint_is_fake_value_impl,
            jack_set_property_impl,
            jack_get_property_impl,
            jack_free_description_impl,
            jack_get_properties_impl,
            jack_get_all_properties_impl,
            jack_remove_property_impl,
            jack_remove_properties_impl,
            jack_remove_all_properties_impl,
            jack_set_property_change_callback_impl,
            jack_get_internal_client_name_impl,
            jack_internal_client_handle_impl,
            jack_internal_client_load_impl,
            jack_internal_client_unload_impl,
            jack_get_max_delayed_usecs_impl,
            jack_get_xrun_delayed_usecs_impl,
            jack_reset_max_delayed_usecs_impl,
            jack_midi_get_event_count_impl,
            jack_midi_event_get_impl,
            jack_midi_clear_buffer_impl,
            jack_midi_max_event_size_impl,
            jack_midi_event_reserve_impl,
            jack_midi_event_write_impl,
            jack_midi_get_lost_event_count_impl,
            jack_ringbuffer_create_impl,
            jack_ringbuffer_free_impl,
            jack_ringbuffer_get_read_vector_impl,
            jack_ringbuffer_get_write_vector_impl,
            jack_ringbuffer_read_impl,
            jack_ringbuffer_peek_impl,
            jack_ringbuffer_read_advance_impl,
            jack_ringbuffer_read_space_impl,
            jack_ringbuffer_mlock_impl,
            jack_ringbuffer_reset_impl,
            jack_ringbuffer_write_impl,
            jack_ringbuffer_write_advance_impl,
            jack_ringbuffer_write_space_impl,
            jack_uuid_to_index_impl,
            jack_uuid_compare_impl,
            jack_uuid_copy_impl,
            jack_uuid_clear_impl,
            jack_uuid_parse_impl,
            jack_uuid_unparse_impl,
            jack_uuid_empty_impl,
        }
    };
}

pub unsafe fn jack_release_timebase(client: *mut jack_client_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_release_timebase_impl;
    f(client)
}
pub unsafe fn jack_get_cycle_times(
    client: *const jack_client_t,
    current_frames: *mut jack_nframes_t,
    current_usecs: *mut jack_time_t,
    next_usecs: *mut jack_time_t,
    period_usecs: *mut ::libc::c_float,
) -> Option<::libc::c_int> {
    let f = FUNCTIONS.jack_get_cycle_times_impl?;
    Some(f(
        client,
        current_frames,
        current_usecs,
        next_usecs,
        period_usecs,
    ))
}
pub unsafe fn jack_set_sync_callback(
    client: *mut jack_client_t,
    sync_callback: JackSyncCallback,
    sync_arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_sync_callback_impl;
    f(client, sync_callback, sync_arg)
}
pub unsafe fn jack_set_sync_timeout(
    client: *mut jack_client_t,
    timeout: jack_time_t,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_sync_timeout_impl;
    f(client, timeout)
}
pub unsafe fn jack_set_timebase_callback(
    client: *mut jack_client_t,
    conditional: ::libc::c_int,
    timebase_callback: TimebaseCallback,
    arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_timebase_callback_impl;
    f(client, conditional, timebase_callback, arg)
}
pub unsafe fn jack_transport_locate(
    client: *mut jack_client_t,
    frame: jack_nframes_t,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_transport_locate_impl;
    f(client, frame)
}
pub unsafe fn jack_transport_query(
    client: *const jack_client_t,
    pos: *mut jack_position_t,
) -> jack_transport_state_t {
    let f = FUNCTIONS.jack_transport_query_impl;
    f(client, pos)
}
pub unsafe fn jack_get_current_transport_frame(client: *const jack_client_t) -> jack_nframes_t {
    let f = FUNCTIONS.jack_get_current_transport_frame_impl;
    f(client)
}
pub unsafe fn jack_transport_reposition(
    client: *mut jack_client_t,
    pos: *const jack_position_t,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_transport_reposition_impl;
    f(client, pos)
}
pub unsafe fn jack_transport_start(client: *mut jack_client_t) -> () {
    let f = FUNCTIONS.jack_transport_start_impl;
    f(client)
}
pub unsafe fn jack_transport_stop(client: *mut jack_client_t) -> () {
    let f = FUNCTIONS.jack_transport_stop_impl;
    f(client)
}
pub unsafe fn jack_get_transport_info(
    client: *mut jack_client_t,
    tinfo: *mut jack_transport_info_t,
) -> () {
    let f = FUNCTIONS.jack_get_transport_info_impl;
    f(client, tinfo)
}
pub unsafe fn jack_set_transport_info(
    client: *mut jack_client_t,
    tinfo: *mut jack_transport_info_t,
) -> () {
    let f = FUNCTIONS.jack_set_transport_info_impl;
    f(client, tinfo)
}
pub unsafe fn jack_client_open(
    client_name: *const ::libc::c_char,
    options: jack_options_t,
    status: *mut jack_status_t,
) -> *mut jack_client_t {
    let f = FUNCTIONS.jack_client_open_impl;
    f(client_name, options, status)
}
pub unsafe fn jack_client_new(client_name: *const ::libc::c_char) -> *mut jack_client_t {
    let f = FUNCTIONS.jack_client_new_impl;
    f(client_name)
}
pub unsafe fn jack_client_close(client: *mut jack_client_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_client_close_impl;
    f(client)
}
pub unsafe fn jack_client_name_size() -> ::libc::c_int {
    let f = FUNCTIONS.jack_client_name_size_impl;
    f()
}
pub unsafe fn jack_get_client_name(client: *mut jack_client_t) -> *mut ::libc::c_char {
    let f = FUNCTIONS.jack_get_client_name_impl;
    f(client)
}
pub unsafe fn jack_get_uuid_for_client_name(
    client: *mut jack_client_t,
    client_name: *const ::libc::c_char,
) -> *mut ::libc::c_char {
    let f = FUNCTIONS.jack_get_uuid_for_client_name_impl;
    f(client, client_name)
}
pub unsafe fn jack_get_client_name_by_uuid(
    client: *mut jack_client_t,
    client_uuid: *const ::libc::c_char,
) -> *mut ::libc::c_char {
    let f = FUNCTIONS.jack_get_client_name_by_uuid_impl;
    f(client, client_uuid)
}
pub unsafe fn jack_internal_client_new(
    client_name: *const ::libc::c_char,
    load_name: *const ::libc::c_char,
    load_init: *const ::libc::c_char,
) -> Option<::libc::c_int> {
    let f = FUNCTIONS.jack_internal_client_new_impl?;
    Some(f(client_name, load_name, load_init))
}
pub unsafe fn jack_internal_client_close(client_name: *const ::libc::c_char) -> Option<()> {
    let f = FUNCTIONS.jack_internal_client_close_impl?;
    Some(f(client_name))
}
pub unsafe fn jack_activate(client: *mut jack_client_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_activate_impl;
    f(client)
}
pub unsafe fn jack_deactivate(client: *mut jack_client_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_deactivate_impl;
    f(client)
}
pub unsafe fn jack_get_client_pid(name: *const ::libc::c_char) -> Option<::libc::c_int> {
    let f = FUNCTIONS.jack_get_client_pid_impl?;
    Some(f(name))
}
pub unsafe fn jack_is_realtime(client: *mut jack_client_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_is_realtime_impl;
    f(client)
}
pub unsafe fn jack_thread_wait(
    client: *mut jack_client_t,
    status: ::libc::c_int,
) -> jack_nframes_t {
    let f = FUNCTIONS.jack_thread_wait_impl;
    f(client, status)
}
pub unsafe fn jack_cycle_wait(client: *mut jack_client_t) -> jack_nframes_t {
    let f = FUNCTIONS.jack_cycle_wait_impl;
    f(client)
}
pub unsafe fn jack_cycle_signal(client: *mut jack_client_t, status: ::libc::c_int) -> () {
    let f = FUNCTIONS.jack_cycle_signal_impl;
    f(client, status)
}
pub unsafe fn jack_set_process_thread(
    client: *mut jack_client_t,
    thread_callback: JackThreadCallback,
    arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_process_thread_impl;
    f(client, thread_callback, arg)
}
pub unsafe fn jack_set_thread_init_callback(
    client: *mut jack_client_t,
    thread_init_callback: JackThreadInitCallback,
    arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    if thread_init_callback.is_some() {
        let f = FUNCTIONS.jack_set_thread_init_callback_impl;
        f(client, thread_init_callback, arg)
    } else {
        0
    }
}
pub unsafe fn jack_on_shutdown(
    client: *mut jack_client_t,
    callback: JackShutdownCallback,
    arg: *mut ::libc::c_void,
) -> () {
    let f = FUNCTIONS.jack_on_shutdown_impl;
    f(client, callback, arg)
}
pub unsafe fn jack_on_info_shutdown(
    client: *mut jack_client_t,
    callback: JackInfoShutdownCallback,
    arg: *mut ::libc::c_void,
) -> () {
    let f = FUNCTIONS.jack_on_info_shutdown_impl;
    f(client, callback, arg)
}
pub unsafe fn jack_set_process_callback(
    client: *mut jack_client_t,
    process_callback: JackProcessCallback,
    arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_process_callback_impl;
    f(client, process_callback, arg)
}
pub unsafe fn jack_set_freewheel_callback(
    client: *mut jack_client_t,
    freewheel_callback: JackFreewheelCallback,
    arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_freewheel_callback_impl;
    f(client, freewheel_callback, arg)
}
pub unsafe fn jack_set_buffer_size_callback(
    client: *mut jack_client_t,
    bufsize_callback: JackBufferSizeCallback,
    arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_buffer_size_callback_impl;
    f(client, bufsize_callback, arg)
}
pub unsafe fn jack_get_sample_rate(client: *mut jack_client_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_get_sample_rate_impl;
    f(client)
}
pub unsafe fn jack_set_sample_rate_callback(
    client: *mut jack_client_t,
    srate_callback: JackSampleRateCallback,
    arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_sample_rate_callback_impl;
    f(client, srate_callback, arg)
}
pub unsafe fn jack_set_client_registration_callback(
    client: *mut jack_client_t,
    registration_callback: JackClientRegistrationCallback,
    arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_client_registration_callback_impl;
    f(client, registration_callback, arg)
}
pub unsafe fn jack_set_port_registration_callback(
    client: *mut jack_client_t,
    registration_callback: JackPortRegistrationCallback,
    arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_port_registration_callback_impl;
    f(client, registration_callback, arg)
}
pub unsafe fn jack_set_port_connect_callback(
    client: *mut jack_client_t,
    connect_callback: JackPortConnectCallback,
    arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_port_connect_callback_impl;
    f(client, connect_callback, arg)
}
pub unsafe fn jack_set_port_rename_callback(
    client: *mut jack_client_t,
    rename_callback: JackPortRenameCallback,
    arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_port_rename_callback_impl;
    f(client, rename_callback, arg)
}
pub unsafe fn jack_set_graph_order_callback(
    client: *mut jack_client_t,
    graph_callback: JackGraphOrderCallback,
    arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_graph_order_callback_impl;
    f(client, graph_callback, arg)
}
pub unsafe fn jack_set_xrun_callback(
    client: *mut jack_client_t,
    xrun_callback: JackXRunCallback,
    arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_xrun_callback_impl;
    f(client, xrun_callback, arg)
}
pub unsafe fn jack_set_latency_callback(
    client: *mut jack_client_t,
    latency_callback: JackLatencyCallback,
    arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_latency_callback_impl;
    f(client, latency_callback, arg)
}
pub unsafe fn jack_set_freewheel(
    client: *mut jack_client_t,
    onoff: ::libc::c_int,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_freewheel_impl;
    f(client, onoff)
}
pub unsafe fn jack_set_buffer_size(
    client: *mut jack_client_t,
    nframes: jack_nframes_t,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_buffer_size_impl;
    f(client, nframes)
}
pub unsafe fn jack_get_buffer_size(client: *mut jack_client_t) -> jack_nframes_t {
    let f = FUNCTIONS.jack_get_buffer_size_impl;
    f(client)
}
pub unsafe fn jack_engine_takeover_timebase(client: *mut jack_client_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_engine_takeover_timebase_impl;
    f(client)
}
pub unsafe fn jack_cpu_load(client: *mut jack_client_t) -> ::libc::c_float {
    let f = FUNCTIONS.jack_cpu_load_impl;
    f(client)
}
pub unsafe fn jack_port_register(
    client: *mut jack_client_t,
    port_name: *const ::libc::c_char,
    port_type: *const ::libc::c_char,
    flags: ::libc::c_ulong,
    buffer_size: ::libc::c_ulong,
) -> *mut jack_port_t {
    let f = FUNCTIONS.jack_port_register_impl;
    f(client, port_name, port_type, flags, buffer_size)
}
pub unsafe fn jack_port_unregister(
    client: *mut jack_client_t,
    port: *mut jack_port_t,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_unregister_impl;
    f(client, port)
}
pub unsafe fn jack_port_get_buffer(
    port: *mut jack_port_t,
    nframes: jack_nframes_t,
) -> *mut ::libc::c_void {
    let f = FUNCTIONS.jack_port_get_buffer_impl;
    f(port, nframes)
}
pub unsafe fn jack_port_uuid(port: *mut jack_port_t) -> jack_uuid_t {
    let f = FUNCTIONS.jack_port_uuid_impl;
    f(port)
}
pub unsafe fn jack_port_name(port: *mut jack_port_t) -> *const ::libc::c_char {
    let f = FUNCTIONS.jack_port_name_impl;
    f(port)
}
pub unsafe fn jack_port_short_name(port: *mut jack_port_t) -> *const ::libc::c_char {
    let f = FUNCTIONS.jack_port_short_name_impl;
    f(port)
}
pub unsafe fn jack_port_flags(port: *mut jack_port_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_flags_impl;
    f(port)
}
pub unsafe fn jack_port_type(port: *const jack_port_t) -> *const ::libc::c_char {
    let f = FUNCTIONS.jack_port_type_impl;
    f(port)
}
pub unsafe fn jack_port_type_id(port: *const jack_port_t) -> Option<jack_port_type_id_t> {
    let f = FUNCTIONS.jack_port_type_id_impl?;
    Some(f(port))
}
pub unsafe fn jack_port_is_mine(
    client: *const jack_client_t,
    port: *const jack_port_t,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_is_mine_impl;
    f(client, port)
}
pub unsafe fn jack_port_connected(port: *const jack_port_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_connected_impl;
    f(port)
}
pub unsafe fn jack_port_connected_to(
    port: *const jack_port_t,
    port_name: *const ::libc::c_char,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_connected_to_impl;
    f(port, port_name)
}
pub unsafe fn jack_port_get_connections(port: *const jack_port_t) -> *mut *const ::libc::c_char {
    let f = FUNCTIONS.jack_port_get_connections_impl;
    f(port)
}
pub unsafe fn jack_port_get_all_connections(
    client: *const jack_client_t,
    port: *const jack_port_t,
) -> *mut *const ::libc::c_char {
    let f = FUNCTIONS.jack_port_get_all_connections_impl;
    f(client, port)
}
pub unsafe fn jack_port_tie(src: *mut jack_port_t, dst: *mut jack_port_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_tie_impl;
    f(src, dst)
}
pub unsafe fn jack_port_untie(port: *mut jack_port_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_untie_impl;
    f(port)
}
pub unsafe fn jack_port_set_name(
    port: *mut jack_port_t,
    port_name: *const ::libc::c_char,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_set_name_impl;
    f(port, port_name)
}
pub unsafe fn jack_port_set_alias(
    port: *mut jack_port_t,
    alias: *const ::libc::c_char,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_set_alias_impl;
    f(port, alias)
}
pub unsafe fn jack_port_unset_alias(
    port: *mut jack_port_t,
    alias: *const ::libc::c_char,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_unset_alias_impl;
    f(port, alias)
}
pub unsafe fn jack_port_get_aliases(
    port: *const jack_port_t,
    aliases: *mut *mut ::libc::c_char,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_get_aliases_impl;
    f(port, aliases)
}
pub unsafe fn jack_port_request_monitor(
    port: *mut jack_port_t,
    onoff: ::libc::c_int,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_request_monitor_impl;
    f(port, onoff)
}
pub unsafe fn jack_port_request_monitor_by_name(
    client: *mut jack_client_t,
    port_name: *const ::libc::c_char,
    onoff: ::libc::c_int,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_request_monitor_by_name_impl;
    f(client, port_name, onoff)
}
pub unsafe fn jack_port_ensure_monitor(
    port: *mut jack_port_t,
    onoff: ::libc::c_int,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_ensure_monitor_impl;
    f(port, onoff)
}
pub unsafe fn jack_port_monitoring_input(port: *mut jack_port_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_monitoring_input_impl;
    f(port)
}
pub unsafe fn jack_connect(
    client: *mut jack_client_t,
    source_port: *const ::libc::c_char,
    destination_port: *const ::libc::c_char,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_connect_impl;
    f(client, source_port, destination_port)
}
pub unsafe fn jack_disconnect(
    client: *mut jack_client_t,
    source_port: *const ::libc::c_char,
    destination_port: *const ::libc::c_char,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_disconnect_impl;
    f(client, source_port, destination_port)
}
pub unsafe fn jack_port_disconnect(
    client: *mut jack_client_t,
    port: *mut jack_port_t,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_disconnect_impl;
    f(client, port)
}
pub unsafe fn jack_port_name_size() -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_name_size_impl;
    f()
}
pub unsafe fn jack_port_type_size() -> ::libc::c_int {
    let f = FUNCTIONS.jack_port_type_size_impl;
    f()
}
pub unsafe fn jack_port_type_get_buffer_size(
    client: *mut jack_client_t,
    port_type: *const ::libc::c_char,
) -> ::libc::size_t {
    let f = FUNCTIONS.jack_port_type_get_buffer_size_impl;
    f(client, port_type)
}
pub unsafe fn jack_port_set_latency(port: *mut jack_port_t, arg1: jack_nframes_t) -> () {
    let f = FUNCTIONS.jack_port_set_latency_impl;
    f(port, arg1)
}
pub unsafe fn jack_port_get_latency_range(
    port: *mut jack_port_t,
    mode: jack_latency_callback_mode_t,
    range: *mut jack_latency_range_t,
) -> () {
    let f = FUNCTIONS.jack_port_get_latency_range_impl;
    f(port, mode, range)
}
pub unsafe fn jack_port_set_latency_range(
    port: *mut jack_port_t,
    mode: jack_latency_callback_mode_t,
    range: *mut jack_latency_range_t,
) -> () {
    let f = FUNCTIONS.jack_port_set_latency_range_impl;
    f(port, mode, range)
}
pub unsafe fn jack_recompute_total_latencies(client: *mut jack_client_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_recompute_total_latencies_impl;
    f(client)
}
pub unsafe fn jack_port_get_latency(port: *mut jack_port_t) -> jack_nframes_t {
    let f = FUNCTIONS.jack_port_get_latency_impl;
    f(port)
}
pub unsafe fn jack_port_get_total_latency(
    client: *mut jack_client_t,
    port: *mut jack_port_t,
) -> jack_nframes_t {
    let f = FUNCTIONS.jack_port_get_total_latency_impl;
    f(client, port)
}
pub unsafe fn jack_recompute_total_latency(
    arg1: *mut jack_client_t,
    port: *mut jack_port_t,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_recompute_total_latency_impl;
    f(arg1, port)
}
pub unsafe fn jack_get_ports(
    client: *mut jack_client_t,
    port_name_pattern: *const ::libc::c_char,
    type_name_pattern: *const ::libc::c_char,
    flags: ::libc::c_ulong,
) -> *mut *const ::libc::c_char {
    let f = FUNCTIONS.jack_get_ports_impl;
    f(client, port_name_pattern, type_name_pattern, flags)
}
pub unsafe fn jack_port_by_name(
    client: *mut jack_client_t,
    port_name: *const ::libc::c_char,
) -> *mut jack_port_t {
    let f = FUNCTIONS.jack_port_by_name_impl;
    f(client, port_name)
}
pub unsafe fn jack_port_by_id(
    client: *mut jack_client_t,
    port_id: jack_port_id_t,
) -> *mut jack_port_t {
    let f = FUNCTIONS.jack_port_by_id_impl;
    f(client, port_id)
}
pub unsafe fn jack_frames_since_cycle_start(arg1: *const jack_client_t) -> jack_nframes_t {
    let f = FUNCTIONS.jack_frames_since_cycle_start_impl;
    f(arg1)
}
pub unsafe fn jack_frame_time(arg1: *const jack_client_t) -> jack_nframes_t {
    let f = FUNCTIONS.jack_frame_time_impl;
    f(arg1)
}
pub unsafe fn jack_last_frame_time(client: *const jack_client_t) -> jack_nframes_t {
    let f = FUNCTIONS.jack_last_frame_time_impl;
    f(client)
}
pub unsafe fn jack_frames_to_time(
    client: *const jack_client_t,
    arg1: jack_nframes_t,
) -> jack_time_t {
    let f = FUNCTIONS.jack_frames_to_time_impl;
    f(client, arg1)
}
pub unsafe fn jack_time_to_frames(
    client: *const jack_client_t,
    arg1: jack_time_t,
) -> jack_nframes_t {
    let f = FUNCTIONS.jack_time_to_frames_impl;
    f(client, arg1)
}
pub unsafe fn jack_get_time() -> jack_time_t {
    let f = FUNCTIONS.jack_get_time_impl;
    f()
}
pub unsafe fn jack_set_error_function(
    func: ::std::option::Option<unsafe extern "C" fn(arg1: *const ::libc::c_char) -> ()>,
) -> () {
    let f = FUNCTIONS.jack_set_error_function_impl;
    f(func)
}
pub unsafe fn jack_set_info_function(
    func: ::std::option::Option<unsafe extern "C" fn(arg1: *const ::libc::c_char) -> ()>,
) -> () {
    let f = FUNCTIONS.jack_set_info_function_impl;
    f(func)
}
pub unsafe fn jack_free(ptr: *mut ::libc::c_void) -> () {
    let f = FUNCTIONS.jack_free_impl;
    f(ptr)
}
pub unsafe fn jack_client_real_time_priority(arg1: *mut jack_client_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_client_real_time_priority_impl;
    f(arg1)
}
pub unsafe fn jack_client_max_real_time_priority(arg1: *mut jack_client_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_client_max_real_time_priority_impl;
    f(arg1)
}
pub unsafe fn jack_set_session_callback(
    client: *mut jack_client_t,
    session_callback: JackSessionCallback,
    arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_session_callback_impl;
    f(client, session_callback, arg)
}
pub unsafe fn jack_session_reply(
    client: *mut jack_client_t,
    event: *mut jack_session_event_t,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_session_reply_impl;
    f(client, event)
}
pub unsafe fn jack_session_event_free(event: *mut jack_session_event_t) -> () {
    let f = FUNCTIONS.jack_session_event_free_impl;
    f(event)
}
pub unsafe fn jack_client_get_uuid(client: *mut jack_client_t) -> *mut ::libc::c_char {
    let f = FUNCTIONS.jack_client_get_uuid_impl;
    f(client)
}
pub unsafe fn jack_session_notify(
    client: *mut jack_client_t,
    target: *const ::libc::c_char,
    _type: jack_session_event_type_t,
    path: *const ::libc::c_char,
) -> *mut jack_session_command_t {
    let f = FUNCTIONS.jack_session_notify_impl;
    f(client, target, _type, path)
}
pub unsafe fn jack_session_commands_free(cmds: *mut jack_session_command_t) -> () {
    let f = FUNCTIONS.jack_session_commands_free_impl;
    f(cmds)
}
pub unsafe fn jack_reserve_client_name(
    client: *mut jack_client_t,
    name: *const ::libc::c_char,
    uuid: *const ::libc::c_char,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_reserve_client_name_impl;
    f(client, name, uuid)
}
pub unsafe fn jack_client_has_session_callback(
    client: *mut jack_client_t,
    client_name: *const ::libc::c_char,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_client_has_session_callback_impl;
    f(client, client_name)
}
pub unsafe fn jackctl_setup_signals(flags: ::libc::c_uint) -> Option<*mut jackctl_sigmask_t> {
    let f = FUNCTIONS.jackctl_setup_signals_impl?;
    Some(f(flags))
}
pub unsafe fn jackctl_wait_signals(signals: *mut jackctl_sigmask_t) -> Option<()> {
    let f = FUNCTIONS.jackctl_wait_signals_impl?;
    Some(f(signals))
}
pub unsafe fn jackctl_server_create(
    on_device_acquire: ::std::option::Option<
        unsafe extern "C" fn(device_name: *const ::libc::c_char) -> u8,
    >,
    on_device_release: ::std::option::Option<
        unsafe extern "C" fn(device_name: *const ::libc::c_char) -> (),
    >,
) -> Option<*mut jackctl_server_t> {
    let f = FUNCTIONS.jackctl_server_create_impl?;
    Some(f(on_device_acquire, on_device_release))
}
pub unsafe fn jackctl_server_destroy(server: *mut jackctl_server_t) -> Option<()> {
    let f = FUNCTIONS.jackctl_server_destroy_impl?;
    Some(f(server))
}
pub unsafe fn jackctl_server_open(
    server: *mut jackctl_server_t,
    driver: *mut jackctl_driver_t,
) -> Option<u8> {
    let f = FUNCTIONS.jackctl_server_open_impl?;
    Some(f(server, driver))
}
pub unsafe fn jackctl_server_start(server: *mut jackctl_server_t) -> Option<u8> {
    let f = FUNCTIONS.jackctl_server_start_impl?;
    Some(f(server))
}
pub unsafe fn jackctl_server_stop(server: *mut jackctl_server_t) -> Option<u8> {
    let f = FUNCTIONS.jackctl_server_stop_impl?;
    Some(f(server))
}
pub unsafe fn jackctl_server_close(server: *mut jackctl_server_t) -> Option<u8> {
    let f = FUNCTIONS.jackctl_server_close_impl?;
    Some(f(server))
}
pub unsafe fn jackctl_server_get_drivers_list(
    server: *mut jackctl_server_t,
) -> Option<*const JSList> {
    let f = FUNCTIONS.jackctl_server_get_drivers_list_impl?;
    Some(f(server))
}
pub unsafe fn jackctl_server_get_parameters(
    server: *mut jackctl_server_t,
) -> Option<*const JSList> {
    let f = FUNCTIONS.jackctl_server_get_parameters_impl?;
    Some(f(server))
}
pub unsafe fn jackctl_server_get_internals_list(
    server: *mut jackctl_server_t,
) -> Option<*const JSList> {
    let f = FUNCTIONS.jackctl_server_get_internals_list_impl?;
    Some(f(server))
}
pub unsafe fn jackctl_server_load_internal(
    server: *mut jackctl_server_t,
    internal: *mut jackctl_internal_t,
) -> Option<u8> {
    let f = FUNCTIONS.jackctl_server_load_internal_impl?;
    Some(f(server, internal))
}
pub unsafe fn jackctl_server_unload_internal(
    server: *mut jackctl_server_t,
    internal: *mut jackctl_internal_t,
) -> Option<u8> {
    let f = FUNCTIONS.jackctl_server_unload_internal_impl?;
    Some(f(server, internal))
}
pub unsafe fn jackctl_server_add_slave(
    server: *mut jackctl_server_t,
    driver: *mut jackctl_driver_t,
) -> Option<u8> {
    let f = FUNCTIONS.jackctl_server_add_slave_impl?;
    Some(f(server, driver))
}
pub unsafe fn jackctl_server_remove_slave(
    server: *mut jackctl_server_t,
    driver: *mut jackctl_driver_t,
) -> Option<u8> {
    let f = FUNCTIONS.jackctl_server_remove_slave_impl?;
    Some(f(server, driver))
}
pub unsafe fn jackctl_server_switch_master(
    server: *mut jackctl_server_t,
    driver: *mut jackctl_driver_t,
) -> Option<u8> {
    let f = FUNCTIONS.jackctl_server_switch_master_impl?;
    Some(f(server, driver))
}
pub unsafe fn jackctl_driver_get_name(
    driver: *mut jackctl_driver_t,
) -> Option<*const ::libc::c_char> {
    let f = FUNCTIONS.jackctl_driver_get_name_impl?;
    Some(f(driver))
}
pub unsafe fn jackctl_driver_get_type(
    driver: *mut jackctl_driver_t,
) -> Option<jackctl_driver_type_t> {
    let f = FUNCTIONS.jackctl_driver_get_type_impl?;
    Some(f(driver))
}
pub unsafe fn jackctl_driver_get_parameters(
    driver: *mut jackctl_driver_t,
) -> Option<*const JSList> {
    let f = FUNCTIONS.jackctl_driver_get_parameters_impl?;
    Some(f(driver))
}
pub unsafe fn jackctl_driver_params_parse(
    driver: *mut jackctl_driver_t,
    argc: ::libc::c_int,
    argv: *mut *mut ::libc::c_char,
) -> Option<::libc::c_int> {
    let f = FUNCTIONS.jackctl_driver_params_parse_impl?;
    Some(f(driver, argc, argv))
}
pub unsafe fn jackctl_internal_get_name(
    internal: *mut jackctl_internal_t,
) -> Option<*const ::libc::c_char> {
    let f = FUNCTIONS.jackctl_internal_get_name_impl?;
    Some(f(internal))
}
pub unsafe fn jackctl_internal_get_parameters(
    internal: *mut jackctl_internal_t,
) -> Option<*const JSList> {
    let f = FUNCTIONS.jackctl_internal_get_parameters_impl?;
    Some(f(internal))
}
pub unsafe fn jackctl_parameter_get_name(
    parameter: *mut jackctl_parameter_t,
) -> Option<*const ::libc::c_char> {
    let f = FUNCTIONS.jackctl_parameter_get_name_impl?;
    Some(f(parameter))
}
pub unsafe fn jackctl_parameter_get_short_description(
    parameter: *mut jackctl_parameter_t,
) -> Option<*const ::libc::c_char> {
    let f = FUNCTIONS.jackctl_parameter_get_short_description_impl?;
    Some(f(parameter))
}
pub unsafe fn jackctl_parameter_get_long_description(
    parameter: *mut jackctl_parameter_t,
) -> Option<*const ::libc::c_char> {
    let f = FUNCTIONS.jackctl_parameter_get_long_description_impl?;
    Some(f(parameter))
}
pub unsafe fn jackctl_parameter_get_type(
    parameter: *mut jackctl_parameter_t,
) -> Option<jackctl_param_type_t> {
    let f = FUNCTIONS.jackctl_parameter_get_type_impl?;
    Some(f(parameter))
}
pub unsafe fn jackctl_parameter_get_id(
    parameter: *mut jackctl_parameter_t,
) -> Option<::libc::c_char> {
    let f = FUNCTIONS.jackctl_parameter_get_id_impl?;
    Some(f(parameter))
}
pub unsafe fn jackctl_parameter_is_set(parameter: *mut jackctl_parameter_t) -> Option<u8> {
    let f = FUNCTIONS.jackctl_parameter_is_set_impl?;
    Some(f(parameter))
}
pub unsafe fn jackctl_parameter_reset(parameter: *mut jackctl_parameter_t) -> Option<u8> {
    let f = FUNCTIONS.jackctl_parameter_reset_impl?;
    Some(f(parameter))
}
pub unsafe fn jackctl_parameter_get_value(
    parameter: *mut jackctl_parameter_t,
) -> Option<Union_jackctl_parameter_value> {
    let f = FUNCTIONS.jackctl_parameter_get_value_impl?;
    Some(f(parameter))
}
pub unsafe fn jackctl_parameter_set_value(
    parameter: *mut jackctl_parameter_t,
    value_ptr: *const Union_jackctl_parameter_value,
) -> Option<u8> {
    let f = FUNCTIONS.jackctl_parameter_set_value_impl?;
    Some(f(parameter, value_ptr))
}
pub unsafe fn jackctl_parameter_get_default_value(
    parameter: *mut jackctl_parameter_t,
) -> Option<Union_jackctl_parameter_value> {
    let f = FUNCTIONS.jackctl_parameter_get_default_value_impl?;
    Some(f(parameter))
}
pub unsafe fn jackctl_parameter_has_range_constraint(
    parameter: *mut jackctl_parameter_t,
) -> Option<u8> {
    let f = FUNCTIONS.jackctl_parameter_has_range_constraint_impl?;
    Some(f(parameter))
}
pub unsafe fn jackctl_parameter_has_enum_constraint(
    parameter: *mut jackctl_parameter_t,
) -> Option<u8> {
    let f = FUNCTIONS.jackctl_parameter_has_enum_constraint_impl?;
    Some(f(parameter))
}
pub unsafe fn jackctl_parameter_get_enum_constraints_count(
    parameter: *mut jackctl_parameter_t,
) -> Option<u32> {
    let f = FUNCTIONS.jackctl_parameter_get_enum_constraints_count_impl?;
    Some(f(parameter))
}
pub unsafe fn jackctl_parameter_get_enum_constraint_value(
    parameter: *mut jackctl_parameter_t,
    index: u32,
) -> Option<Union_jackctl_parameter_value> {
    let f = FUNCTIONS.jackctl_parameter_get_enum_constraint_value_impl?;
    Some(f(parameter, index))
}
pub unsafe fn jackctl_parameter_get_enum_constraint_description(
    parameter: *mut jackctl_parameter_t,
    index: u32,
) -> Option<*const ::libc::c_char> {
    let f = FUNCTIONS.jackctl_parameter_get_enum_constraint_description_impl?;
    Some(f(parameter, index))
}
pub unsafe fn jackctl_parameter_get_range_constraint(
    parameter: *mut jackctl_parameter_t,
    min_ptr: *mut Union_jackctl_parameter_value,
    max_ptr: *mut Union_jackctl_parameter_value,
) -> Option<()> {
    let f = FUNCTIONS.jackctl_parameter_get_range_constraint_impl?;
    Some(f(parameter, min_ptr, max_ptr))
}
pub unsafe fn jackctl_parameter_constraint_is_strict(
    parameter: *mut jackctl_parameter_t,
) -> Option<u8> {
    let f = FUNCTIONS.jackctl_parameter_constraint_is_strict_impl?;
    Some(f(parameter))
}
pub unsafe fn jackctl_parameter_constraint_is_fake_value(
    parameter: *mut jackctl_parameter_t,
) -> Option<u8> {
    let f = FUNCTIONS.jackctl_parameter_constraint_is_fake_value_impl?;
    Some(f(parameter))
}
pub unsafe fn jack_set_property(
    arg1: *mut jack_client_t,
    subject: jack_uuid_t,
    key: *const ::libc::c_char,
    value: *const ::libc::c_char,
    _type: *const ::libc::c_char,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_property_impl;
    f(arg1, subject, key, value, _type)
}
pub unsafe fn jack_get_property(
    subject: jack_uuid_t,
    key: *const ::libc::c_char,
    value: *mut *mut ::libc::c_char,
    _type: *mut *mut ::libc::c_char,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_get_property_impl;
    f(subject, key, value, _type)
}
pub unsafe fn jack_free_description(
    desc: *mut jack_description_t,
    free_description_itself: ::libc::c_int,
) -> () {
    let f = FUNCTIONS.jack_free_description_impl;
    f(desc, free_description_itself)
}
pub unsafe fn jack_get_properties(
    subject: jack_uuid_t,
    desc: *mut jack_description_t,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_get_properties_impl;
    f(subject, desc)
}
pub unsafe fn jack_get_all_properties(descs: *mut *mut jack_description_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_get_all_properties_impl;
    f(descs)
}
pub unsafe fn jack_remove_property(
    client: *mut jack_client_t,
    subject: jack_uuid_t,
    key: *const ::libc::c_char,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_remove_property_impl;
    f(client, subject, key)
}
pub unsafe fn jack_remove_properties(
    client: *mut jack_client_t,
    subject: jack_uuid_t,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_remove_properties_impl;
    f(client, subject)
}
pub unsafe fn jack_remove_all_properties(client: *mut jack_client_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_remove_all_properties_impl;
    f(client)
}
pub unsafe fn jack_set_property_change_callback(
    client: *mut jack_client_t,
    callback: JackPropertyChangeCallback,
    arg: *mut ::libc::c_void,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_set_property_change_callback_impl;
    f(client, callback, arg)
}
pub unsafe fn jack_get_internal_client_name(
    client: *mut jack_client_t,
    intclient: jack_intclient_t,
) -> Option<*mut ::libc::c_char> {
    let f = FUNCTIONS.jack_get_internal_client_name_impl?;
    Some(f(client, intclient))
}
pub unsafe fn jack_internal_client_handle(
    client: *mut jack_client_t,
    client_name: *const ::libc::c_char,
    status: *mut jack_status_t,
) -> Option<jack_intclient_t> {
    let f = FUNCTIONS.jack_internal_client_handle_impl?;
    Some(f(client, client_name, status))
}
pub unsafe fn jack_internal_client_load(
    client: *mut jack_client_t,
    client_name: *const ::libc::c_char,
    options: jack_options_t,
    status: *mut jack_status_t,
    load_name: *const ::libc::c_char,
    load_init: *const ::libc::c_char,
) -> Option<jack_intclient_t> {
    let f = FUNCTIONS.jack_internal_client_load_impl?;
    Some(f(
        client,
        client_name,
        options,
        status,
        load_name,
        load_init,
    ))
}
pub unsafe fn jack_internal_client_unload(
    client: *mut jack_client_t,
    intclient: jack_intclient_t,
) -> Option<jack_status_t> {
    let f = FUNCTIONS.jack_internal_client_unload_impl?;
    Some(f(client, intclient))
}
pub unsafe fn jack_get_max_delayed_usecs(client: *mut jack_client_t) -> ::libc::c_float {
    let f = FUNCTIONS.jack_get_max_delayed_usecs_impl;
    f(client)
}
pub unsafe fn jack_get_xrun_delayed_usecs(client: *mut jack_client_t) -> ::libc::c_float {
    let f = FUNCTIONS.jack_get_xrun_delayed_usecs_impl;
    f(client)
}
pub unsafe fn jack_reset_max_delayed_usecs(client: *mut jack_client_t) -> () {
    let f = FUNCTIONS.jack_reset_max_delayed_usecs_impl;
    f(client)
}
pub unsafe fn jack_midi_get_event_count(port_buffer: *mut ::libc::c_void) -> u32 {
    let f = FUNCTIONS.jack_midi_get_event_count_impl;
    f(port_buffer)
}
pub unsafe fn jack_midi_event_get(
    event: *mut jack_midi_event_t,
    port_buffer: *mut ::libc::c_void,
    event_index: u32,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_midi_event_get_impl;
    f(event, port_buffer, event_index)
}
pub unsafe fn jack_midi_clear_buffer(port_buffer: *mut ::libc::c_void) -> () {
    let f = FUNCTIONS.jack_midi_clear_buffer_impl;
    f(port_buffer)
}
pub unsafe fn jack_midi_max_event_size(port_buffer: *mut ::libc::c_void) -> ::libc::size_t {
    let f = FUNCTIONS.jack_midi_max_event_size_impl;
    f(port_buffer)
}
pub unsafe fn jack_midi_event_reserve(
    port_buffer: *mut ::libc::c_void,
    time: jack_nframes_t,
    data_size: ::libc::size_t,
) -> *mut jack_midi_data_t {
    let f = FUNCTIONS.jack_midi_event_reserve_impl;
    f(port_buffer, time, data_size)
}
pub unsafe fn jack_midi_event_write(
    port_buffer: *mut ::libc::c_void,
    time: jack_nframes_t,
    data: *const jack_midi_data_t,
    data_size: ::libc::size_t,
) -> ::libc::c_int {
    let f = FUNCTIONS.jack_midi_event_write_impl;
    f(port_buffer, time, data, data_size)
}
pub unsafe fn jack_midi_get_lost_event_count(port_buffer: *mut ::libc::c_void) -> u32 {
    let f = FUNCTIONS.jack_midi_get_lost_event_count_impl;
    f(port_buffer)
}
pub unsafe fn jack_ringbuffer_create(sz: ::libc::size_t) -> *mut jack_ringbuffer_t {
    let f = FUNCTIONS.jack_ringbuffer_create_impl;
    f(sz)
}
pub unsafe fn jack_ringbuffer_free(rb: *mut jack_ringbuffer_t) -> () {
    let f = FUNCTIONS.jack_ringbuffer_free_impl;
    f(rb)
}
pub unsafe fn jack_ringbuffer_get_read_vector(
    rb: *const jack_ringbuffer_t,
    vec: *mut jack_ringbuffer_data_t,
) -> () {
    let f = FUNCTIONS.jack_ringbuffer_get_read_vector_impl;
    f(rb, vec)
}
pub unsafe fn jack_ringbuffer_get_write_vector(
    rb: *const jack_ringbuffer_t,
    vec: *mut jack_ringbuffer_data_t,
) -> () {
    let f = FUNCTIONS.jack_ringbuffer_get_write_vector_impl;
    f(rb, vec)
}
pub unsafe fn jack_ringbuffer_read(
    rb: *mut jack_ringbuffer_t,
    dest: *mut ::libc::c_char,
    cnt: ::libc::size_t,
) -> ::libc::size_t {
    let f = FUNCTIONS.jack_ringbuffer_read_impl;
    f(rb, dest, cnt)
}
pub unsafe fn jack_ringbuffer_peek(
    rb: *mut jack_ringbuffer_t,
    dest: *mut ::libc::c_char,
    cnt: ::libc::size_t,
) -> ::libc::size_t {
    let f = FUNCTIONS.jack_ringbuffer_peek_impl;
    f(rb, dest, cnt)
}
pub unsafe fn jack_ringbuffer_read_advance(rb: *mut jack_ringbuffer_t, cnt: ::libc::size_t) -> () {
    let f = FUNCTIONS.jack_ringbuffer_read_advance_impl;
    f(rb, cnt)
}
pub unsafe fn jack_ringbuffer_read_space(rb: *const jack_ringbuffer_t) -> ::libc::size_t {
    let f = FUNCTIONS.jack_ringbuffer_read_space_impl;
    f(rb)
}
pub unsafe fn jack_ringbuffer_mlock(rb: *mut jack_ringbuffer_t) -> ::libc::c_int {
    let f = FUNCTIONS.jack_ringbuffer_mlock_impl;
    f(rb)
}
pub unsafe fn jack_ringbuffer_reset(rb: *mut jack_ringbuffer_t) -> () {
    let f = FUNCTIONS.jack_ringbuffer_reset_impl;
    f(rb)
}
pub unsafe fn jack_ringbuffer_write(
    rb: *mut jack_ringbuffer_t,
    src: *const ::libc::c_char,
    cnt: ::libc::size_t,
) -> ::libc::size_t {
    let f = FUNCTIONS.jack_ringbuffer_write_impl;
    f(rb, src, cnt)
}
pub unsafe fn jack_ringbuffer_write_advance(rb: *mut jack_ringbuffer_t, cnt: ::libc::size_t) -> () {
    let f = FUNCTIONS.jack_ringbuffer_write_advance_impl;
    f(rb, cnt)
}
pub unsafe fn jack_ringbuffer_write_space(rb: *const jack_ringbuffer_t) -> ::libc::size_t {
    let f = FUNCTIONS.jack_ringbuffer_write_space_impl;
    f(rb)
}
pub unsafe fn jack_uuid_to_index(arg1: jack_uuid_t) -> u32 {
    let f = FUNCTIONS.jack_uuid_to_index_impl;
    f(arg1)
}
pub unsafe fn jack_uuid_compare(arg1: jack_uuid_t, arg2: jack_uuid_t) -> ::std::os::raw::c_int {
    let f = FUNCTIONS.jack_uuid_compare_impl;
    f(arg1, arg2)
}
pub unsafe fn jack_uuid_copy(dst: *mut jack_uuid_t, src: jack_uuid_t) -> () {
    let f = FUNCTIONS.jack_uuid_copy_impl;
    f(dst, src)
}
pub unsafe fn jack_uuid_clear(arg1: *mut jack_uuid_t) -> () {
    let f = FUNCTIONS.jack_uuid_clear_impl;
    f(arg1)
}
pub unsafe fn jack_uuid_parse(
    buf: *const ::std::os::raw::c_char,
    arg1: *mut jack_uuid_t,
) -> ::std::os::raw::c_int {
    let f = FUNCTIONS.jack_uuid_parse_impl;
    f(buf, arg1)
}
pub unsafe fn jack_uuid_unparse(arg1: jack_uuid_t, buf: *mut ::std::os::raw::c_char) -> () {
    let f = FUNCTIONS.jack_uuid_unparse_impl;
    f(arg1, buf)
}
pub unsafe fn jack_uuid_empty(arg1: jack_uuid_t) -> ::std::os::raw::c_int {
    let f = FUNCTIONS.jack_uuid_empty_impl;
    f(arg1)
}
