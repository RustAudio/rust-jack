#[test]
fn can_open_client() {
    let (client, status) =
        crate::Client::new("my new client", crate::ClientOptions::empty()).unwrap();
    assert_eq!(status, crate::ClientStatus::empty());
    assert_eq!(client.name(), "my new client");
}
