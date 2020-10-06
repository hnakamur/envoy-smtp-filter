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

/// HELO command is used to identify the SMTP client to the SMTP server.
#[derive(Debug)]
pub struct Helo {
    /// Domain
    domain: ByteString,
}

impl TryFrom<Vec<u8>> for Helo {
    type Error = Error;

    fn try_from(args: Vec<u8>) -> Result<Self> {
        Ok(Helo {
            domain: args.into(),
        })
    }
}

impl Helo {
    pub const VERB: &'static str = "HELO";
}
