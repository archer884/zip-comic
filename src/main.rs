use std::{
    borrow::Cow,
    fs::{self, File},
    io,
    path::Path,
};

use clap::Clap;
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

/// comic reader archive creator
///
/// Note: this zip utility only works for flat archives.
#[derive(Clap, Clone, Debug)]
struct Opts {
    /// directory to be zipped
    path: String,
    /// destination zip archive
    output: Option<String>,

    /// remove folder when done
    #[clap(short, long)]
    force: bool,
}

impl Opts {
    fn source(&self) -> &Path {
        self.path.as_ref()
    }

    fn destination(&self) -> Cow<Path> {
        self.output
            .as_ref()
            .map(|path| Cow::Borrowed(path.as_ref()))
            .unwrap_or_else(|| {
                let path: &Path = self.path.as_ref();
                Cow::from(path.with_extension("zip"))
            })
    }
}

fn main() {
    let opts = Opts::parse();
    if let Err(e) = run(&opts) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn run(opts: &Opts) -> anyhow::Result<()> {
    let directory = fs::read_dir(opts.source())?;
    let mut archive = File::create(opts.destination()).map(ZipWriter::new)?;

    for entry in directory {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let path = entry.path();
        if path.is_file() {
            archive.start_file(
                name,
                FileOptions::default().compression_method(CompressionMethod::Stored),
            )?;
            io::copy(&mut File::open(entry.path())?, &mut archive)?;
        }
    }

    if opts.force {
        fs::remove_dir_all(opts.source())?;
    }

    Ok(())
}
