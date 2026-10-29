use approx::assert_abs_diff_eq;

/// Wait until the server reports timing information, which it may only do once process cycles
/// have run.
fn wait_for_timing(client: &crate::Client) {
    crate::tests::wait_for(std::time::Duration::from_secs(1), || {
        client.frames_to_time(client.sample_rate()) != client.frames_to_time(0)
    });
}

#[test]
fn frame_and_time_are_convertable() {
    let ac = crate::tests::active_client();
    let client = ac.as_client();
    let frame_time = client.frame_time();
    assert_abs_diff_eq!(
        client.time_to_frames(client.frames_to_time(frame_time)) as f64,
        frame_time as f64,
        epsilon = 1.0,
    );
}

#[test]
fn one_frame_duration_is_inverse_of_sample_rate() {
    // Some servers, such as PipeWire, only convert times for active clients.
    let ac = crate::tests::active_client();
    let client = ac.as_client();
    wait_for_timing(client);
    let sample_rate = client.sample_rate();
    assert_abs_diff_eq!(
        (client.frames_to_time(sample_rate as _) - client.frames_to_time(0)) as f64,
        1_000_000.0,
        epsilon = 1_000_000.0 * 1e-3,
    );
}

#[test]
fn one_second_is_sample_rate_frames() {
    let ac = crate::tests::active_client();
    let client = ac.as_client();
    wait_for_timing(client);
    // Convert times near now. Times far from the current cycle, such as 0, may not be
    // representable by every server.
    let now = client.time();
    let t0 = client.time_to_frames(now);
    let t1 = client.time_to_frames(now + 1_000_000);
    assert_abs_diff_eq!(
        (t1 - t0) as f64,
        client.sample_rate() as f64,
        epsilon = client.sample_rate() as f64 * 1e-3
    );
}

#[test]
fn cycle_times_are_available_in_process() {
    // `jack_get_cycle_times` is an optional symbol. It must be found both with dynamic loading and
    // with static linking (`--no-default-features`).
    let (client, _) = crate::Client::new("", Default::default()).unwrap();
    let (send, recv) = std::sync::mpsc::sync_channel(1);
    let handler = crate::contrib::ClosureProcessHandler::new(move |_, ps| {
        send.try_send(ps.cycle_times()).ok();
        crate::Control::Continue
    });
    let ac = client.activate_async((), handler).unwrap();
    let cycle_times = recv
        .recv_timeout(std::time::Duration::from_secs(1))
        .unwrap();
    ac.deactivate().unwrap();
    let cycle_times = cycle_times.unwrap();
    assert!(cycle_times.period_usecs > 0.0, "{:?}", cycle_times);
}
