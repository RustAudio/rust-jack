#[test]
fn client_can_open() {
    let (client, status) =
        crate::Client::new("my new client", crate::ClientOptions::empty()).unwrap();
    assert_eq!(status, crate::ClientStatus::empty());
    assert_eq!(client.name(), "my new client");
    assert_ne!(client.sample_rate(), 0);
    assert_ne!(client.buffer_size(), 0);
    assert_ne!(client.uuid_string(), "");
}

#[test]
fn time_is_montonically_increasing() {
    let (client, _) = crate::Client::new("time client", crate::ClientOptions::empty()).unwrap();

    let t0 = client.time();
    let frames0 = client.frames_since_cycle_start();
    let frame_time0 = client.frame_time();

    std::thread::sleep(std::time::Duration::from_millis(50));
    assert_ne!(client.time(), t0);
    assert_ne!(client.frames_since_cycle_start(), frames0);
    assert_ne!(client.frame_time(), frame_time0);
}

#[test]
fn frame_and_time_and_convertable() {
    let (client, _) = crate::Client::new("frame times", crate::ClientOptions::empty()).unwrap();
    let sample_rate = client.sample_rate();
    assert_eq!(
        (client.frames_to_time(1) - client.frames_to_time(0)) as f64,
        (1_000_000.0 / sample_rate as f64).round()
    );
}
