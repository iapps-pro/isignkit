use anyhow::anyhow;
use clap::{Arg, Error};
use std::{
    ffi::OsStr,
    fs::File,
    io::{read_to_string, stdin},
    path::PathBuf,
};
use url::Url;

#[derive(Clone, Debug)]
pub(crate) enum InputFile {
    Stdin,
    Local(PathBuf),
    Remote(Url),
}

impl InputFile {
    #[must_use]
    pub(crate) fn is_remote(&self) -> bool {
        matches!(self, Self::Remote(_))
    }
}

#[derive(Clone)]
pub(crate) struct InputFileParser;

impl clap::builder::TypedValueParser for InputFileParser {
    type Value = InputFile;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, Error> {
        match value.to_str().and_then(|value| Url::parse(value).ok()) {
            Some(url) => Ok(InputFile::Remote(url)),
            None if value == OsStr::new("-") => Ok(InputFile::Stdin),
            None => Ok(InputFile::Local(PathBuf::from(value))),
        }
    }
}

pub(crate) trait InputFileReader {
    fn read_remote(&self, _url: &Url) -> anyhow::Result<String> {
        Err(anyhow!("Remote is not supported by this reader!"))
    }

    fn read_to_string(&self, file: &InputFile, stdin_available: bool) -> anyhow::Result<String> {
        match file {
            InputFile::Stdin if stdin_available => {
                log::info!("Reading from stdin....");
                Ok(read_to_string(stdin())?)
            }
            InputFile::Stdin => Ok(String::new()),
            InputFile::Local(path) => {
                let file = File::open(path)?;
                Ok(read_to_string(file)?)
            }
            InputFile::Remote(url) => self.read_remote(url),
        }
    }
}

pub(crate) struct PlainReader;
impl InputFileReader for PlainReader {}
