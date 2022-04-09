use bitflags::bitflags;

use std::io::Write;

bitflags! {
    struct FunctionFlags: u8 {
        const NONE = 0b00000000;
        const WEAK = 0b00000001;
    }
}

fn main() {
    println!("cargo:rerun-if-env-changed=RUST_JACK_DLOPEN");
    let dlopen = std::env::var("RUST_JACK_DLOPEN").is_ok();
    if dlopen {
        println!("cargo:rustc-cfg=feature=\"dlopen\"");
    }
    if !(dlopen || cfg!(feature = "dlopen")) {
        // pkg-config is required to find PipeWire's implementation of libjack
        // Refer to https://github.com/RustAudio/rust-jack/issues/142 for details.
        // Do not unwrap this because linking might still work if pkg-config is
        // not installed, for example on Windows.
        pkg_config::find_library("jack").unwrap();
    }
    write_src("src/functions.rs", FUNCTIONS);
}

fn write_src(path: &str, fns: &[Function]) {
    let mut out = std::fs::File::create(path).unwrap();
    writeln!(out, "use crate::types::*;").unwrap();
    writeln!(out, "use lazy_static::lazy_static;").unwrap();
    writeln!(out, "pub struct JackFunctions {{").unwrap();
    writeln!(out, "    pub library: libloading::Library,").unwrap();
    for f in fns.iter() {
        if f.flags.contains(FunctionFlags::WEAK) {
            writeln!(out, 
                "    {}_impl: Option<unsafe extern \"C\" fn({}) -> {}>,",
                f.name,
                f.arg_types(),
                f.ret
            ).unwrap();
        } else {
            writeln!(out, 
                "    {}_impl: unsafe extern \"C\" fn({}) -> {},",
                f.name,
                f.arg_types(),
                f.ret
            ).unwrap();
        }
    }
    writeln!(out, "}}\n").unwrap();

    writeln!(out, "lazy_static! {{").unwrap();
    writeln!(out, "    static ref LIB: JackFunctions = unsafe {{").unwrap();
    writeln!(out, "        let library = libloading::Library::new(crate::consts::JACK_LIB).unwrap();").unwrap();
    for f in fns.iter() {
        if f.flags.contains(FunctionFlags::WEAK) {
            writeln!(out, 
                "        let {}_impl = library.get::<unsafe extern \"C\" fn({}) -> {}>(b\"{}\").ok();",
                f.name,
                f.args_full(),
                f.ret,
                f.name,
            ).unwrap();
            writeln!(out, 
                "        let {}_impl = {}_impl.map(|sym| sym.into_raw());",
                f.name, f.name
            ).unwrap();
            writeln!(out, 
                "        let {}_impl = {}_impl.map(|sym| *sym.deref() as {});",
                f.name,
                f.name,
                f.type_name()
            ).unwrap();
        } else {
            writeln!(out, 
                "        let {}_impl = library.get::<unsafe extern \"C\" fn({}) -> {}>(b\"{}\").unwrap();",
                f.name,
                f.args_full(),
                f.ret,
                f.name,
            ).unwrap();
            writeln!(out, "        let {}_impl = {}_impl.into_raw();", f.name, f.name).unwrap();
            writeln!(out, 
                "        let {}_impl = *{}_impl.deref() as {};",
                f.name,
                f.name,
                f.type_name()
            ).unwrap();
        }
    }
    writeln!(out, "        JackFunctions {{").unwrap();
    writeln!(out, "            library,").unwrap();
    for f in fns.iter() {
        writeln!(out, "            {}_impl,", f.name).unwrap();
    }
    writeln!(out, "        }}").unwrap();
    writeln!(out, "    }};\n").unwrap();
    writeln!(out, "}}\n").unwrap();

    for f in fns.iter() {
        if f.flags.contains(FunctionFlags::WEAK) {
            writeln!(out, 
                "pub unsafe fn {}({}) -> Option<{}> {{",
                f.name,
                f.args_full(),
                f.ret
            ).unwrap();
            writeln!(out, "    let f = LIB.{}_impl?;", f.name).unwrap();
            writeln!(out, "    Some(f({}))", f.arg_names()).unwrap();
        } else {
            writeln!(out, 
                "pub unsafe fn {}({}) -> {} {{",
                f.name,
                f.args_full(),
                f.ret
            ).unwrap();
            writeln!(out, "    let f = LIB.{}_impl;", f.name).unwrap();
            writeln!(out, "    f({})", f.arg_names()).unwrap();
        }
        writeln!(out, "}}").unwrap();
    }
}

struct Function {
    name: &'static str,
    args: &'static [(&'static str, &'static str)],
    ret: &'static str,
    flags: FunctionFlags,
}

impl Function {
    fn args_full(&self) -> String {
        let mut args = String::new();
        for &(name, ty) in self.args.iter() {
            if !args.is_empty() {
                args.push_str(", ");
            }
            if !name.is_empty() {
                args.push_str(name);
                args.push_str(": ");
            }
            args.push_str(ty);
        }
        args
    }

    fn arg_types(&self) -> String {
        let mut args = String::new();
        for &(_, ty) in self.args.iter() {
            if !args.is_empty() {
                args.push_str(", ");
            }
            args.push_str(ty);
        }
        args
    }

    fn arg_names(&self) -> String {
        let mut args = String::new();
        for &(name, _) in self.args.iter() {
            if !args.is_empty() {
                args.push_str(", ");
            }
            args.push_str(name);
        }
        args
    }

    fn type_name(&self) -> String {
        format!(
            "unsafe extern \"C\" fn({}) -> {}",
            self.args_full(),
            self.ret
        )
    }
}

