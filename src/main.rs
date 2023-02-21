use nokhwa::pixel_format::RgbFormat;
use nokhwa::query;
use nokhwa::utils::ApiBackend;
use nokhwa::utils::RequestedFormat;
use nokhwa::utils::RequestedFormatType;
use nokhwa::CallbackCamera;
use std::io::prelude::*;
use std::net::TcpListener;
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::Mutex;

fn main() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();

    let connections: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(vec![]));

    let cameras = query(ApiBackend::Auto).unwrap();
    cameras.iter().for_each(|cam| println!("{cam:?}"));

    let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate);

    let first_camera = cameras.first().unwrap();

    let camera_connections = connections.clone();

    let mut threaded = CallbackCamera::new(first_camera.index().clone(), format, move |buffer| {
        let image_buffer = [b"--frame\nContent-Type: image/jpeg\n\n", buffer.buffer()].concat();
        camera_connections
            .lock()
            .unwrap()
            .iter()
            .for_each(|mut stream| {
                stream.write_all(&image_buffer).unwrap();
            })
    })
    .unwrap();
    threaded.open_stream().unwrap();

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream, connections.clone());
    }

    println!("Shutting down.");
}

fn handle_connection(mut stream: TcpStream, connections: Arc<Mutex<Vec<TcpStream>>>) {
    let messgage = b"HTTP/1.1 200 OK\nContent-Type: multipart/x-mixed-replace; boundary=frame\n\n";
    stream.write_all(messgage).unwrap();
    connections.lock().unwrap().push(stream);
}
