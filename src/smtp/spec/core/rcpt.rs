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

use envoy::extension::{Error, Result};
use envoy::host::ByteString;

/// RECIPIENT command is used to identify an individual recipient of the mail data.
///
/// Multiple recipients are specified by multiple uses of this command.
#[derive(Debug)]
pub struct Rcpt {
    // "<Postmaster@" Domain ">" / "<Postmaster>" / Forward-path
    to: ByteString,
    // Rcpt-parameters
    params: Option<ByteString>,
}

impl TryFrom<Vec<u8>> for Rcpt {
    type Error = Error;

    fn try_from(args: Vec<u8>) -> Result<Self> {
        Ok(Rcpt {
            to: args.into(),
            params: None,
        })
    }
}

impl Rcpt {
    pub const VERB: &'static str = "RCPT";

    pub fn to(&self) -> &ByteString {
        &self.to
    }
}
