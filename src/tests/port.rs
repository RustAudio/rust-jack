use crate::{AudioIn, Client, ClientOptions, Error, LatencyType};

#[test]
fn port_methods_fail_after_client_is_dropped() {
    let (client, _) = Client::new("", ClientOptions::default()).unwrap();
    let port = client.register_port("in", AudioIn::default()).unwrap();
    let transport = client.transport();
    drop(client);

    assert_eq!(port.name(), Err(Error::ClientIsNoLongerAlive));
    assert_eq!(port.flags(), Err(Error::ClientIsNoLongerAlive));
    assert_eq!(
        port.get_latency_range(LatencyType::Capture),
        Err(Error::ClientIsNoLongerAlive)
    );
    assert_eq!(
        port.set_latency_range(LatencyType::Capture, (1, 2)),
        Err(Error::ClientIsNoLongerAlive)
    );
    assert_eq!(port.get_connections(), Err(Error::ClientIsNoLongerAlive));
    assert_eq!(
        port.get_connected_client_uuids(),
        Err(Error::ClientIsNoLongerAlive)
    );
    assert!(transport.query_state().is_err());
}

#[test]
fn port_methods_race_with_client_drop() {
    // A port used on another thread while its client is dropped must either succeed or report
    // that the client is gone. It must never call into a closed client.
    for _ in 0..20 {
        let (client, _) = Client::new("", ClientOptions::default()).unwrap();
        let port = client.register_port("in", AudioIn::default()).unwrap();
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let user = {
            let stop = stop.clone();
            std::thread::spawn(move || {
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    match port.name() {
                        Ok(name) => assert!(name.ends_with(":in"), "{}", name),
                        Err(err) => {
                            assert_eq!(err, Error::ClientIsNoLongerAlive);
                            return;
                        }
                    }
                }
            })
        };
        std::thread::sleep(std::time::Duration::from_millis(5));
        drop(client);
        std::thread::sleep(std::time::Duration::from_millis(5));
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        user.join().unwrap();
    }
}
