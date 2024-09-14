use approx::assert_abs_diff_eq;

#[test]
fn frame_and_time_are_convertable() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::empty()).unwrap();
    assert_eq!(client.time_to_frames(client.frames_to_time(0)), 0);
}

#[test]
fn one_frame_duration_is_inverse_of_sample_rate() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::empty()).unwrap();
    let sample_rate = client.sample_rate();
    assert_abs_diff_eq!(
        (client.frames_to_time(sample_rate as _) - client.frames_to_time(0)) as f64,
        1_000_000.0,
        epsilon = 1_000_000.0 * 1e-4,
    );
}

#[test]
fn one_second_is_sample_rate_frames() {
    let (client, _) = crate::Client::new("", crate::ClientOptions::empty()).unwrap();
    let t0 = client.time_to_frames(0);
    let t1 = client.time_to_frames(1_000_000);
    assert_abs_diff_eq!(
        (t1 - t0) as f64,
        client.sample_rate() as f64,
        epsilon = client.sample_rate() as f64 * 1e-5
    );
}
