//! Creates a jack midi input and output ports. The application prints
//! out all values sent to it through the input port. It also sends a
//! Note On and Off event, once every cycle, on the output port.
use std::convert::From;
use std::io;
use std::sync::mpsc::sync_channel;

const MAX_MIDI: usize = 3;

//a fixed size container to copy data out of real-time thread
#[derive(Copy, Clone)]
struct MidiCopy {
    len: usize,
    data: [u8; MAX_MIDI],
    time: jack::Frames,
}

impl From<jack::RawMidi<'_>> for MidiCopy {
    fn from(midi: jack::RawMidi<'_>) -> Self {
        let len = std::cmp::min(MAX_MIDI, midi.bytes.len());
        let mut data = [0; MAX_MIDI];
        data[..len].copy_from_slice(&midi.bytes[..len]);
        MidiCopy {
            len,
            data,
            time: midi.time,
        }
    }
}

impl std::fmt::Debug for MidiCopy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Midi {{ time: {}, len: {}, data: {:?} }}",
            self.time,
            self.len,
            &self.data[..self.len]
        )
    }
}

fn main() {
    // open client
    let (client, _status) =
        jack::Client::new("rust_jack_show_midi", jack::ClientOptions::NO_START_SERVER).unwrap();

    //create a sync channel to send back copies of midi messages we get
    let (sender, receiver) = sync_channel(64);

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
            let c: MidiCopy = e.into();
            let _ = sender.try_send(c);
        }
        let mut put_p = maker.writer(ps);
        put_p
            .write(&jack::RawMidi {
                time: 0,
                bytes: &[
                    0b10010000, /* Note On, channel 1 */
                    0b01000000, /* Key number */
                    0b01111111, /* Velocity */
                ],
            })
            .unwrap();
        put_p
            .write(&jack::RawMidi {
                time: ps.n_frames() / 2,
                bytes: &[
                    0b10000000, /* Note Off, channel 1 */
                    0b01000000, /* Key number */
                    0b01111111, /* Velocity */
                ],
            })
            .unwrap();
        jack::Control::Continue
    };

    // activate
    let active_client = client
        .activate_async((), jack::ClosureProcessHandler::new(cback))
        .unwrap();

    //spawn a non-real-time thread that prints out the midi messages we get
    std::thread::spawn(move || {
        while let Ok(m) = receiver.recv() {
            println!("{:?}", m);
        }
    });

    // wait
    println!("Press any key to quit");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();

    // optional deactivation
    active_client.deactivate().unwrap();
}
