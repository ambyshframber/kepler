use std::path::{Path, PathBuf};
use innit::IniDocument;
use std::cell::RefCell;
use std::time::Instant;
use std::fs::read_to_string;

pub struct GeminiConfig {
    ini: IniDocument,

    port: u16,

    redirects_table: RefCell<IniDocument>,
    redirects_ttl: u64,
    redirects_last_update: RefCell<Instant>,
}
impl GeminiConfig {
    pub fn new(path: impl AsRef<Path>) -> GeminiConfig {
        let ini = read_to_string(path).unwrap();
        let document = IniDocument::from_string(ini).unwrap();
        let port = document.get(PORT, "").map(|p| p.parse::<u16>().unwrap()).unwrap_or(1965);
        let redirects_ttl = document.get(REDIRECTS_TTL, "").map(|p| p.parse::<u64>().unwrap()).unwrap_or(1800);
        let redirects_last_update = RefCell::new(Instant::now());
        let redirects_table = RefCell::new(IniDocument::empty());

        let cfg = GeminiConfig {
            ini: document,
            port, redirects_last_update, redirects_table, redirects_ttl
        };
        if let Some(p) = cfg.ini.get(REDIRECTS_FILE, "") {
            cfg.update_redirects(p)
        }
        cfg
    }
    fn update_redirects(&self, path: &str) {
        match read_to_string(path) {
            Ok(ini) => {
                match IniDocument::from_string(ini) {
                    Ok(document) => { self.redirects_table.replace(document); }
                    Err(e) => eprintln!("error loading redirects table: {}", e)
                }
            }
            Err(e) => eprintln!("error loading redirects table: {}", e)
        }
    }

    pub fn port(&self) -> u16 {
        self.port
    }
    pub fn content_folder(&self) -> PathBuf {
        PathBuf::from(self.ini.get(CONTENT_FOLDER, "").unwrap_or("content"))
    }
    pub fn certificate_file(&self) -> &str {
        self.ini.get(CERT_CHAIN_FILE, "").unwrap_or("cert.pem")
    }
    pub fn private_key_file(&self) -> &str {
        self.ini.get(PRIVATE_KEY_FILE, "").unwrap_or("key.pem")
    }
    pub fn hostname(&self) -> &str {
        self.ini.get(HOSTNAME, "").unwrap_or("localhost")
    }
    pub fn index(&self) -> PathBuf {
        PathBuf::from(self.ini.get(INDEX, "").unwrap_or("index.gmi"))
    }
    pub fn check_redirect(&self, path: impl AsRef<Path>) -> Option<(String, bool)> {
        let redir_file_path = if let Some(p) = self.ini.get(REDIRECTS_FILE, "") {
            p
        }
        else {
            return None
        };
        
        let last_update = self.redirects_last_update.borrow(); // should never be mutably borrowed
        if last_update.elapsed().as_secs() > self.redirects_ttl {
            self.update_redirects(redir_file_path);
            self.redirects_last_update.replace(Instant::now());
        }

        let table = self.redirects_table.borrow();
        let path = path.as_ref().to_str()?;
        let dest = table.get_case_insensitive(REDIRECT_DESTINATION, path);
        let permanent = table.get(REDIRECT_IS_PERMANENT, path).map(|s| s.parse::<bool>().unwrap_or(false)).unwrap_or(true);
        dest.map(move |d| (d.to_string(), permanent))
    }
}

const PRIVATE_KEY_FILE: &str = "private_key_file";
const CERT_CHAIN_FILE: &str = "cert_chain_file";
const CONTENT_FOLDER: &str = "content_root";
const HOSTNAME: &str = "hostname";
const PORT: &str = "port";
const INDEX: &str = "index";

const REDIRECTS_FILE: &str = "redirects_file";
const REDIRECTS_TTL: &str = "redirects_ttl";
const REDIRECT_DESTINATION: &str = "destination";
const REDIRECT_IS_PERMANENT: &str = "permanent";
