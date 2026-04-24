use log::info;

mod models;
mod parser;
mod logger;
mod downloader;
mod extractor;
mod videos;

#[tokio::main]
async fn main() {
    env_logger::init();
    logger::log_startup();
    info!("Canvas Payload Parser starting...");
    println!("Canvas Payload Parser v0.1.0");
}
