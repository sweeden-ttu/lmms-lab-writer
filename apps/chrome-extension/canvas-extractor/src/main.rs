use log::info;

mod models;
mod parser;

fn main() {
    env_logger::init();
    info!("Canvas Payload Parser starting...");
    println!("Canvas Payload Parser v0.1.0");
}
