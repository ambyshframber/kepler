use openssl::ssl::{SslMethod, SslAcceptor, SslStream, SslFiletype};
use std::net::{TcpListener, TcpStream};
use std::sync::{Mutex, Arc};
use std::io::{Read, Write};
use crate::config::GeminiConfig;
use crate::utils::*;
use percent_encoding::percent_decode_str;
use std::path::{Path, PathBuf};
use mime_guess::from_path;
use std::fs::read;
use threadpool::ThreadPool;
//use std::thread::sleep;
//use std::time::Duration;

pub struct Server {
    listener: TcpListener,
    acceptor: Arc<SslAcceptor>,

    config: Arc<Mutex<GeminiConfig>>,

    threads: ThreadPool,
    cur_job: u8
}

impl Server {
    pub fn new(config: GeminiConfig) -> Server {
        let mut acceptor = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        acceptor.set_private_key_file(config.private_key_file(), SslFiletype::PEM).unwrap();
        acceptor.set_certificate_chain_file(config.certificate_file()).unwrap();
        acceptor.check_private_key().unwrap();
        let acceptor = Arc::new(acceptor.build());

        let listener = TcpListener::bind(("0.0.0.0", config.port())).unwrap();
        let threads = ThreadPool::new(4);
        let config = Arc::new(Mutex::new(config));

        Server {
            listener, acceptor, config, threads, cur_job: 0
        }
    }
    pub fn run_server(&mut self) {
        loop {
            let stream = self.listener.accept();
            if let Ok((stream, socket)) = stream {
                println!("connection from {}", socket);
                let acceptor = self.acceptor.clone();
                let accepted_stream = acceptor.accept(stream);
                if let Ok(stream) = accepted_stream {
                    self.handle_connection(stream);
                    self.cur_job = self.cur_job.wrapping_add(1);
                }
            }
        }
    }

    fn handle_connection(&self, mut stream: SslStream<TcpStream>) {
        let mut buf = [0; 1026];
        let _ = stream.read(&mut buf);
        let request = String::from_utf8_lossy(&buf).trim_matches('\0').to_string();

        let id = self.cur_job;
        let c_config = Arc::clone(&self.config);
        self.threads.execute(move || {
            let worker = Worker::new(c_config, id);
            let response = worker.process_request(request);
            let _ = match response {
                Ok((mime, data)) => {
                    let header = format!("20 {}\r\n", mime);
                    let _ = stream.write(header.as_bytes());
                    stream.write(&data)
                }
                Err(e) => stream.write(e.to_string().as_bytes())
            };
            let _ = stream.flush();
        });
    }
}

struct Worker {
    config: Arc<Mutex<GeminiConfig>>,
    id: u8
}
impl Worker {
    pub fn new(config: Arc<Mutex<GeminiConfig>>, id: u8) -> Worker {
        Worker {
            config, id
        }
    }
    pub fn log(&self, msg: &str) {
        println!("[{:02x}] {}", self.id, msg)
    }

    pub fn process_request(&self, request: String) -> Result<(String, Vec<u8>), GeminiError> {
        self.log(&format!("got request {:?}", request.trim()));
        //sleep(Duration::from_secs(5));
        let path = self.process_url(&request)?;
        let path = normalise_path(path);
        let conf_lock = self.config.lock().map_err(|_| GeminiError::temporary_failure("internal server error"))?;
        if let Some((redir, perm)) = conf_lock.check_redirect(&path) {
            self.log(&format!("\tredirecting to {:?}", redir));
            return Err(GeminiError::redirect(&redir, perm))
        }
        drop(conf_lock);
        let path = self.pre_postfix_path(path)?;
        self.log(&format!("\tnormalised to: {:?}", path));
        if !path.exists() {
            return Err(GeminiError::not_found())
        }
        let mime_type = from_path(&path).first_or_octet_stream().to_string();
        let data = read(path).map_err(|_| GeminiError::temporary_failure("internal server error"))?;
        Ok((mime_type, data))
    }
    // checks the url for validity and safety etc, then returns the path
    fn process_url(&self, u: &str) -> Result<String, GeminiError> {
        let url: Uri = Uri::new(u)?;

        let conf_lock = self.config.lock().map_err(|_| GeminiError::temporary_failure("internal server error"))?;
        let server_hostname = conf_lock.hostname();
        if url.hostname != server_hostname {
            return Err(GeminiError::bad_request(""))
        }

        let path = percent_decode_str(url.path).decode_utf8().map_err(|_| GeminiError::bad_request(""))?.to_string();
        self.log(&format!("\trequest path: {:?}", path));
        Ok(path)
    }

    fn pre_postfix_path(&self, p: impl AsRef<Path>) -> Result<PathBuf, GeminiError> {
        let conf_lock = self.config.lock().map_err(|_| GeminiError::temporary_failure("internal server error"))?;
        let content_root = conf_lock.content_folder();
        let path = content_root.join(p.as_ref());
        Ok(if path.is_dir() {
            let default_file = conf_lock.index();
            path.join(default_file)
        }
        else {
            path
        })
    }
}
