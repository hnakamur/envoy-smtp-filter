// Copyright 2020 Tetrate
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::convert::TryFrom;

use bstr::ByteSlice;
use envoy::extension::{Error, Result};

use crate::smtp::spec::core::{
    Data, Ehlo, Expn, Helo, Help, Mail, Noop, Quit, Rcpt, Rset, Vrfy, SP,
};
use crate::smtp::spec::extensions::starttls::StartTls;
use crate::smtp::spec::unknown::Unknown;

/// Enumerates SMTP commands supported by this Mail Transfer Agent.
#[derive(Debug)]
pub enum Command {
    Helo(Helo),
    Ehlo(Ehlo),
    Mail(Mail),
    Rcpt(Rcpt),
    Data(Data),
    Rset(Rset),
    Vrfy(Vrfy),
    Expn(Expn),
    Help(Help),
    Noop(Noop),
    Quit(Quit),
    StartTls(StartTls),
    Unknown(Unknown),
}

impl Command {
    pub fn verb(&self) -> &str {
        match self {
            Command::Helo(_) => Helo::VERB,
            Command::Ehlo(_) => Ehlo::VERB,
            Command::Mail(_) => Mail::VERB,
            Command::Rcpt(_) => Rcpt::VERB,
            Command::Data(_) => Data::VERB,
            Command::Rset(_) => Rset::VERB,
            Command::Vrfy(_) => Vrfy::VERB,
            Command::Expn(_) => Expn::VERB,
            Command::Help(_) => Help::VERB,
            Command::Noop(_) => Noop::VERB,
            Command::Quit(_) => Quit::VERB,
            Command::StartTls(StartTls) => StartTls::VERB,
            Command::Unknown(unknown) => &unknown.verb(),
        }
    }
}

impl TryFrom<Vec<u8>> for Command {
    type Error = Error;

    fn try_from(line: Vec<u8>) -> Result<Self> {
        let (verb, args) = match line.find(SP) {
            Some(index) => (&line[0..index], &line[index + 1..]),
            None => (&line[..], &line[0..0]),
        };

        let mut verb = String::from_utf8(verb.to_vec())?;
        verb.make_ascii_uppercase();
        let args = args.to_vec();
        match verb.as_str() {
            Helo::VERB => Helo::try_from(args).map(Command::Helo),
            Ehlo::VERB => Ehlo::try_from(args).map(Command::Ehlo),
            Mail::VERB => Mail::try_from(args).map(Command::Mail),
            Rcpt::VERB => Rcpt::try_from(args).map(Command::Rcpt),
            Data::VERB => Ok(Command::Data(Data)),
            Rset::VERB => Ok(Command::Rset(Rset)),
            Vrfy::VERB => Vrfy::try_from(args).map(Command::Vrfy),
            Expn::VERB => Expn::try_from(args).map(Command::Expn),
            Help::VERB => Help::try_from(args).map(Command::Help),
            Noop::VERB => Noop::try_from(args).map(Command::Noop),
            Quit::VERB => Ok(Command::Quit(Quit)),
            StartTls::VERB => Ok(Command::StartTls(StartTls)),
            _ => Unknown::try_from(line).map(Command::Unknown),
        }
    }
}
