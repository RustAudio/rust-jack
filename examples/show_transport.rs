use std::io;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

fn main() {
    // Create client
    let (client, _status) =
        jack::Client::new("rust_jack_trans", jack::ClientOptions::default()).unwrap();

    let transport = client.transport();
    let stop = Arc::new(AtomicBool::new(false));

    let show = {
        let stop = stop.clone();
        std::thread::spawn(move || {
            let mut state_last: Option<jack::TransportStatePosition> = None;
            while !stop.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(10));
                if let Ok(state) = transport.query() {
                    //print if state has changed
                    if state_last
                        .as_ref()
                        .map_or_else(|| true, |l| l.state != state.state)
                    {
                        println!("{:?}", state.state);
                    };
                    //print if bbt structure changes (ignoring tick and bar_start_tick)
                    if let Some(mut bbt) = state.pos.bbt() {
                        let print = if let Some(last) = &state_last {
                            if let Some(mut last) = last.pos.bbt() {
                                //zero out some details so we can just compare directly
                                bbt.tick = 0;
                                bbt.bar_start_tick = 0.0;
                                last.tick = 0;
                                last.bar_start_tick = 0.0;
                                bbt != last
                            } else {
                                true
                            }
                        } else {
                            true
                        };
                        if print {
                            println!(
                                "bar {} beat {} bpm {} sig: {}/{} tpb {}",
                                bbt.bar,
                                bbt.beat,
                                bbt.bpm,
                                bbt.sig_num,
                                bbt.sig_denom,
                                bbt.ticks_per_beat
                            );
                        }
                    }
                    state_last = Some(state);
                } else {
                    eprint!("couldn't get state");
                    break;
                }
            }
        })
    };

    // Wait for user input to quit
    println!("Press enter/return to quit...");
    let mut user_input = String::new();
    io::stdin().read_line(&mut user_input).ok();
    stop.store(true, Ordering::Relaxed);
    show.join().expect("failed to join");
}
