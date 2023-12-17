use std::io;

use futures::{future, StreamExt, TryStreamExt};
use kanal::AsyncReceiver;
use tokio::net::{TcpListener, TcpStream, ToSocketAddrs};
use tokio_tungstenite::tungstenite::Error;

use crate::pollers::Event;

async fn handle(raw: TcpStream, rx: &AsyncReceiver<Event>) -> Result<(), Error> {
    let ws_stream = tokio_tungstenite::accept_async(raw).await?;
    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|_| future::ok(()));
    let receive_from_others = rx
        .stream()
        .map(|e| Ok(format!("{e:?}").into()))
        .forward(outgoing);

    future::select(broadcast_incoming, receive_from_others).await;

    Ok(())
}

pub async fn listen(addr: impl ToSocketAddrs, rx: AsyncReceiver<Event>) -> io::Result<()> {
    let socket = TcpListener::bind(addr).await?;

    // We only want one listener at a time :)
    while let Ok((stream, _)) = socket.accept().await {
        drop(handle(stream, &rx).await);
    }

    Ok(())
}