const FUNCTIONS: &[Function] = &[
    Function {
        name: "jack_release_timebase",
        args: &[("client", "*mut jack_client_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_get_cycle_times",
        args: &[
            ("client", "*const jack_client_t"),
            ("current_frames", "*mut jack_nframes_t"),
            ("current_usecs", "*mut jack_time_t"),
            ("next_usecs", "*mut jack_time_t"),
            ("period_usecs", "*mut ::libc::c_float"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::WEAK,
    },
    Function {
        name: "jack_set_sync_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("sync_callback", "JackSyncCallback"),
            ("sync_arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_sync_timeout",
        args: &[("client", "*mut jack_client_t"), ("timeout", "jack_time_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_timebase_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("conditional", "::libc::c_int"),
            ("timebase_callback", "TimebaseCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_transport_locate",
        args: &[
            ("client", "*mut jack_client_t"),
            ("frame", "jack_nframes_t"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_transport_query",
        args: &[
            ("client", "*const jack_client_t"),
            ("pos", "*mut jack_position_t"),
        ],
        ret: "jack_transport_state_t",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_get_current_transport_frame",
        args: &[("client", "*const jack_client_t")],
        ret: "jack_nframes_t",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_transport_reposition",
        args: &[
            ("client", "*mut jack_client_t"),
            ("pos", "*const jack_position_t"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_transport_start",
        args: &[("client", "*mut jack_client_t")],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_transport_stop",
        args: &[("client", "*mut jack_client_t")],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_get_transport_info",
        args: &[
            ("client", "*mut jack_client_t"),
            ("tinfo", "*mut jack_transport_info_t"),
        ],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_transport_info",
        args: &[
            ("client", "*mut jack_client_t"),
            ("tinfo", "*mut jack_transport_info_t"),
        ],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_client_open",
        args: &[
            ("client_name", "*const ::libc::c_char"),
            ("options", "jack_options_t"),
            ("status", "*mut jack_status_t"),
            // Varargs not supported
            // ("", "..."),
        ],
        ret: "*mut jack_client_t",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_client_new",
        args: &[("client_name", "*const ::libc::c_char")],
        ret: "*mut jack_client_t",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_client_close",
        args: &[("client", "*mut jack_client_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_client_name_size",
        args: &[],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_get_client_name",
        args: &[("client", "*mut jack_client_t")],
        ret: "*mut ::libc::c_char",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_get_uuid_for_client_name",
        args: &[
            ("client", "*mut jack_client_t"),
            ("client_name", "*const ::libc::c_char"),
        ],
        ret: "*mut ::libc::c_char",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_get_client_name_by_uuid",
        args: &[
            ("client", "*mut jack_client_t"),
            ("client_uuid", "*const ::libc::c_char"),
        ],
        ret: "*mut ::libc::c_char",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_internal_client_new",
        args: &[
            ("client_name", "*const ::libc::c_char"),
            ("load_name", "*const ::libc::c_char"),
            ("load_init", "*const ::libc::c_char"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::WEAK,
    },
    Function {
        name: "jack_internal_client_close",
        args: &[("client_name", "*const ::libc::c_char")],
        ret: "()",
        flags: FunctionFlags::WEAK,
    },
    Function {
        name: "jack_activate",
        args: &[("client", "*mut jack_client_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_deactivate",
        args: &[("client", "*mut jack_client_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_get_client_pid",
        args: &[("name", "*const ::libc::c_char")],
        ret: "::libc::c_int",
        flags: FunctionFlags::WEAK,
    },
    // #[cfg(not(target_os = "windows"))]
    // pub fn jack_client_thread_id(client: *mut jack_client_t) -> jack_native_thread_t;
    Function {
        name: "jack_is_realtime",
        args: &[("client", "*mut jack_client_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_thread_wait",
        args: &[
            ("client", "*mut jack_client_t"),
            ("status", "::libc::c_int"),
        ],
        ret: "jack_nframes_t",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_cycle_wait",
        args: &[("client", "*mut jack_client_t")],
        ret: "jack_nframes_t",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_cycle_signal",
        args: &[
            ("client", "*mut jack_client_t"),
            ("status", "::libc::c_int"),
        ],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_process_thread",
        args: &[
            ("client", "*mut jack_client_t"),
            ("thread_callback", "JackThreadCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_thread_init_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("thread_init_callback", "JackThreadInitCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_on_shutdown",
        args: &[
            ("client", "*mut jack_client_t"),
            ("callback", "JackShutdownCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_on_info_shutdown",
        args: &[
            ("client", "*mut jack_client_t"),
            ("callback", "JackInfoShutdownCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_process_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("process_callback", "JackProcessCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_freewheel_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("freewheel_callback", "JackFreewheelCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_buffer_size_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("bufsize_callback", "JackBufferSizeCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_get_sample_rate",
        args: &[
            ("client", "*mut jack_client_t"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_sample_rate_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("srate_callback", "JackSampleRateCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_client_registration_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("registration_callback", "JackClientRegistrationCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_port_registration_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("registration_callback", "JackPortRegistrationCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_port_connect_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("connect_callback", "JackPortConnectCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_port_rename_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("rename_callback", "JackPortRenameCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_graph_order_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("graph_callback", "JackGraphOrderCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_xrun_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("xrun_callback", "JackXRunCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_latency_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("latency_callback", "JackLatencyCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_freewheel",
        args: &[("client", "*mut jack_client_t"), ("onoff", "::libc::c_int")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_set_buffer_size",
        args: &[
            ("client", "*mut jack_client_t"),
            ("nframes", "jack_nframes_t"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_get_buffer_size",
        args: &[("client", "*mut jack_client_t")],
        ret: "jack_nframes_t",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_engine_takeover_timebase",
        args: &[("client", "*mut jack_client_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_cpu_load",
        args: &[("client", "*mut jack_client_t")],
        ret: "::libc::c_float",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_register",
        args: &[
            ("client", "*mut jack_client_t"),
            ("port_name", "*const ::libc::c_char"),
            ("port_type", "*const ::libc::c_char"),
            ("flags", "::libc::c_ulong"),
            ("buffer_size", "::libc::c_ulong"),
        ],
        ret: "*mut jack_port_t",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_unregister",
        args: &[
            ("client", "*mut jack_client_t"),
            ("port", "*mut jack_port_t"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_get_buffer",
        args: &[("port", "*mut jack_port_t"), ("nframes", "jack_nframes_t")],
        ret: "*mut ::libc::c_void",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_uuid",
        args: &[("port", "*mut jack_port_t")],
        ret: "jack_uuid_t",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_name",
        args: &[("port", "*mut jack_port_t")],
        ret: "*const ::libc::c_char",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_short_name",
        args: &[("port", "*mut jack_port_t")],
        ret: "*const ::libc::c_char",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_flags",
        args: &[("port", "*mut jack_port_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_type",
        args: &[("port", "*const jack_port_t")],
        ret: "*const ::libc::c_char",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_type_id",
        args: &[("port", "*const jack_port_t")],
        ret: "jack_port_type_id_t",
        flags: FunctionFlags::WEAK,
    },
    Function {
        name: "jack_port_is_mine",
        args: &[
            ("client", "*const jack_client_t"),
            ("port", "*const jack_port_t"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_connected",
        args: &[("port", "*const jack_port_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_connected_to",
        args: &[
            ("port", "*const jack_port_t"),
            ("port_name", "*const ::libc::c_char"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_get_connections",
        args: &[("port", "*const jack_port_t")],
        ret: "*mut *const ::libc::c_char",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_get_all_connections",
        args: &[
            ("client", "*const jack_client_t"),
            ("port", "*const jack_port_t"),
        ],
        ret: "*mut *const ::libc::c_char",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_tie",
        args: &[("src", "*mut jack_port_t"), ("dst", "*mut jack_port_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_untie",
        args: &[("port", "*mut jack_port_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_set_name",
        args: &[
            ("port", "*mut jack_port_t"),
            ("port_name", "*const ::libc::c_char"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_set_alias",
        args: &[
            ("port", "*mut jack_port_t"),
            ("alias", "*const ::libc::c_char"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_unset_alias",
        args: &[
            ("port", "*mut jack_port_t"),
            ("alias", "*const ::libc::c_char"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_get_aliases",
        args: &[
            ("port", "*const jack_port_t"),
            ("aliases", "*mut *mut ::libc::c_char"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_request_monitor",
        args: &[("port", "*mut jack_port_t"), ("onoff", "::libc::c_int")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_request_monitor_by_name",
        args: &[
            ("client", "*mut jack_client_t"),
            ("port_name", "*const ::libc::c_char"),
            ("onoff", "::libc::c_int"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_ensure_monitor",
        args: &[("port", "*mut jack_port_t"), ("onoff", "::libc::c_int")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_port_monitoring_input",
        args: &[("port", "*mut jack_port_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_connect",
        args: &[
            ("client", "*mut jack_client_t"),
            ("source_port", "*const ::libc::c_char"),
            ("destination_port", "*const ::libc::c_char"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_disconnect(
    //     client: *mut jack_client_t,
    //     source_port: *const ::libc::c_char,
    //     destination_port: *const ::libc::c_char,
    // ) -> ::libc::c_int;
    Function {
        name: "jack_disconnect",
        args: &[
            ("client", "*mut jack_client_t"),
            ("source_port", "*const ::libc::c_char"),
            ("destination_port", "*const ::libc::c_char"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_port_disconnect(client: *mut jack_client_t, port: *mut jack_port_t) -> ::libc::c_int;
    Function {
        name: "jack_port_disconnect",
        args: &[
            ("client", "*mut jack_client_t"),
            ("port", "*mut jack_port_t"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_port_name_size() -> ::libc::c_int;
    Function {
        name: "jack_port_name_size",
        args: &[],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_port_type_size() -> ::libc::c_int;
    Function {
        name: "jack_port_type_size",
        args: &[],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_port_type_get_buffer_size(
    //     client: *mut jack_client_t,
    //     port_type: *const ::libc::c_char,
    // ) -> ::libc::size_t;
    Function {
        name: "jack_port_type_get_buffer_size",
        args: &[
            ("client", "*mut jack_client_t"),
            ("port_type", "*const ::libc::c_char"),
        ],
        ret: "::libc::size_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_port_set_latency(port: *mut jack_port_t, arg1: jack_nframes_t) -> ();
    Function {
        name: "jack_port_set_latency",
        args: &[("port", "*mut jack_port_t"), ("arg1", "jack_nframes_t")],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_port_get_latency_range(
    //     port: *mut jack_port_t,
    //     mode: jack_latency_callback_mode_t,
    //     range: *mut jack_latency_range_t,
    // ) -> ();
    Function {
        name: "jack_port_get_latency_range",
        args: &[
            ("port", "*mut jack_port_t"),
            ("mode", "jack_latency_callback_mode_t"),
            ("range", "*mut jack_latency_range_t"),
        ],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_port_set_latency_range(
    //     port: *mut jack_port_t,
    //     mode: jack_latency_callback_mode_t,
    //     range: *mut jack_latency_range_t,
    // ) -> ();
    Function {
        name: "jack_port_set_latency_range",
        args: &[
            ("port", "*mut jack_port_t"),
            ("mode", "jack_latency_callback_mode_t"),
            ("range", "*mut jack_latency_range_t"),
        ],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_recompute_total_latencies(client: *mut jack_client_t) -> ::libc::c_int;
    Function {
        name: "jack_recompute_total_latencies",
        args: &[("client", "*mut jack_client_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_port_get_latency(port: *mut jack_port_t) -> jack_nframes_t;
    Function {
        name: "jack_port_get_latency",
        args: &[("port", "*mut jack_port_t")],
        ret: "jack_nframes_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_port_get_total_latency(
    //     client: *mut jack_client_t,
    //     port: *mut jack_port_t,
    // ) -> jack_nframes_t;
    Function {
        name: "jack_port_get_total_latency",
        args: &[
            ("client", "*mut jack_client_t"),
            ("port", "*mut jack_port_t"),
        ],
        ret: "jack_nframes_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_recompute_total_latency(
    //     arg1: *mut jack_client_t,
    //     port: *mut jack_port_t,
    // ) -> ::libc::c_int;
    Function {
        name: "jack_recompute_total_latency",
        args: &[("arg1", "*mut jack_client_t"), ("port", "*mut jack_port_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_get_ports(
    //     client: *mut jack_client_t,
    //     port_name_pattern: *const ::libc::c_char,
    //     type_name_pattern: *const ::libc::c_char,
    //     flags: ::libc::c_ulong,
    // ) -> *mut *const ::libc::c_char;
    Function {
        name: "jack_get_ports",
        args: &[
            ("client", "*mut jack_client_t"),
            ("port_name_pattern", "*const ::libc::c_char"),
            ("type_name_pattern", "*const ::libc::c_char"),
            ("flags", "::libc::c_ulong"),
        ],
        ret: "*mut *const ::libc::c_char",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_port_by_name(
    //     client: *mut jack_client_t,
    //     port_name: *const ::libc::c_char,
    // ) -> *mut jack_port_t;
    Function {
        name: "jack_port_by_name",
        args: &[
            ("client", "*mut jack_client_t"),
            ("port_name", "*const ::libc::c_char"),
        ],
        ret: "*mut jack_port_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_port_by_id(client: *mut jack_client_t, port_id: jack_port_id_t) -> *mut jack_port_t;
    Function {
        name: "jack_port_by_id",
        args: &[
            ("client", "*mut jack_client_t"),
            ("port_id", "jack_port_id_t"),
        ],
        ret: "*mut jack_port_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_frames_since_cycle_start(arg1: *const jack_client_t) -> jack_nframes_t;
    Function {
        name: "jack_frames_since_cycle_start",
        args: &[("arg1", "*const jack_client_t")],
        ret: "jack_nframes_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_frame_time(arg1: *const jack_client_t) -> jack_nframes_t;
    Function {
        name: "jack_frame_time",
        args: &[("arg1", "*const jack_client_t")],
        ret: "jack_nframes_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_last_frame_time(client: *const jack_client_t) -> jack_nframes_t;
    Function {
        name: "jack_last_frame_time",
        args: &[("client", "*const jack_client_t")],
        ret: "jack_nframes_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_frames_to_time(client: *const jack_client_t, arg1: jack_nframes_t) -> jack_time_t;
    Function {
        name: "jack_frames_to_time",
        args: &[
            ("client", "*const jack_client_t"),
            ("arg1", "jack_nframes_t"),
        ],
        ret: "jack_time_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_time_to_frames(client: *const jack_client_t, arg1: jack_time_t) -> jack_nframes_t;
    Function {
        name: "jack_time_to_frames",
        args: &[("client", "*const jack_client_t"), ("arg1", "jack_time_t")],
        ret: "jack_nframes_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_get_time() -> jack_time_t;
    Function {
        name: "jack_get_time",
        args: &[],
        ret: "jack_time_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_set_error_function(
    //     func: ::std::option::Option<unsafe extern "C" fn(arg1: *const ::libc::c_char) -> ()>,
    // ) -> ();
    Function {
        name: "jack_set_error_function",
        args: &[(
            "func",
            "::std::option::Option<unsafe extern \"C\" fn(arg1: *const ::libc::c_char) -> ()>",
        )],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_set_info_function(
    //     func: ::std::option::Option<unsafe extern "C" fn(arg1: *const ::libc::c_char) -> ()>,
    // ) -> ();
    Function {
        name: "jack_set_info_function",
        args: &[(
            "func",
            "::std::option::Option<unsafe extern \"C\" fn(arg1: *const ::libc::c_char) -> ()>",
        )],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_free(ptr: *mut ::libc::c_void) -> ();
    Function {
        name: "jack_free",
        args: &[("ptr", "*mut ::libc::c_void")],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_client_real_time_priority(arg1: *mut jack_client_t) -> ::libc::c_int;
    Function {
        name: "jack_client_real_time_priority",
        args: &[("arg1", "*mut jack_client_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_client_max_real_time_priority(arg1: *mut jack_client_t) -> ::libc::c_int;
    Function {
        name: "jack_client_max_real_time_priority",
        args: &[("arg1", "*mut jack_client_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // #[cfg(not(target_os = "windows"))]
    // pub fn jack_acquire_real_time_scheduling(
    //     thread: jack_native_thread_t,
    //     priority: ::libc::c_int,
    // ) -> ::libc::c_int;
    // #[cfg(not(target_os = "windows"))]
    // pub fn jack_client_create_thread(
    //     client: *mut jack_client_t,
    //     thread: *mut jack_native_thread_t,
    //     priority: ::libc::c_int,
    //     realtime: ::libc::c_int,
    //     start_routine: ::std::option::Option<
    //         unsafe extern "C" fn(arg1: *mut ::libc::c_void) -> *mut ::libc::c_void,
    //     >,
    //     arg: *mut ::libc::c_void,
    // ) -> ::libc::c_int;
    // #[cfg(not(target_os = "windows"))]
    // pub fn jack_drop_real_time_scheduling(thread: jack_native_thread_t) -> ::libc::c_int;
    // #[cfg(not(target_os = "windows"))]
    // pub fn jack_client_stop_thread(
    //     client: *mut jack_client_t,
    //     thread: jack_native_thread_t,
    // ) -> ::libc::c_int;
    // #[cfg(not(target_os = "windows"))]
    // pub fn jack_client_kill_thread(
    //     client: *mut jack_client_t,
    //     thread: jack_native_thread_t,
    // ) -> ::libc::c_int;
    // #[cfg(not(target_os = "windows"))]
    // pub fn jack_set_thread_creator(creator: jack_thread_creator_t) -> ();
    // pub fn jack_set_session_callback(
    //     client: *mut jack_client_t,
    //     session_callback: JackSessionCallback,
    //     arg: *mut ::libc::c_void,
    // ) -> ::libc::c_int;
    Function {
        name: "jack_set_session_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("session_callback", "JackSessionCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_session_reply(
    //     client: *mut jack_client_t,
    //     event: *mut jack_session_event_t,
    // ) -> ::libc::c_int;
    Function {
        name: "jack_session_reply",
        args: &[
            ("client", "*mut jack_client_t"),
            ("event", "*mut jack_session_event_t"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_session_event_free(event: *mut jack_session_event_t) -> ();
    Function {
        name: "jack_session_event_free",
        args: &[("event", "*mut jack_session_event_t")],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_client_get_uuid(client: *mut jack_client_t) -> *mut ::libc::c_char;
    Function {
        name: "jack_client_get_uuid",
        args: &[("client", "*mut jack_client_t")],
        ret: "*mut ::libc::c_char",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_session_notify(
    //     client: *mut jack_client_t,
    //     target: *const ::libc::c_char,
    //     _type: jack_session_event_type_t,
    //     path: *const ::libc::c_char,
    // ) -> *mut jack_session_command_t;
    Function {
        name: "jack_session_notify",
        args: &[
            ("client", "*mut jack_client_t"),
            ("target", "*const ::libc::c_char"),
            ("_type", "jack_session_event_type_t"),
            ("path", "*const ::libc::c_char"),
        ],
        ret: "*mut jack_session_command_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_session_commands_free(cmds: *mut jack_session_command_t) -> ();
    Function {
        name: "jack_session_commands_free",
        args: &[("cmds", "*mut jack_session_command_t")],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_reserve_client_name(
    //     client: *mut jack_client_t,
    //     name: *const ::libc::c_char,
    //     uuid: *const ::libc::c_char,
    // ) -> ::libc::c_int;
    Function {
        name: "jack_reserve_client_name",
        args: &[
            ("client", "*mut jack_client_t"),
            ("name", "*const ::libc::c_char"),
            ("uuid", "*const ::libc::c_char"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_client_has_session_callback(
    //     client: *mut jack_client_t,
    //     client_name: *const ::libc::c_char,
    // ) -> ::libc::c_int;
    Function {
        name: "jack_client_has_session_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("client_name", "*const ::libc::c_char"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jackctl_setup_signals(flags: ::libc::c_uint) -> *mut jackctl_sigmask_t;
    Function {
        name: "jackctl_setup_signals",
        args: &[("flags", "::libc::c_uint")],
        ret: "*mut jackctl_sigmask_t",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_wait_signals(signals: *mut jackctl_sigmask_t) -> ();
    Function {
        name: "jackctl_wait_signals",
        args: &[("signals", "*mut jackctl_sigmask_t")],
        ret: "()",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_server_create(
    //     on_device_acquire: ::std::option::Option<
    //         unsafe extern "C" fn(device_name: *const ::libc::c_char) -> u8,
    //     >,
    //     on_device_release: ::std::option::Option<
    //         unsafe extern "C" fn(device_name: *const ::libc::c_char) -> (),
    //     >,
    // ) -> *mut jackctl_server_t;
    Function {
        name: "jackctl_server_create",
        args: &[
            ("on_device_acquire", "::std::option::Option<unsafe extern \"C\" fn(device_name: *const ::libc::c_char) -> u8>"),
            ("on_device_release", "::std::option::Option<unsafe extern \"C\" fn(device_name: *const ::libc::c_char) -> ()>"),
        ],
        ret: "*mut jackctl_server_t",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_server_destroy(server: *mut jackctl_server_t) -> ();
    Function {
        name: "jackctl_server_destroy",
        args: &[("server", "*mut jackctl_server_t")],
        ret: "()",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_server_open(server: *mut jackctl_server_t, driver: *mut jackctl_driver_t) -> u8;
    Function {
        name: "jackctl_server_open",
        args: &[
            ("server", "*mut jackctl_server_t"),
            ("driver", "*mut jackctl_driver_t"),
        ],
        ret: "u8",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_server_start(server: *mut jackctl_server_t) -> u8;
    Function {
        name: "jackctl_server_start",
        args: &[("server", "*mut jackctl_server_t")],
        ret: "u8",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_server_stop(server: *mut jackctl_server_t) -> u8;
    Function {
        name: "jackctl_server_stop",
        args: &[("server", "*mut jackctl_server_t")],
        ret: "u8",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_server_close(server: *mut jackctl_server_t) -> u8;
    Function {
        name: "jackctl_server_close",
        args: &[("server", "*mut jackctl_server_t")],
        ret: "u8",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_server_get_drivers_list(server: *mut jackctl_server_t) -> *const JSList;
    Function {
        name: "jackctl_server_get_drivers_list",
        args: &[("server", "*mut jackctl_server_t")],
        ret: "*const JSList",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_server_get_parameters(server: *mut jackctl_server_t) -> *const JSList;
    Function {
        name: "jackctl_server_get_parameters",
        args: &[("server", "*mut jackctl_server_t")],
        ret: "*const JSList",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_server_get_internals_list(server: *mut jackctl_server_t) -> *const JSList;
    Function {
        name: "jackctl_server_get_internals_list",
        args: &[("server", "*mut jackctl_server_t")],
        ret: "*const JSList",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_server_load_internal(
    //     server: *mut jackctl_server_t,
    //     internal: *mut jackctl_internal_t,
    // ) -> u8;
    Function {
        name: "jackctl_server_load_internal",
        args: &[
            ("server", "*mut jackctl_server_t"),
            ("internal", "*mut jackctl_internal_t"),
        ],
        ret: "u8",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_server_unload_internal(
    //     server: *mut jackctl_server_t,
    //     internal: *mut jackctl_internal_t,
    // ) -> u8;
    Function {
        name: "jackctl_server_unload_internal",
        args: &[
            ("server", "*mut jackctl_server_t"),
            ("internal", "*mut jackctl_internal_t"),
        ],
        ret: "u8",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_server_add_slave(server: *mut jackctl_server_t, driver: *mut jackctl_driver_t)
    //     -> u8;
    Function {
        name: "jackctl_server_add_slave",
        args: &[
            ("server", "*mut jackctl_server_t"),
            ("driver", "*mut jackctl_driver_t"),
        ],
        ret: "u8",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_server_remove_slave(
    //     server: *mut jackctl_server_t,
    //     driver: *mut jackctl_driver_t,
    // ) -> u8;
    Function {
        name: "jackctl_server_remove_slave",
        args: &[
            ("server", "*mut jackctl_server_t"),
            ("driver", "*mut jackctl_driver_t"),
        ],
        ret: "u8",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_server_switch_master(
    //     server: *mut jackctl_server_t,
    //     driver: *mut jackctl_driver_t,
    // ) -> u8;
    Function {
        name: "jackctl_server_switch_master",
        args: &[
            ("server", "*mut jackctl_server_t"),
            ("driver", "*mut jackctl_driver_t"),
        ],
        ret: "u8",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_driver_get_name(driver: *mut jackctl_driver_t) -> *const ::libc::c_char;
    Function {
        name: "jackctl_driver_get_name",
        args: &[("driver", "*mut jackctl_driver_t")],
        ret: "*const ::libc::c_char",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_driver_get_type(driver: *mut jackctl_driver_t) -> jackctl_driver_type_t;
    Function {
        name: "jackctl_driver_get_type",
        args: &[("driver", "*mut jackctl_driver_t")],
        ret: "jackctl_driver_type_t",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_driver_get_parameters(driver: *mut jackctl_driver_t) -> *const JSList;
    Function {
        name: "jackctl_driver_get_parameters",
        args: &[("driver", "*mut jackctl_driver_t")],
        ret: "*const JSList",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_driver_params_parse(
    //     driver: *mut jackctl_driver_t,
    //     argc: ::libc::c_int,
    //     argv: *mut *mut ::libc::c_char,
    // ) -> ::libc::c_int;
    Function {
        name: "jackctl_driver_params_parse",
        args: &[
            ("driver", "*mut jackctl_driver_t"),
            ("argc", "::libc::c_int"),
            ("argv", "*mut *mut ::libc::c_char"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_internal_get_name(internal: *mut jackctl_internal_t) -> *const ::libc::c_char;
    Function {
        name: "jackctl_internal_get_name",
        args: &[("internal", "*mut jackctl_internal_t")],
        ret: "*const ::libc::c_char",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_internal_get_parameters(internal: *mut jackctl_internal_t) -> *const JSList;
    Function {
        name: "jackctl_internal_get_parameters",
        args: &[("internal", "*mut jackctl_internal_t")],
        ret: "*const JSList",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_get_name(parameter: *mut jackctl_parameter_t) -> *const ::libc::c_char;
    Function {
        name: "jackctl_parameter_get_name",
        args: &[("parameter", "*mut jackctl_parameter_t")],
        ret: "*const ::libc::c_char",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_get_short_description(
    //     parameter: *mut jackctl_parameter_t,
    // ) -> *const ::libc::c_char;
    Function {
        name: "jackctl_parameter_get_short_description",
        args: &[("parameter", "*mut jackctl_parameter_t")],
        ret: "*const ::libc::c_char",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_get_long_description(
    //     parameter: *mut jackctl_parameter_t,
    // ) -> *const ::libc::c_char;
    Function {
        name: "jackctl_parameter_get_long_description",
        args: &[("parameter", "*mut jackctl_parameter_t")],
        ret: "*const ::libc::c_char",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_get_type(parameter: *mut jackctl_parameter_t) -> jackctl_param_type_t;
    Function {
        name: "jackctl_parameter_get_type",
        args: &[("parameter", "*mut jackctl_parameter_t")],
        ret: "jackctl_param_type_t",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_get_id(parameter: *mut jackctl_parameter_t) -> ::libc::c_char;
    Function {
        name: "jackctl_parameter_get_id",
        args: &[("parameter", "*mut jackctl_parameter_t")],
        ret: "::libc::c_char",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_is_set(parameter: *mut jackctl_parameter_t) -> u8;
    Function {
        name: "jackctl_parameter_is_set",
        args: &[("parameter", "*mut jackctl_parameter_t")],
        ret: "u8",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_reset(parameter: *mut jackctl_parameter_t) -> u8;
    Function {
        name: "jackctl_parameter_reset",
        args: &[("parameter", "*mut jackctl_parameter_t")],
        ret: "u8",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_get_value(
    //     parameter: *mut jackctl_parameter_t,
    // ) -> Union_jackctl_parameter_value;
    Function {
        name: "jackctl_parameter_get_value",
        args: &[("parameter", "*mut jackctl_parameter_t")],
        ret: "Union_jackctl_parameter_value",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_set_value(
    //     parameter: *mut jackctl_parameter_t,
    //     value_ptr: *const Union_jackctl_parameter_value,
    // ) -> u8;
    Function {
        name: "jackctl_parameter_set_value",
        args: &[
            ("parameter", "*mut jackctl_parameter_t"),
            ("value_ptr", "*const Union_jackctl_parameter_value"),
        ],
        ret: "u8",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_get_default_value(
    //     parameter: *mut jackctl_parameter_t,
    // ) -> Union_jackctl_parameter_value;
    Function {
        name: "jackctl_parameter_get_default_value",
        args: &[("parameter", "*mut jackctl_parameter_t")],
        ret: "Union_jackctl_parameter_value",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_has_range_constraint(parameter: *mut jackctl_parameter_t) -> u8;
    Function {
        name: "jackctl_parameter_has_range_constraint",
        args: &[("parameter", "*mut jackctl_parameter_t")],
        ret: "u8",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_has_enum_constraint(parameter: *mut jackctl_parameter_t) -> u8;
    Function {
        name: "jackctl_parameter_has_enum_constraint",
        args: &[("parameter", "*mut jackctl_parameter_t")],
        ret: "u8",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_get_enum_constraints_count(parameter: *mut jackctl_parameter_t) -> u32;
    Function {
        name: "jackctl_parameter_get_enum_constraints_count",
        args: &[("parameter", "*mut jackctl_parameter_t")],
        ret: "u32",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_get_enum_constraint_value(
    //     parameter: *mut jackctl_parameter_t,
    //     index: u32,
    // ) -> Union_jackctl_parameter_value;
    Function {
        name: "jackctl_parameter_get_enum_constraint_value",
        args: &[
            ("parameter", "*mut jackctl_parameter_t"),
            ("index", "u32"),
        ],
        ret: "Union_jackctl_parameter_value",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_get_enum_constraint_description(
    //     parameter: *mut jackctl_parameter_t,
    //     index: u32,
    // ) -> *const ::libc::c_char;
    Function {
        name: "jackctl_parameter_get_enum_constraint_description",
        args: &[
            ("parameter", "*mut jackctl_parameter_t"),
            ("index", "u32"),
        ],
        ret: "*const ::libc::c_char",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_get_range_constraint(
    //     parameter: *mut jackctl_parameter_t,
    //     min_ptr: *mut Union_jackctl_parameter_value,
    //     max_ptr: *mut Union_jackctl_parameter_value,
    // ) -> ();
    Function {
        name: "jackctl_parameter_get_range_constraint",
        args: &[
            ("parameter", "*mut jackctl_parameter_t"),
            ("min_ptr", "*mut Union_jackctl_parameter_value"),
            ("max_ptr", "*mut Union_jackctl_parameter_value"),
        ],
        ret: "()",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_constraint_is_strict(parameter: *mut jackctl_parameter_t) -> u8;
    Function {
        name: "jackctl_parameter_constraint_is_strict",
        args: &[("parameter", "*mut jackctl_parameter_t")],
        ret: "u8",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jackctl_parameter_constraint_is_fake_value(parameter: *mut jackctl_parameter_t) -> u8;
    Function {
        name: "jackctl_parameter_constraint_is_fake_value",
        args: &[("parameter", "*mut jackctl_parameter_t")],
        ret: "u8",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jack_error(format: *const ::libc::c_char, ...) -> ();
    // Function {
    //     name: "jack_error",
    //     args: &[
    //         ("format", "*const ::libc::c_char"),
    //         // Varargs not yet supported.
    //         // (""..."),
    //     ],
    //     ret: "()",
    //     flags: FunctionFlags::NONE,
    // },
    // pub fn jack_info(format: *const ::libc::c_char, ...) -> ();
    // Function {
    //     name: "jack_info",
    //     args: &[
    //         ("format", "*const ::libc::c_char"),
    //         // Varargs not yet supported.
    //         // ("", "..."),
    //     ],
    //     ret: "()",
    //     flags: FunctionFlags::NONE,
    // },
    // pub fn jack_log(format: *const ::libc::c_char, ...) -> ();
    // Function {
    //     name: "jack_log",
    //     args: &[
    //         ("format", "*const ::libc::c_char"),
    //         // Varargs not yet supported.
    //         // ("", "..."),
    //     ],
    //     ret: "()",
    //     flags: FunctionFlags::NONE,
    // },
    // pub fn jack_set_property(
    //     arg1: *mut jack_client_t,
    //     subject: jack_uuid_t,
    //     key: *const ::libc::c_char,
    //     value: *const ::libc::c_char,
    //     _type: *const ::libc::c_char,
    // ) -> ::libc::c_int;
    Function {
        name: "jack_set_property",
        args: &[
            ("arg1", "*mut jack_client_t"),
            ("subject", "jack_uuid_t"),
            ("key", "*const ::libc::c_char"),
            ("value", "*const ::libc::c_char"),
            ("_type", "*const ::libc::c_char"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_get_property(
    //     subject: jack_uuid_t,
    //     key: *const ::libc::c_char,
    //     value: *mut *mut ::libc::c_char,
    //     _type: *mut *mut ::libc::c_char,
    // ) -> ::libc::c_int;
    Function {
        name: "jack_get_property",
        args: &[
            ("subject", "jack_uuid_t"),
            ("key", "*const ::libc::c_char"),
            ("value", "*mut *mut ::libc::c_char"),
            ("_type", "*mut *mut ::libc::c_char"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_free_description(
    //     desc: *mut jack_description_t,
    //     free_description_itself: ::libc::c_int,
    // ) -> ();
    Function {
        name: "jack_free_description",
        args: &[
            ("desc", "*mut jack_description_t"),
            ("free_description_itself", "::libc::c_int"),
        ],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_get_properties(subject: jack_uuid_t, desc: *mut jack_description_t) -> ::libc::c_int;
    Function {
        name: "jack_get_properties",
        args: &[
            ("subject", "jack_uuid_t"),
            ("desc", "*mut jack_description_t"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_get_all_properties(descs: *mut *mut jack_description_t) -> ::libc::c_int;
    Function {
        name: "jack_get_all_properties",
        args: &[("descs", "*mut *mut jack_description_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_remove_property(
    //     client: *mut jack_client_t,
    //     subject: jack_uuid_t,
    //     key: *const ::libc::c_char,
    // ) -> ::libc::c_int;
    Function {
        name: "jack_remove_property",
        args: &[
            ("client", "*mut jack_client_t"),
            ("subject", "jack_uuid_t"),
            ("key", "*const ::libc::c_char"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_remove_properties(client: *mut jack_client_t, subject: jack_uuid_t) -> ::libc::c_int;
    Function {
        name: "jack_remove_properties",
        args: &[("client", "*mut jack_client_t"), ("subject", "jack_uuid_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_remove_all_properties(client: *mut jack_client_t) -> ::libc::c_int;
    Function {
        name: "jack_remove_all_properties",
        args: &[("client", "*mut jack_client_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_set_property_change_callback(
    //     client: *mut jack_client_t,
    //     callback: JackPropertyChangeCallback,
    //     arg: *mut ::libc::c_void,
    // ) -> ::libc::c_int;
    Function {
        name: "jack_set_property_change_callback",
        args: &[
            ("client", "*mut jack_client_t"),
            ("callback", "JackPropertyChangeCallback"),
            ("arg", "*mut ::libc::c_void"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_get_internal_client_name(
    //     client: *mut jack_client_t,
    //     intclient: jack_intclient_t,
    // ) -> *mut ::libc::c_char;
    Function {
        name: "jack_get_internal_client_name",
        args: &[
            ("client", "*mut jack_client_t"),
            ("intclient", "jack_intclient_t"),
        ],
        ret: "*mut ::libc::c_char",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jack_internal_client_handle(
    //     client: *mut jack_client_t,
    //     client_name: *const ::libc::c_char,
    //     status: *mut jack_status_t,
    // ) -> jack_intclient_t;
    Function {
        name: "jack_internal_client_handle",
        args: &[
            ("client", "*mut jack_client_t"),
            ("client_name", "*const ::libc::c_char"),
            ("status", "*mut jack_status_t"),
        ],
        ret: "jack_intclient_t",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jack_internal_client_load(
    //     client: *mut jack_client_t,
    //     client_name: *const ::libc::c_char,
    //     options: jack_options_t,
    //     status: *mut jack_status_t,
    //     ...
    // ) -> jack_intclient_t;
    Function {
        name: "jack_internal_client_load",
        args: &[
            ("client", "*mut jack_client_t"),
            ("client_name", "*const ::libc::c_char"),
            ("options", "jack_options_t"),
            ("status", "*mut jack_status_t"),
            ("load_name", "*const ::libc::c_char"),
            ("load_init", "*const ::libc::c_char"),
            // Varargs not supported.
            // ("", "..."),
        ],
        ret: "jack_intclient_t",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jack_internal_client_unload(
    //     client: *mut jack_client_t,
    //     intclient: jack_intclient_t,
    // ) -> jack_status_t;
    Function {
        name: "jack_internal_client_unload",
        args: &[
            ("client", "*mut jack_client_t"),
            ("intclient", "jack_intclient_t"),
        ],
        ret: "jack_status_t",
        flags: FunctionFlags::WEAK,
    },
    // pub fn jack_get_max_delayed_usecs(client: *mut jack_client_t) -> ::libc::c_float;
    Function {
        name: "jack_get_max_delayed_usecs",
        args: &[("client", "*mut jack_client_t")],
        ret: "::libc::c_float",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_get_xrun_delayed_usecs(client: *mut jack_client_t) -> ::libc::c_float;
    Function {
        name: "jack_get_xrun_delayed_usecs",
        args: &[("client", "*mut jack_client_t")],
        ret: "::libc::c_float",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_reset_max_delayed_usecs(client: *mut jack_client_t) -> ();
    Function {
        name: "jack_reset_max_delayed_usecs",
        args: &[("client", "*mut jack_client_t")],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_midi_get_event_count(port_buffer: *mut ::libc::c_void) -> u32;
    Function {
        name: "jack_midi_get_event_count",
        args: &[("port_buffer", "*mut ::libc::c_void")],
        ret: "u32",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_midi_event_get(
    //     event: *mut jack_midi_event_t,
    //     port_buffer: *mut ::libc::c_void,
    //     event_index: u32,
    // ) -> ::libc::c_int;
    Function {
        name: "jack_midi_event_get",
        args: &[
            ("event", "*mut jack_midi_event_t"),
            ("port_buffer", "*mut ::libc::c_void"),
            ("event_index", "u32"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_midi_clear_buffer(port_buffer: *mut ::libc::c_void) -> ();
    Function {
        name: "jack_midi_clear_buffer",
        args: &[("port_buffer", "*mut ::libc::c_void")],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_midi_max_event_size(port_buffer: *mut ::libc::c_void) -> ::libc::size_t;
    Function {
        name: "jack_midi_max_event_size",
        args: &[("port_buffer", "*mut ::libc::c_void")],
        ret: "::libc::size_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_midi_event_reserve(
    //     port_buffer: *mut ::libc::c_void,
    //     time: jack_nframes_t,
    //     data_size: ::libc::size_t,
    // ) -> *mut jack_midi_data_t;
    Function {
        name: "jack_midi_event_reserve",
        args: &[
            ("port_buffer", "*mut ::libc::c_void"),
            ("time", "jack_nframes_t"),
            ("data_size", "::libc::size_t"),
        ],
        ret: "*mut jack_midi_data_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_midi_event_write(
    //     port_buffer: *mut ::libc::c_void,
    //     time: jack_nframes_t,
    //     data: *const jack_midi_data_t,
    //     data_size: ::libc::size_t,
    // ) -> ::libc::c_int;
    Function {
        name: "jack_midi_event_write",
        args: &[
            ("port_buffer", "*mut ::libc::c_void"),
            ("time", "jack_nframes_t"),
            ("data", "*const jack_midi_data_t"),
            ("data_size", "::libc::size_t"),
        ],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_midi_get_lost_event_count(port_buffer: *mut ::libc::c_void) -> u32;
    Function {
        name: "jack_midi_get_lost_event_count",
        args: &[("port_buffer", "*mut ::libc::c_void")],
        ret: "u32",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_ringbuffer_create(sz: ::libc::size_t) -> *mut jack_ringbuffer_t;
    Function {
        name: "jack_ringbuffer_create",
        args: &[("sz", "::libc::size_t")],
        ret: "*mut jack_ringbuffer_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_ringbuffer_free(rb: *mut jack_ringbuffer_t) -> ();
    Function {
        name: "jack_ringbuffer_free",
        args: &[("rb", "*mut jack_ringbuffer_t")],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_ringbuffer_get_read_vector(
    //     rb: *const jack_ringbuffer_t,
    //     vec: *mut jack_ringbuffer_data_t,
    // ) -> ();
    Function {
        name: "jack_ringbuffer_get_read_vector",
        args: &[
            ("rb", "*const jack_ringbuffer_t"),
            ("vec", "*mut jack_ringbuffer_data_t"),
        ],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_ringbuffer_get_write_vector(
    //     rb: *const jack_ringbuffer_t,
    //     vec: *mut jack_ringbuffer_data_t,
    // ) -> ();
    Function {
        name: "jack_ringbuffer_get_write_vector",
        args: &[
            ("rb", "*const jack_ringbuffer_t"),
            ("vec", "*mut jack_ringbuffer_data_t"),
        ],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_ringbuffer_read(
    //     rb: *mut jack_ringbuffer_t,
    //     dest: *mut ::libc::c_char,
    //     cnt: ::libc::size_t,
    // ) -> ::libc::size_t;
    Function {
        name: "jack_ringbuffer_read",
        args: &[
            ("rb", "*mut jack_ringbuffer_t"),
            ("dest", "*mut ::libc::c_char"),
            ("cnt", "::libc::size_t"),
        ],
        ret: "::libc::size_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_ringbuffer_peek(
    //     rb: *mut jack_ringbuffer_t,
    //     dest: *mut ::libc::c_char,
    //     cnt: ::libc::size_t,
    // ) -> ::libc::size_t;
    Function {
        name: "jack_ringbuffer_peek",
        args: &[
            ("rb", "*mut jack_ringbuffer_t"),
            ("dest", "*mut ::libc::c_char"),
            ("cnt", "::libc::size_t"),
        ],
        ret: "::libc::size_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_ringbuffer_read_advance(rb: *mut jack_ringbuffer_t, cnt: ::libc::size_t) -> ();
    Function {
        name: "jack_ringbuffer_read_advance",
        args: &[("rb", "*mut jack_ringbuffer_t"), ("cnt", "::libc::size_t")],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_ringbuffer_read_space(rb: *const jack_ringbuffer_t) -> ::libc::size_t;
    Function {
        name: "jack_ringbuffer_read_space",
        args: &[("rb", "*const jack_ringbuffer_t")],
        ret: "::libc::size_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_ringbuffer_mlock(rb: *mut jack_ringbuffer_t) -> ::libc::c_int;
    Function {
        name: "jack_ringbuffer_mlock",
        args: &[("rb", "*mut jack_ringbuffer_t")],
        ret: "::libc::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_ringbuffer_reset(rb: *mut jack_ringbuffer_t) -> ();
    Function {
        name: "jack_ringbuffer_reset",
        args: &[("rb", "*mut jack_ringbuffer_t")],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_ringbuffer_reset_size(rb: *mut jack_ringbuffer_t, sz: ::libc::size_t) -> ();
    Function {
        name: "jack_ringbuffer_reset_size",
        args: &[("rb", "*mut jack_ringbuffer_t"), ("sz", "::libc::size_t")],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_ringbuffer_write(
    //     rb: *mut jack_ringbuffer_t,
    //     src: *const ::libc::c_char,
    //     cnt: ::libc::size_t,
    // ) -> ::libc::size_t;
    Function {
        name: "jack_ringbuffer_write",
        args: &[
            ("rb", "*mut jack_ringbuffer_t"),
            ("src", "*const ::libc::c_char"),
            ("cnt", "::libc::size_t"),
        ],
        ret: "::libc::size_t",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_ringbuffer_write_advance(rb: *mut jack_ringbuffer_t, cnt: ::libc::size_t) -> ();
    Function {
        name: "jack_ringbuffer_write_advance",
        args: &[("rb", "*mut jack_ringbuffer_t"), ("cnt", "::libc::size_t")],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_ringbuffer_write_space(rb: *const jack_ringbuffer_t) -> ::libc::size_t;
    Function {
        name: "jack_ringbuffer_write_space",
        args: &[("rb", "*const jack_ringbuffer_t")],
        ret: "::libc::size_t",
        flags: FunctionFlags::NONE,
    },

    // pub fn jack_uuid_to_index(arg1: jack_uuid_t) -> u32;
    Function {
        name: "jack_uuid_to_index",
        args: &[("arg1", "jack_uuid_t")],
        ret: "u32",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_uuid_compare(arg1: jack_uuid_t, arg2: jack_uuid_t) -> ::std::os::raw::c_int;
    Function {
        name: "jack_uuid_compare",
        args: &[("arg1", "jack_uuid_t"), ("arg2", "jack_uuid_t")],
        ret: "::std::os::raw::c_int",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_uuid_copy(dst: *mut jack_uuid_t, src: jack_uuid_t);
    Function {
        name: "jack_uuid_copy",
        args: &[("dst", "*mut jack_uuid_t"), ("src", "jack_uuid_t")],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    // pub fn jack_uuid_clear(arg1: *mut jack_uuid_t);
    Function {
        name: "jack_uuid_clear",
        args: &[("arg1", "*mut jack_uuid_t")],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_uuid_parse",
        args: &[
            ("buf", "*const ::std::os::raw::c_char"),
            ("arg1", "*mut jack_uuid_t"),
        ],
        ret: "::std::os::raw::c_int",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_uuid_unparse",
        args: &[
            ("arg1", "jack_uuid_t"),
            ("buf", "*mut ::std::os::raw::c_char"),
        ],
        ret: "()",
        flags: FunctionFlags::NONE,
    },
    Function {
        name: "jack_uuid_empty",
        args: &[("arg1", "jack_uuid_t")],
        ret: "::std::os::raw::c_int",
        flags: FunctionFlags::NONE,
    },
];
