#[test]
fn panic_in_process_handler_propagates_as_error_in_deactivate() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    let (send, recv) = std::sync::mpsc::sync_channel(1);
    let process_handler = crate::contrib::ClosureProcessHandler::new(move |_, _| {
        send.try_send(true).ok();
        panic!("panic should convert to error!");
    });
    let ac = client.activate_async((), process_handler).unwrap();
    assert!(recv
        .recv_timeout(std::time::Duration::from_secs(1))
        .unwrap());
    assert_eq!(ac.deactivate().err(), Some(crate::Error::ClientPanicked));
}

#[test]
fn panic_in_buffer_size_handler_propagates_as_error_in_deactivate() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    let (send, recv) = std::sync::mpsc::sync_channel(2);
    let handler = crate::contrib::ClosureProcessHandler::with_state(
        (),
        move |_, _, _| {
            send.try_send(true).unwrap();
            panic!("intentional panic here");
        },
        move |_, _, _| crate::Control::Continue,
    );
    let ac = client.activate_async((), handler).unwrap();
    assert!(recv
        .recv_timeout(std::time::Duration::from_secs(1))
        .unwrap());
    assert_eq!(ac.deactivate().err(), Some(crate::Error::ClientPanicked));
}

#[test]
fn quitting_stops_calling_process() {
    eprintln!("Creating client.");
    let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    let mut calls = 0;
    let (send, recv) = std::sync::mpsc::sync_channel(2);
    eprintln!("Creating callback.");
    let process_handler = crate::contrib::ClosureProcessHandler::new(move |_, _| {
        send.try_send(true).unwrap();
        calls += 1;
        assert_eq!(calls, 1);
        crate::Control::Quit
    });
    eprintln!("Activating client.");
    let ac = client.activate_async((), process_handler).unwrap();
    eprintln!("Waiting for async response.");
    assert!(recv
        .recv_timeout(std::time::Duration::from_secs(1))
        .unwrap());
    ac.deactivate().unwrap();
}

#[test]
fn quitting_buffer_size_never_runs_process() {
    // JACK calls `buffer_size` once before the first `process`. If it quits, `process` must never
    // be called.
    let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    let (send, recv) = std::sync::mpsc::sync_channel(2);
    let handler = crate::contrib::ClosureProcessHandler::with_state(
        (),
        |_, _, _| panic!("buffer_size quit, so process should not be called"),
        move |_, _, _| {
            send.try_send(true).ok();
            crate::Control::Quit
        },
    );
    let ac = client.activate_async((), handler).unwrap();
    assert!(recv
        .recv_timeout(std::time::Duration::from_secs(1))
        .unwrap());
    // Give the process handler some time to try to run.
    std::thread::sleep(std::time::Duration::from_millis(500));
    ac.deactivate().unwrap();
}

#[test]
fn buffer_size_is_called_before_process() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    let (send, recv) = std::sync::mpsc::sync_channel(2);
    let process_handler = crate::contrib::ClosureProcessHandler::with_state(
        "initializing",
        move |state, _, _| {
            assert_eq!(*state, "processing");
            send.try_send(true).ok();
            crate::Control::Continue
        },
        |state, _, _| {
            assert_eq!(*state, "initializing");
            *state = "processing";
            // Give the processing thread some time to run, in case it wants to.
            std::thread::sleep(std::time::Duration::from_secs(3));
            crate::Control::Continue
        },
    );
    let ac = client.activate_async((), process_handler).unwrap();
    assert!(recv
        .recv_timeout(std::time::Duration::from_secs(5))
        .unwrap());
    assert_eq!(ac.deactivate().unwrap().2.state, "processing");
}

