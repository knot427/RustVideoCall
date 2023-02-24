use nokhwa::pixel_format::RgbFormat;
use nokhwa::Camera;

use nokhwa::utils::CameraIndex;
use nokhwa::utils::RequestedFormat;
use nokhwa::utils::RequestedFormatType;

use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::time::Duration;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();

    let (tx, rx): (Sender<TcpStream>, Receiver<TcpStream>) = mpsc::channel();

    thread::spawn(move || {
        let mut connections: Vec<TcpStream> = vec![];

        loop {
            println!("Waiting for connection");
            match rx.recv() {
                Ok(stream) => {
                    connections.push(stream);
                }
                Err(_) => {
                    println!("thread failed to recive tcp connection");
                    break;
                }
            };
            println!("Starting camera");
            let index = CameraIndex::Index(0);
            // request the absolute highest resolution CameraFormat that can be decoded to RGB.
            let requested =
                RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

            // make the camera
            let mut camera = match Camera::new(index, requested) {
                Ok(cam) => cam,
                Err(_) => {
                    println!("Failed to start camera");
                    continue;
                }
            };

            loop {
                let frame = match camera.frame() {
                    Ok(frame) => frame,
                    Err(_) => break,
                };

                let frame = [b"--frame\r\nContent-Type: image/jpeg\r\n\r\n", frame.buffer(), b"\r\n"].concat();

                connections = connections
                    .into_iter()
                    .filter_map(|mut stream| match stream.write_all(&frame) {
                        Ok(_) => Option::Some(stream),
                        Err(_) => Option::None,
                    })
                    .collect();

                if let Ok(stream) = rx.recv_timeout(Duration::from_millis(1)) {
                    connections.push(stream)
                };
                if connections.is_empty() {
                    println!("no active connections, closing camera");
                    break;
                }
            }
        }
    });

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let messgage =
            b"HTTP/1.1 200 OK\nContent-Type: multipart/x-mixed-replace; boundary=frame\n\n";
        match stream.write_all(messgage) {
            Ok(_) => {}
            Err(_) => continue,
        };
        match tx.send(stream) {
            Ok(_) => {}
            Err(_) => {
                println!("failed to send connection to thread");
                break;
            }
        };
    }

    println!("Shutting down.");
}
