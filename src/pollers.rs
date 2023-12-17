use kanal::{AsyncSender, SendError};
use tokio::time::{sleep, Duration};

use crate::hdmem::{self, *};
use crate::poll::*;

#[derive(Debug)]
pub enum Event {
    FindingGame,
    GameConnected,
    GameDisconnected(hdmem::MemError),
    Pace(Vec<f32>),
}

struct Pace;

impl Poll for Pace {
    async fn poll(mem: &Mem, tx: AsyncSender<Event>) -> Result<!> {
        let offsets = [mem.base, 0x385790, 0x00, 0x78, 0xb8];

        let paceaddr = loop {
            if let Some(poo) = mem.offsets(offsets).await? {
                break poo + 0x228;
            }
        };

        let mut pace: Vec<f32> = Vec::with_capacity(155);

        loop {
            let len = mem.read_val::<u32>(paceaddr + 0x18).await? as usize;

            if len != pace.len() {
                let vecloc = mem.read_addr(paceaddr).await?;
                mem.read_into_vec(vecloc, &mut pace, len).await?;

                drop(tx.send(Event::Pace(pace.clone())).await);
            }

            sleep(Duration::from_micros(8333)).await;
        }
    }
}

pub async fn poll_all(tx: AsyncSender<Event>) -> std::result::Result<!, SendError> {
    let mut runner = PollRunner::new();

    runner.add(Pace);

    loop {
        tx.send(Event::FindingGame).await?;

        match Mem::init() {
            Ok(mem) => {
                tx.send(Event::GameConnected).await?;
                let err = runner.poll(&mem, &tx).await;
                tx.send(Event::GameDisconnected(err)).await?;
            }
            _ => (),
        };

        sleep(Duration::from_secs(5)).await;
    }
}