#[test]
fn signals_in_audio_ports_are_forwarded() {
    // Setup clients and ports.
    let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    let input = client
        .register_port("in", crate::AudioIn::default())
        .unwrap();
    let mut output = client
        .register_port("out", crate::AudioOut::default())
        .unwrap();
    let (input_name, output_name) = (input.name().unwrap(), output.name().unwrap());
    let (send, recv) = std::sync::mpsc::sync_channel(1);

    // Setup checks.
    let process_handler = crate::contrib::ClosureProcessHandler::new(move |_, ps| {
        // The buffer size can change after activation, so use the current process cycle's size.
        let buffer_size = ps.n_frames() as usize;
        assert_ne!(buffer_size, 0);
        let test_val = 0.25;
        output.as_mut_slice(ps).fill(test_val);
        assert_eq!(output.as_mut_slice(ps).len(), buffer_size);

        assert_eq!(input.as_slice(ps).len(), buffer_size);
        // We don't fail if the input is not yet ready as this depends on port connection. Port
        // connection takes some time so the first few iterations may not contain the expected data.
        if input.as_slice(ps).iter().all(|x| *x == test_val) {
            send.try_send(true).unwrap();
            crate::Control::Quit
        } else {
            crate::Control::Continue
        }
    });

    // Runs checks.
    let ac = client.activate_async((), process_handler).unwrap();
    ac.as_client()
        .connect_ports_by_name(&output_name, &input_name)
        .unwrap();
    assert!(recv
        .recv_timeout(std::time::Duration::from_secs(1))
        .unwrap());
    ac.deactivate().unwrap();
}

#[test]
fn messages_in_midi_ports_are_forwarded() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();

    let buffer_size = client.buffer_size() as usize;
    assert_ne!(buffer_size, 0);
    let input = client
        .register_port("in", crate::MidiIn::default())
        .unwrap();
    let mut output = client
        .register_port("out", crate::MidiOut::default())
        .unwrap();
    let (input_name, output_name) = (input.name().unwrap(), output.name().unwrap());
    let (send, recv) = std::sync::mpsc::sync_channel(1);
    let process_handler = crate::contrib::ClosureProcessHandler::new(move |_, ps| {
        let mut writer = output.writer(ps);
        assert_ne!(writer.max_event_size(), 0);
        for time in 0..10 {
            writer
                .write(&crate::RawMidi {
                    time,
                    bytes: &[0, 1, 2],
                })
                .unwrap();
        }

        let iter = input.iter(ps);
        let ports_are_probably_connected = iter.clone().count() == 10;
        if ports_are_probably_connected {
            for (idx, msg) in iter.enumerate() {
                assert_eq!(msg.time as usize, idx);
                assert_eq!(msg.bytes, &[0, 1, 2]);
            }
            send.try_send(true).unwrap();
            crate::Control::Quit
        } else {
            crate::Control::Continue
        }
    });
    let ac = client.activate_async((), process_handler).unwrap();
    ac.as_client()
        .connect_ports_by_name(&output_name, &input_name)
        .unwrap();
    assert!(recv
        .recv_timeout(std::time::Duration::from_secs(1))
        .unwrap());
    ac.deactivate().unwrap();
}

#[test]
fn activating_client_notifies_buffer_size_before_beginning() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    let initial_buffer_size = client.buffer_size() as usize;
    assert_ne!(initial_buffer_size, 0);
}

