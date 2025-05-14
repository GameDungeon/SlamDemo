use flate2::write::GzEncoder;
use flate2::Compression;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt, join};
use log::*;
use tokio_tungstenite::WebSocketStream;
use std::fmt::Debug;
use std::io::Write;
use std::{net::SocketAddr, time::Duration};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast::{self, Sender, Receiver};
use tokio_tungstenite::{
    accept_async,
    tungstenite::{Error, Message, Result},
};

#[derive(Clone, Debug)]
pub enum ServerMessage {
    Ping,
    Point,
    PoseUpdate,
    MapUpdate {x: i64, y: i64, img: Box<Vec<u8>>}
}

async fn accept_connection(peer: SocketAddr, stream: TcpStream, message_broadcast: Sender<ServerMessage>) {
    if let Err(e) = handle_connection(peer, stream, message_broadcast).await {
        match e {
            Error::ConnectionClosed | Error::Protocol(_) | Error::Utf8 => (),
            err => error!("Error processing connection: {}", err),
        }
    }
}

async fn handle_connection(peer: SocketAddr, stream: TcpStream, message_broadcast: Sender<ServerMessage>) -> Result<()> {
    let ws_stream = accept_async(stream).await.expect("Failed to accept");
    info!("New WebSocket connection: {}", peer);
    let (ws_sender,
        ws_receiver) = ws_stream.split();
    
    join!(
        handle_outgoing(message_broadcast.subscribe(), ws_sender),
        handle_incoming(message_broadcast, ws_receiver)
    );

    Ok(())
}

async fn handle_incoming(message_broadcast: Sender<ServerMessage>,
    server_reciver: SplitStream<WebSocketStream<TcpStream>>) {
    
}

async fn handle_outgoing(mut message_receiver: Receiver<ServerMessage>,
    mut server_sender: SplitSink<WebSocketStream<TcpStream>, Message>) {

    loop {
        let msg = message_receiver.recv().await;
        match msg.unwrap() {
            ServerMessage::Ping => server_sender.send(Message::Text("Ping".into())).await.unwrap(),
            ServerMessage::MapUpdate { x, y, img } => {
                let mut bin = Vec::new();
                let mut e = GzEncoder::new(&mut bin, Compression::default());
                e.write_all(b"MU").unwrap();
                e.write_all(&x.to_be_bytes()).unwrap();
                e.write_all(&y.to_be_bytes()).unwrap();
                e.write_all(&img);
                e.finish().unwrap();

                server_sender.send(Message::binary(bin)).await.unwrap();
            },
            _ => ()
        }
    }
    
}

pub async fn server_main(message_broadcast: Sender<ServerMessage>) {
    let addr = "127.0.0.1:7890";
    let listener = TcpListener::bind(&addr).await.expect("Can't listen");
    info!("Listening on: {}", addr);

    while let Ok((stream, _)) = listener.accept().await {
        let peer = stream.peer_addr().expect("connected streams should have a peer address");
        info!("Peer address: {}", peer);

        tokio::spawn(accept_connection(peer, stream, message_broadcast.clone()));
    }
}


