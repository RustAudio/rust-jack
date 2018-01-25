//! Creates a jack midi input and output ports. The application prints
//! out all values sent to it through the input port. It also sends a
//! Note On and Off event, once every cycle, on the output port.
extern crate jack;
use std::io;

fn main() {
    // open client
    let (client, _status) =
        jack::Client::new("rust_jack_show_midi", jack::ClientOptionsNO_START_SERVER).unwrap();

    // process logic
    let mut maker = client
        .register_port("rust_midi_maker", jack::MidiOut::default())
        .unwrap();
    let shower = client
        .register_port("rust_midi_shower", jack::MidiIn::default())
        .unwrap();
    let cback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let show_p = shower.iter(ps);
        for e in show_p {
            println!("{:?}", e);
        }
        let mut put_p = maker.writer(ps);
        put_p
            .write(&jack::RawMidi {
                time: 0,
                bytes: &[
                    0b10010000 /* Note On, channel 1 */, 0b01000000 /* Key number */,
                    0b01111111 /* Velocity */,
                ],
            })
            .unwrap();
        put_p
            .write(&jack::RawMidi {
                time: ps.n_frames() / 2,
                bytes: &[
                    0b10000000 /* Note Off, channel 1 */, 0b01000000 /* Key number */,
                    0b01111111 /* Velocity */,
                ],
            })
            .unwrap();
        jack::Control::Continue
    };

    // activate
    let active_client = client
        .activate_async((), jack::ClosureProcessHandler::new(cback))
        .unwrap();

    // wait
    println!("Press any key to quit");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    // optional deactivation
    active_client.deactivate().unwrap();
}
