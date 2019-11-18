
use std::path::{Path, PathBuf};
use std::io;
use std::io::Read;
use std::fs::{OpenOptions, File};



#[derive(Debug)]
pub struct Resource {

    root: PathBuf,
    
}

impl Default for Resource {

    fn default() -> Self {
        use std::env;
        #[cfg(not(debug_assertions))]
        let root = env::current_exe().map(|path: PathBuf| {
            path.parent().unwrap().to_path_buf()
        })
        .unwrap();
        #[cfg(debug_assertions)]
        let root = env::var("CARGO_MANIFEST_DIR").map(|s: String| {
            let mut root = PathBuf::from(s);
            //root.push("src");
            root
        })
        .unwrap();
        Resource { root }
    }

}



impl Resource {

    pub fn join(&self, file: &str) -> PathBuf {
        let fpath = Path::new(file);
        if fpath.is_absolute() {
            fpath.to_path_buf()
        } else {
            let mut path = self.root.clone();
            for p in fpath.components() {
                path.push(p);
            }       
            path
        }
    }

    pub fn open(&self, file: &str, options: OpenOptions) -> io::Result<File> {
        options.open(self.join(file))
    }

    pub fn open_read_only(&self, file: &str) -> io::Result<File> {
        OpenOptions::new().read(true).open(self.join(file))
    }


    pub fn load_as_bytes(&self, file: &str) -> io::Result<Vec<u8>> {
        let path = self.join(file);
        let mut ifile = OpenOptions::new().read(true).open(path)?;
        let mut buf: Vec<u8> = Vec::new();
        ifile.read_to_end(&mut buf)?;
        Ok(buf)
    }

    pub fn load_as_string(&self, file: &str) -> io::Result<String> {
        let path = self.join(file);
        let mut ifile = OpenOptions::new().read(true).open(path)?;
        let mut buf: String = String::new();
        ifile.read_to_string(&mut buf)?;
        Ok(buf)
    }
}

use std::time::Duration;

const NANOS_PER_SEC: u64 = 1_000_000_000;

pub fn duration_to_nanos(d: Duration) -> u64 {
    d.as_secs() as u64 * NANOS_PER_SEC + d.subsec_nanos() as u64
}