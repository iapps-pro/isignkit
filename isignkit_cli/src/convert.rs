use super::input_file::{InputFile, InputFileParser, InputFileReader, Reader};
use anyhow::Result;
use isignkit::{altstore, gbox, sidestore};
use std::{
    fs::File,
    io::{Write, stdout},
    path::{Path, PathBuf},
};

#[derive(clap::ValueEnum, Copy, Clone)]
enum Format {
    Altstore,
    Sidestore,
    Gbox,
}

impl Format {
    fn useragent(&self) -> &str {
        match self {
            Self::Altstore => "AltStore/1.0",
            Self::Sidestore => "SideStore/1.0",
            Self::Gbox => "GBox/1.0",
        }
    }
}

/// Converts apps repository into between different backends
///
/// Since these backends use their own formats and datatypes, some data can be lost.
///
/// Please, carefully check output file and add necessary fields.
#[derive(clap::Parser)]
pub(crate) struct ConvertCommand {
    /// Source repository format
    #[arg(long, short = 'f')]
    source_format: Format,

    /// Target repository format
    #[arg(long, short = 'O')]
    target_format: Format,

    /// Source to convert from. Can be stdin (-), URL or file.
    #[arg(value_parser = InputFileParser)]
    source: InputFile,

    /// Target file. Default is stdout (-)
    #[clap(default_value = "-", hide_default_value = true)]
    target: PathBuf,
}

impl super::CliCommand for ConvertCommand {
    fn run(&self) -> Result<()> {
        let reader = Reader::with_useragent(self.source_format.useragent())?;
        let input = reader.read_to_string(&self.source, false)?;

        let repo = AnyRepository::new(&input, self.source_format)?;
        let apps_cnt_before = repo.apps_count();

        let repo = repo.convert(self.target_format);
        let apps_cnt_after = repo.apps_count();

        let is_stdout = self.target == Path::new("-");
        let mut target: Box<dyn Write> = if is_stdout {
            Box::new(stdout())
        } else {
            Box::new(File::create(&self.target)?)
        };

        let json = serde_json::to_string_pretty(&repo)?;

        #[allow(
            clippy::unused_io_amount,
            reason = "output can intentionally be truncated by user (e.g. pipes)"
        )]
        target.write(json.as_bytes())?;
        target.flush()?;

        #[allow(clippy::cast_precision_loss)]
        let percent = (apps_cnt_after as f64 / apps_cnt_before as f64) * 100.0;
        eprintln!();
        eprintln!(
            "Successfully converted {apps_cnt_after} of {apps_cnt_before} apps ({percent:.1}%)"
        );

        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(untagged)]
enum AnyRepository {
    Altstore(altstore::Repository),
    Sidestore(sidestore::Repository),
    Gbox(gbox::GboxRepo),
}

impl AnyRepository {
    fn new(s: &str, format: Format) -> Result<Self> {
        match format {
            Format::Altstore => Ok(Self::Altstore(serde_json::from_slice(s.as_bytes())?)),
            Format::Sidestore => Ok(Self::Sidestore(serde_json::from_slice(s.as_bytes())?)),
            Format::Gbox => Ok(Self::Gbox(serde_json::from_slice(s.as_bytes())?)),
        }
    }

    fn convert(self, format: Format) -> Self {
        match (self, format) {
            (Self::Altstore(repo), Format::Altstore) => Self::Altstore(repo),
            (Self::Altstore(repo), Format::Sidestore) => Self::Sidestore(repo.into()),
            (Self::Altstore(repo), Format::Gbox) => Self::Gbox(repo.into()),
            (Self::Sidestore(repo), Format::Altstore) => Self::Altstore(repo.into()),
            (Self::Sidestore(repo), Format::Sidestore) => Self::Sidestore(repo),
            (Self::Sidestore(repo), Format::Gbox) => Self::Gbox(repo.into()),
            (Self::Gbox(repo), Format::Altstore) => Self::Altstore(repo.into()),
            (Self::Gbox(repo), Format::Sidestore) => Self::Sidestore(repo.into()),
            (Self::Gbox(repo), Format::Gbox) => Self::Gbox(repo),
        }
    }

    fn apps_count(&self) -> usize {
        match self {
            Self::Altstore(repo) => {
                let apps = repo.apps.as_ref();
                apps.map(Vec::len).unwrap_or_default()
            }
            Self::Sidestore(repo) => {
                let apps = repo.apps.as_ref();
                apps.map(Vec::len).unwrap_or_default()
            }
            Self::Gbox(gbox::GboxRepo::Plain(repo)) => repo.applications.len(),
            Self::Gbox(gbox::GboxRepo::Encrypted(_)) => 0,
        }
    }
}
