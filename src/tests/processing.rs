use crate::RawMidi;

#[test]
fn panic_in_process_handler_propagates_as_error_in_deactivate() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::NO_START_SERVER).unwrap();
    let (send, recv) = std::sync::mpsc::sync_channel(1);
    eprintln!("Activating async client.");
    let process_handler = crate::contrib::ClosureProcessHandler::new(move |_, _| {
        send.try_send(true).ok();
        panic!("panic should convert to error!");
    });
    let ac = client.activate_async((), process_handler).unwrap();
    eprintln!("Waiting for process signal.");
    assert!(recv
        .recv_timeout(std::time::Duration::from_secs(1))
        .unwrap());
    eprintln!("Deactivating client.");
    assert_eq!(ac.deactivate().err(), Some(crate::Error::ClientPanicked));
}

#[test]
fn quitting_stops_calling_process() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::NO_START_SERVER).unwrap();
    let mut calls = 0;
    let (send, recv) = std::sync::mpsc::sync_channel(2);
    let process_handler = crate::contrib::ClosureProcessHandler::new(move |_, _| {
        send.try_send(true).unwrap();
        calls += 1;
        assert_eq!(calls, 1);
        crate::Control::Quit
    });
    eprintln!("Activating async client.");
    let ac = client.activate_async((), process_handler).unwrap();
    eprintln!("Waiting for process signal.");
    assert!(recv
        .recv_timeout(std::time::Duration::from_secs(1))
        .unwrap());
    eprintln!("Deactivating client.");
    ac.deactivate().unwrap();
}

#[test]
fn signals_in_audio_ports_are_forwarded() {
    // Setup clients and ports.
    let (client, _) = crate::Client::new("", crate::ClientOptions::NO_START_SERVER).unwrap();
    let buffer_size = client.buffer_size() as usize;
    assert_ne!(buffer_size, 0);
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
    eprintln!("Activating async client.");
    let ac = client.activate_async((), process_handler).unwrap();
    ac.as_client()
        .connect_ports_by_name(&output_name, &input_name)
        .unwrap();
    eprintln!("Waiting for process signal.");
    assert!(recv
        .recv_timeout(std::time::Duration::from_secs(1))
        .unwrap());
    eprintln!("Deactivating client.");
    ac.deactivate().unwrap();
}

#[test]
fn messages_in_midi_ports_are_forwarded() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::NO_START_SERVER).unwrap();

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
                .write(&RawMidi {
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
    let (client, _) = crate::Client::new("", crate::ClientOptions::NO_START_SERVER).unwrap();
    let initial_buffer_size = client.buffer_size() as usize;
    assert_ne!(initial_buffer_size, 0);
}
