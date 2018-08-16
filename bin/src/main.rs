extern crate failure;
extern crate libloading;

use std::fs;
use std::time::{Duration, SystemTime};

use failure::Error;
use libloading::{Library, Symbol};

const LIB_PATH: &str = "./target/debug/liblib.dylib";

struct Lib {
    path: String,
    lib: Option<Library>,
    modified: SystemTime,
}

impl Lib {
    pub fn load(path: &str) -> Result<Lib, Error> {
        let lib = Library::new(LIB_PATH)?;
        let metadata = fs::metadata(path)?;
        let modified = metadata.modified()?;
        Ok(Lib {
            path: path.to_string(),
            lib: Some(lib),
            modified,
        })
    }

    pub unsafe fn get<T>(&self, symbol: &[u8]) -> Result<Symbol<T>, Error> {
        Ok(self.lib.as_ref().unwrap().get(symbol)?)
    }

    pub fn is_modified(&self) -> Result<bool, Error> {
        let metadata = fs::metadata(&self.path)?;
        let modified = metadata.modified()?;

        Ok(modified > self.modified)
    }

    pub fn reload(&mut self) -> Result<(), Error> {
        self.lib = None; // force unload lib
        let lib = Library::new(&self.path)?;
        let metadata = fs::metadata(&self.path)?;
        let modified = metadata.modified()?;
        self.lib = Some(lib);
        self.modified = modified;
        Ok(())
    }
}

fn main() -> Result<(), Error> {
    let mut lib = Lib::load(LIB_PATH)?;

    loop {
        if lib.is_modified()? {
            println!("Reloading...");
            lib.reload()?;
        }

        unsafe {
            let func: Symbol<fn(&str)> = lib.get(b"run")?;
            func("Main");
        }

        ::std::thread::sleep(Duration::from_millis(1000));
    }
}
