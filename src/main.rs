#![windows_subsystem = "windows"]
#![feature(never_type)]
#![feature(type_alias_impl_trait)]

use std::env;

use kanal::unbounded_async;
use tray_item::{IconSource, TrayItem};

// todo: make better async mem lib :smile:
mod hdmem;
mod poll;
mod pollers;
mod util;
mod ws;

async fn tray() {
    let mut tray = TrayItem::new("Sparrow", IconSource::Resource("tray-default")).unwrap();

    tray.add_label("Sparrow").unwrap();
    tray.add_label("by fluffiac :3").unwrap();
    tray.inner_mut().add_separator().unwrap();

    let (tx, rx) = kanal::bounded(0);
    tray.add_menu_item("Quit", move || tx.send(()).unwrap())
        .unwrap();
    drop(rx.as_async().recv().await);
}

#[tokio::main]
async fn main() {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8666".to_string());

    let (tx, rx) = unbounded_async();

    let pollers = tokio::spawn(pollers::poll_all(tx));
    let ws = tokio::spawn(ws::listen(addr, rx));

    tokio::select! {
        err = pollers => println!("{err:?}"),
        err = ws => println!("{err:?}"),
        _ = tray() => (),
    }
}
