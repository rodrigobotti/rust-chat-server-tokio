use std::thread;

use tokio::{
    self,
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let parallelism = thread::available_parallelism().unwrap().get();
    let (tx, _rx) = broadcast::channel(parallelism);

    println!("TCP server started");

    loop {
        let (mut socket, addr) = listener.accept().await.unwrap();
        println!("Client connected");

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);

            loop {
                let mut line = String::new();
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        let bytes_read = result.unwrap();
                        if bytes_read == 0 {
                            println!("Client disconnected");
                            break;
                        }
                        let msg = format!("{addr}> {line}");
                        let message = (msg, addr);
                        tx.send(message).unwrap();
                    }
                    result = rx.recv() => {
                        let (msg, from_addr) = result.unwrap();
                        if from_addr != addr {
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}
