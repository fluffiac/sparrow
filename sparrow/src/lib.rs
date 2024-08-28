mod json;
mod pipe;

use std::time::Duration;

use json::*;
use pipe::pipe;

use hdmem::{HdMemReader, Proc};

/// Busy loop that manages connection to the game process and reading its memory.
async fn mem_read(chan: kanal::Sender<String>) {
    loop {
        let proc = loop {
            match Proc::with_name("hyperdemon.exe").await {
                Ok(proc) => break proc,
                Err(_) => (),
            };
            Timer::after(Duration::from_secs(5)).await;
        };

        let _ = chan.send(String::from(CONNECTED));

        let err = HdMemReader::new()
            .on_queue_ids_update(|f| {
                print!("Queued IDs:");
                for id in f {
                    if *id == 0 { continue };
                    print!(" {id}")
                }
                println!();
                println!("Queue Length: {}", f.iter().filter(|id| **id != 0).count());
            })
            .on_multiplayer_score_update(|f| {
                println!("Score: {} - {}", f.0, f.1);
            })
            // .on_pace_update(pipe(pace).to(&chan))
            // .on_training_info_update(pipe(debug).to(&chan))
            .read_with(proc)
            .run_on_threads(1);

        println!("{err:?}");

        let _ = chan.send(String::from(DISCONNECTED));
    }
}

use async_io::{block_on, Timer};

pub fn main() {
    let (chan, recv) = kanal::bounded(1);

    std::thread::spawn(|| block_on(mem_read(chan)));

    for event in recv {
        println!("{}", event);
    }
}
