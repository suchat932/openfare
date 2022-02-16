use anyhow::{format_err, Context, Result};
use serde::Serialize;
use std::io::Write;
use structopt::{self, StructOpt};

#[derive(Debug)]
pub struct LockFileHandle {
    pub lock: openfare_lib::lock::Lock,
    lock_hash: Option<blake3::Hash>,
    path: std::path::PathBuf,
}

impl LockFileHandle {
    pub fn new(user_lock_file_path: &Option<std::path::PathBuf>, force: bool) -> Result<Self> {
        let path = if let Some(user_lock_file_path) = user_lock_file_path {
            user_lock_file_path.clone()
        } else {
            let working_directory = std::env::current_dir()?;
            working_directory.join(openfare_lib::lock::FILE_NAME)
        };

        if path.exists() && !force {
            return Err(format_err!(
                "File already exists and --force flag not given. Exiting.\n{}",
                path.display()
            ));
        }

        Ok(Self {
            lock: openfare_lib::lock::Lock::default(),
            lock_hash: None,
            path,
        })
    }

    pub fn load(user_lock_file_path: &Option<std::path::PathBuf>) -> Result<Self> {
        let path = user_lock_file_path.clone().or(find_lock_file()?);
        if let Some(path) = path {
            let lock = from_file(&path)?;
            let lock_hash = Some(Self::get_lock_hash(&lock)?);
            let lock_handle = Self {
                lock,
                lock_hash,
                path,
            };
            println!("load lock_handle: {:?}", lock_handle);
            Ok(lock_handle)
        } else {
            Err(format_err!(
                "Filed to find lock file. Provide path or change working directory."
            ))
        }
    }

    pub fn path(&self) -> &std::path::PathBuf {
        &self.path
    }

    fn get_lock_hash(lock: &openfare_lib::lock::Lock) -> Result<blake3::Hash> {
        let serialized_lock = serde_json::to_string(&lock)?;
        Ok(blake3::hash(&serialized_lock.as_bytes()))
    }
}

impl Drop for LockFileHandle {
    fn drop(&mut self) {
        // Skip writing lock if unchanged.
        println!("self.lock: {:?}", self.lock);
        let current_lock_hash = Self::get_lock_hash(&self.lock).expect("current lock hash");
        println!("current_lock_hash: {:?}", current_lock_hash);
        println!("self.lock_hash: {:?}", self.lock_hash);
        if let Some(lock_hash) = self.lock_hash {
            if current_lock_hash == lock_hash {
                log::debug!("Lock file unchanged. Not writing to file.");
                return ();
            }
        }
        log::debug!("Lock file changed. Writing to file.");
        if self.path.is_file() {
            std::fs::remove_file(&self.path).unwrap_or_default();
        }

        let buf = Vec::new();
        let formatter = serde_json::ser::PrettyFormatter::with_indent(b"    ");
        let mut serializer = serde_json::Serializer::with_formatter(buf, formatter);
        self.lock.serialize(&mut serializer).unwrap();

        let lock_json = String::from_utf8(serializer.into_inner()).unwrap();

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .append(false)
            .create(true)
            .open(&self.path)
            .context(format!(
                "Can't open/create file for writing: {}",
                self.path.display()
            ))
            .unwrap();

        file.write_all(lock_json.as_bytes())
            .expect("Unable to write data");
    }
}

fn find_lock_file() -> Result<Option<std::path::PathBuf>> {
    let working_directory = std::env::current_dir()?;

    assert!(working_directory.is_absolute());
    let mut working_directory = working_directory.clone();

    loop {
        let target_absolute_path = working_directory.join(openfare_lib::lock::FILE_NAME);
        if target_absolute_path.is_file() {
            return Ok(Some(target_absolute_path));
        }

        // No need to move further up the directory tree after this loop.
        if working_directory == std::path::PathBuf::from("/") {
            break;
        }

        // Move further up the directory tree.
        working_directory.pop();
    }
    Ok(None)
}

fn from_file(path: &std::path::PathBuf) -> Result<openfare_lib::lock::Lock> {
    let file = std::fs::File::open(&path)?;
    let reader = std::io::BufReader::new(file);
    let lock: openfare_lib::lock::Lock = serde_json::from_reader(reader)?;
    Ok(lock)
}

#[derive(Debug, StructOpt, Clone)]
pub struct LockFilePathArg {
    /// Lock file path. Searches in current working directory if not given.
    #[structopt(name = "lock-file-path", long, short = "p")]
    pub path: Option<std::path::PathBuf>,
}
