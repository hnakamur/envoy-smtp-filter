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

/// VERIFY command asks the receiver to confirm that the argument identifies a user or mailbox.
#[derive(Debug)]
pub struct Vrfy {
    // user or mailbox
    user_or_mailbox: ByteString,
}

impl TryFrom<Vec<u8>> for Vrfy {
    type Error = Error;

    fn try_from(args: Vec<u8>) -> Result<Self> {
        Ok(Vrfy {
            user_or_mailbox: args.into(),
        })
    }
}

impl Vrfy {
    pub const VERB: &'static str = "VRFY";
}
