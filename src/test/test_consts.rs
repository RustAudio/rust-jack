use super::super::*;

#[test]
fn valid_client_name_size() {
    assert!(*CLIENT_NAME_SIZE > 0);
}

#[test]
fn valid_port_name_size() {
    assert!(*PORT_NAME_SIZE > 0);
}

#[test]
fn valid_port_type_size() {
    assert!(*PORT_TYPE_SIZE > 0);
}
