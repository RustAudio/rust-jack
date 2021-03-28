use std::env;

fn main() {
    let (client, _status) =
        jack::Client::new("rust_jack_trans", jack::ClientOptions::NO_START_SERVER).unwrap();

    let transport = client.transport();

    let usage = || {
        let commands = [
            "start",
            "stop",
            "bpm <value>",
            "sig <numerator> <denominator>",
            "locate <bar> <beat> <tick>",
        ];
        println!("usage:");
        for cmd in commands.iter() {
            println!("\t {} {}", env::args().next().unwrap(), cmd);
        }
    };

    if env::args().len() == 1 {
        usage();
    } else {
        let jack::TransportStatePosition { mut pos, state: _ } =
            transport.query().expect("failed to query transport");

        let mut bbt = pos.bbt().unwrap_or_else(jack::TransportBBT::default);
        pos.set_frame(pos.frame() + 44100);

        let mut new_bbt = None;
        let mut args = env::args().skip(1);
        let cmd = args.next().expect("failed to get command");
        match cmd.as_str() {
            "start" => transport.start().expect("failed to start"),
            "stop" => transport.stop().expect("failed to stop"),
            "bpm" => {
                new_bbt = Some(
                    bbt.with_bpm(
                        args.next()
                            .expect("failed to get bpm value")
                            .parse::<f64>()
                            .expect("failed to convert"),
                    ),
                );
            }
            "locate" => {
                new_bbt = Some(
                    bbt.with_bbt(
                        args.next()
                            .expect("failed to get bar value")
                            .parse::<usize>()
                            .expect("failed to convert"),
                        args.next()
                            .expect("failed to get beat value")
                            .parse::<usize>()
                            .expect("failed to convert"),
                        args.next()
                            .expect("failed to get tick value")
                            .parse::<usize>()
                            .expect("failed to convert"),
                    ),
                );
            }
            "sig" => {
                new_bbt = Some(
                    bbt.with_timesig(
                        args.next()
                            .expect("failed to get timesig numerator")
                            .parse::<f32>()
                            .expect("failed to convert"),
                        args.next()
                            .expect("failed to get timesig denominator")
                            .parse::<f32>()
                            .expect("failed to convert"),
                    ),
                );
            }
            c => {
                println!("unknown command {}", c);
                usage();
                return;
            }
        };

        if let Some(bbt) = new_bbt {
            let bbt = bbt.validated().expect("transport bbt failed to validate");
            pos.set_bbt(Some(bbt)).expect("error settting bbt");
            transport.reposition(&pos).expect("failed to reposition");
        }
    }
}
