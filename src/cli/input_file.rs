use clap::{Arg, Error};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::{
    fs::File,
    io::{read_to_string, stdin},
};
use url::Url;

#[derive(Clone, Debug)]
pub(crate) enum InputFile {
    Local(PathBuf),
    Remote(Url),
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
            None => Ok(InputFile::Local(PathBuf::from(value))),
        }
    }
}

impl InputFile {
    pub(crate) fn read_to_string(&self) -> anyhow::Result<String> {
        match self {
            Self::Local(path) if path == Path::new("-") => Ok(read_to_string(stdin())?),
            Self::Local(path) => {
                let file = File::open(path)?;
                Ok(read_to_string(file)?)
            }
            Self::Remote(url) => {
                let client = reqwest::blocking::Client::new();
                let contents = client
                    .get(url.as_str())
                    .header("User-Agent", "GBox")
                    .send()?
                    .error_for_status()?
                    .text()?;

                Ok(contents)
            }
        }
    }
}