#[test]
fn deactivated_client_survives_graph_changes() {
    // After deactivation, JACK must not call into the freed callback context. Before all callbacks
    // were cleared on deactivation, this crashed the test binary with a use-after-free.
    struct Handler(std::sync::Arc<std::sync::atomic::AtomicUsize>);
    impl crate::NotificationHandler for Handler {
        fn client_registration(&mut self, _: &crate::Client, _: &str, _: bool) {
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
    }
    let registrations = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let (client, _) = crate::Client::new("deactivated", crate::ClientOptions::default()).unwrap();
    let ac = client
        .activate_async(Handler(registrations.clone()), ())
        .unwrap();
    let (client, _handler, _) = ac.deactivate().unwrap();
    let registrations_at_deactivation = registrations.load(std::sync::atomic::Ordering::SeqCst);
    for i in 0..3 {
        let name = format!("graph-change-{i}");
        let (other, _) = crate::Client::new(&name, crate::ClientOptions::default()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
        drop(other);
    }
    std::thread::sleep(std::time::Duration::from_millis(200));
    assert_eq!(
        registrations.load(std::sync::atomic::Ordering::SeqCst),
        registrations_at_deactivation
    );
    drop(client);
}

#[test]
fn set_freewheel_notifies_freewheel_handler() {
    struct Handler(std::sync::mpsc::SyncSender<bool>);
    impl crate::NotificationHandler for Handler {
        fn freewheel(&mut self, _: &crate::Client, is_freewheel_enabled: bool) {
            self.0.try_send(is_freewheel_enabled).ok();
        }
    }
    let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    let (send, recv) = std::sync::mpsc::sync_channel(4);
    let ac = client.activate_async(Handler(send), ()).unwrap();
    let timeout = std::time::Duration::from_secs(1);
    ac.as_client().set_freewheel(true).unwrap();
    assert_eq!(recv.recv_timeout(timeout), Ok(true));
    ac.as_client().set_freewheel(false).unwrap();
    assert_eq!(recv.recv_timeout(timeout), Ok(false));
    ac.deactivate().unwrap();
}

#[test]
fn renaming_a_port_calls_port_rename() {
    struct Handler(std::sync::mpsc::SyncSender<(String, String)>);
    impl crate::NotificationHandler for Handler {
        fn port_rename(
            &mut self,
            _: &crate::Client,
            _: crate::PortId,
            old_name: &str,
            new_name: &str,
        ) -> crate::Control {
            self.0
                .try_send((old_name.to_string(), new_name.to_string()))
                .ok();
            crate::Control::Continue
        }
    }
    let (owner, _) = crate::Client::new("renamer", crate::ClientOptions::default()).unwrap();
    let mut port = owner
        .register_port("before", crate::AudioIn::default())
        .unwrap();
    let _owner = owner.activate_async((), ()).unwrap();
    let (observer, _) = crate::Client::new("observer", crate::ClientOptions::default()).unwrap();
    let (send, recv) = std::sync::mpsc::sync_channel(4);
    let ac = observer.activate_async(Handler(send), ()).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(100));
    port.set_name("after").unwrap();
    let (old_name, new_name) = recv
        .recv_timeout(std::time::Duration::from_secs(1))
        .unwrap();
    assert!(old_name.ends_with(":before"), "{}", old_name);
    assert!(new_name.ends_with(":after"), "{}", new_name);
    ac.deactivate().unwrap();
}

#[test]
fn writing_too_much_midi_returns_not_enough_space() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    let mut output = client
        .register_port("out", crate::MidiOut::default())
        .unwrap();
    let (send, recv) = std::sync::mpsc::sync_channel(1);
    let process_handler = crate::contrib::ClosureProcessHandler::new(move |_, ps| {
        let mut writer = output.writer(ps);
        let too_big = vec![0x90; writer.max_event_size() + 1];
        let result = writer.write(&crate::RawMidi {
            time: 0,
            bytes: &too_big,
        });
        send.try_send(result).ok();
        crate::Control::Quit
    });
    let ac = client.activate_async((), process_handler).unwrap();
    let result = recv
        .recv_timeout(std::time::Duration::from_secs(1))
        .unwrap();
    ac.deactivate().ok();
    assert_eq!(result, Err(crate::Error::NotEnoughSpace));
}

#[test]
fn failed_activation_returns_an_error_and_drops_the_handlers() {
    struct DropFlag(std::sync::Arc<std::sync::atomic::AtomicBool>);
    impl Drop for DropFlag {
        fn drop(&mut self) {
            self.0.store(true, std::sync::atomic::Ordering::SeqCst);
        }
    }
    impl crate::NotificationHandler for DropFlag {}

    let dropped = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
    // JACK refuses to register callbacks on a client that is already active.
    assert_eq!(unsafe { jack_sys::jack_activate(client.raw()) }, 0);
    let res = client.activate_async(DropFlag(dropped.clone()), ());
    assert!(
        matches!(res, Err(crate::Error::CallbackRegistrationError)),
        "{:?}",
        res.map(|_| ())
    );
    assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
}

#[test]
fn deactivating_while_process_writes_to_stderr_does_not_abort() {
    // PipeWire's JACK stops its process thread with `pthread_cancel`. If the cancellation lands
    // on a cancellation point inside a Rust callback (such as the `write` behind `eprintln!`), the
    // forced unwind used to run through `catch_unwind` and abort the whole process with "FATAL:
    // exception not rethrown".
    for _ in 0..10 {
        let (client, _) = crate::Client::new("", crate::ClientOptions::default()).unwrap();
        let (send, recv) = std::sync::mpsc::sync_channel(1);
        let process_handler = crate::contrib::ClosureProcessHandler::new(move |_, _| {
            send.try_send(()).ok();
            for _ in 0..1000 {
                eprint!("");
                std::io::Write::flush(&mut std::io::stderr()).ok();
                // `write` with a zero length still reaches a cancellation point.
                unsafe { libc::write(2, std::ptr::null(), 0) };
            }
            crate::Control::Continue
        });
        let ac = client.activate_async((), process_handler).unwrap();
        recv.recv_timeout(std::time::Duration::from_secs(1))
            .unwrap();
        ac.deactivate().unwrap();
    }
}
