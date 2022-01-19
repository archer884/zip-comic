use std::{
    borrow::Cow,
    ffi::OsStr,
    fs::{self, File},
    io,
    path::Path,
};

use clap::Parser;
use zip::{write::FileOptions, CompressionMethod, ZipWriter};

/// comic reader archive creator
///
/// Note: this zip utility only works for flat archives.
#[derive(Clone, Debug, Parser)]
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

                // Turns out that .with_extension() will pretty much NEVER do the right thing for
                // a directory path. As such, we're going to roll our own. 🎵 ...Cowboy! 🎵

                let mut archive_name = path
                    .file_name()
                    .unwrap_or_else(|| OsStr::new("archive"))
                    .to_owned();
                archive_name.push(".cbz");
                Cow::from(path.with_file_name(archive_name))
            })
    }
}

fn main() {
    if let Err(e) = run(&Opts::parse()) {
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::Opts;

    #[test]
    fn destination() {
        let opts = Opts {
            path: String::from("/Red vs. Blue"),
            output: None,
            force: false,
        };

        assert_eq!(opts.destination(), Path::new("/Red vs. Blue.zip"))
    }
}
