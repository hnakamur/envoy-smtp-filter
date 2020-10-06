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
use envoy::host::ByteString;

use crate::smtp::spec::core::SP;

/// Represent unknown command.
#[derive(Debug)]
pub struct Unknown {
    // verb
    verb: String,
    // args
    args: ByteString,
}

impl TryFrom<Vec<u8>> for Unknown {
    type Error = Error;

    fn try_from(line: Vec<u8>) -> Result<Self> {
        let (verb, args) = match line.find(SP) {
            Some(index) => (&line[0..index], &line[index + 1..]),
            None => (&line[..], &line[0..0]),
        };

        let mut verb = String::from_utf8(verb.to_vec())?;
        verb.make_ascii_uppercase();
        let args = args.to_vec();

        Ok(Unknown {
            verb,
            args: args.into(),
        })
    }
}

impl Unknown {
    pub fn verb(&self) -> &str {
        &self.verb
    }
}
