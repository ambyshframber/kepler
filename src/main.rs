use config::GeminiConfig;
use server::Server;
use std::env::{args, set_current_dir};
use std::path::PathBuf;

mod server;
mod config;
mod utils;

fn main() {
    let args: Vec<String> = args().collect();
    let config_path = PathBuf::from(&args[1]);
    let folder = config_path.parent().unwrap();
    if !folder.as_os_str().is_empty() {
        set_current_dir(folder).unwrap();
    }
    let config_file = config_path.file_name().unwrap().to_string_lossy().to_string();
    let cfg = GeminiConfig::new(config_file);

    let mut server = Server::new(cfg);
    server.run_server();
}
