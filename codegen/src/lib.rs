use std::borrow::{Borrow, Cow};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::{env, fmt, io};

use walkdir::WalkDir;

use flate2::write::GzEncoder;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Compression {
    None,
    Gzip,
}

impl fmt::Display for Compression {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Compression::None => fmt.write_str("None"),
            Compression::Gzip => fmt.write_str("Gzip"),
        }
    }
}

pub struct IncludeDir {
    files: HashMap<String, (Compression, PathBuf)>,
    name: String,
    manifest_dir: PathBuf,
}

pub fn start(static_name: &str) -> IncludeDir {
    IncludeDir {
        files: HashMap::new(),
        name: static_name.to_owned(),
        manifest_dir: Path::new(
            &env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set"),
        )
        .to_owned(),
    }
}

#[cfg(windows)]
fn as_key(path: &str) -> Cow<str> {
    Cow::Owned(path.replace("\\", "/"))
}

#[cfg(not(windows))]
fn as_key(path: &str) -> Cow<str> {
    Cow::Borrowed(path)
}

impl IncludeDir {
    /// Add a single file to the binary.
    /// With Gzip compression, the file will be encoded to OUT_DIR first.
    /// For chaining, it's not sensible to return a Result. If any to-be-included
    /// files can't be found, or encoded, this function will panic!.
    pub fn file<P: AsRef<Path>>(mut self, path: P, comp: Compression) -> IncludeDir {
        self.add_file(path, comp).unwrap();
        self
    }

    /// ## Panics
    ///
    /// This function panics when CARGO_MANIFEST_DIR or OUT_DIR are not defined.
    pub fn add_file<P: AsRef<Path>>(&mut self, path: P, comp: Compression) -> io::Result<()> {
        let key = path.as_ref().to_string_lossy();
        println!("cargo:rerun-if-changed={}", path.as_ref().display());

        match comp {
            Compression::None => {
                self.files.insert(
                    as_key(key.borrow()).into_owned(),
                    (comp, path.as_ref().to_owned()),
                );
            }
            Compression::Gzip => {
                // gzip encode file to OUT_DIR
                let in_path = self.manifest_dir.join(&path);
                let mut in_file = BufReader::new(File::open(&in_path)?);

                let path_without_root = if path.as_ref().is_absolute() {
                    path.as_ref()
                        .components()
                        .filter(|c| match c {
                            std::path::Component::Prefix(_) => false,
                            std::path::Component::RootDir => false,
                            _ => true,
                        })
                        .collect::<PathBuf>()
                } else {
                    path.as_ref().to_path_buf()
                };

                let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join(&path_without_root);
                fs::create_dir_all(&out_path.parent().unwrap())?;
                let out_file = BufWriter::new(File::create(&out_path)?);
                let mut encoder = GzEncoder::new(out_file, flate2::Compression::best());

                io::copy(&mut in_file, &mut encoder)?;

                self.files
                    .insert(as_key(key.borrow()).into_owned(), (comp, out_path));
            }
        }
        Ok(())
    }

    /// Add a whole directory recursively to the binary.
    /// This function calls `file`, and therefore will panic! on missing files.
    pub fn dir<P: AsRef<Path>>(mut self, path: P, comp: Compression) -> IncludeDir {
        self.add_dir(path, comp).unwrap();
        self
    }

    /// ## Panics
    ///
    /// This function panics when CARGO_MANIFEST_DIR or OUT_DIR are not defined.
    pub fn add_dir<P: AsRef<Path>>(&mut self, path: P, comp: Compression) -> io::Result<()> {
        for entry in WalkDir::new(path).follow_links(true).into_iter() {
            match entry {
                Ok(ref e) if !e.file_type().is_dir() => {
                    self.add_file(e.path(), comp)?;
                }
                _ => (),
            }
        }
        Ok(())
    }

    pub fn build(self, out_name: &str, filter: Vec<String>) -> io::Result<()> {
        let out_path = Path::new(&env::var("OUT_DIR").unwrap()).join(out_name);
        let mut out_file = BufWriter::new(File::create(&out_path)?);

        writeln!(
            &mut out_file,
            "#[allow(clippy::unreadable_literal, missing_docs)]"
        )?;

        writeln!(
            &mut out_file,
            "pub static {}: ::tauri_includedir::Files = ::tauri_includedir::Files {{\n\
                    files:  ",
            self.name
        )?;

        let mut map: phf_codegen::Map<&str> = phf_codegen::Map::new();
        let entries: Vec<_> = self
            .files
            .iter()
            .map(|(name, (compression, path))| {
                let include_path = format!("{}", self.manifest_dir.join(path).display());
                (
                    as_key(&name).to_string(),
                    (compression, as_key(&include_path).to_string()),
                )
            })
            .collect();

        for (name, (compression, include_path)) in &entries {
            if !filter.iter().any(|value| include_path.ends_with(value)) {
                let include_path = format!("{}", self.manifest_dir.join(include_path).display());
                map.entry(
                    name,
                    &format!(
                        "(::tauri_includedir::Compression::{}, \
                                    include_bytes!(\"{}\") as &'static [u8])",
                        compression, include_path
                    ),
                );
            }
        }

        writeln!(&mut out_file, "{}}};", map.build())?;
        Ok(())
    }
}
