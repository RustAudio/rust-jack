use approx::assert_abs_diff_eq;

use crate::contrib::ClosureProcessHandler;

fn active_client() -> crate::AsyncClient<(), impl crate::ProcessHandler> {
    let (client, _) = crate::Client::new("", crate::ClientOptions::empty()).unwrap();
    let (send, recv) = std::sync::mpsc::sync_channel(1);
    let ac = client
        .activate_async(
            (),
            ClosureProcessHandler::new(move |_, _| {
                send.try_send(()).ok();
                crate::Control::Continue
            }),
        )
        .unwrap();
    recv.recv_timeout(std::time::Duration::from_secs(1))
        .unwrap();
    ac
}

#[test]
fn frame_and_time_are_convertable() {
    let ac = active_client();
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
    let ac = active_client();
    let client = ac.as_client();
    let sample_rate = client.sample_rate();
    assert_abs_diff_eq!(
        (client.frames_to_time(sample_rate as _) - client.frames_to_time(0)) as f64,
        1_000_000.0,
        epsilon = 1_000_000.0 * 1e-3,
    );
}

#[test]
fn one_second_is_sample_rate_frames() {
    let ac = active_client();
    let client = ac.as_client();
    let now = client.time();
    let t0 = client.time_to_frames(now);
    let t1 = client.time_to_frames(now + 1_000_000);
    assert_abs_diff_eq!(
        (t1 - t0) as f64,
        client.sample_rate() as f64,
        epsilon = client.sample_rate() as f64 * 1e-3
    );
}
