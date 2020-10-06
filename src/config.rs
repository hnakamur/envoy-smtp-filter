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

use serde::Deserialize;

use envoy::extension;

/// Configuration for a SMTP Filter.
#[derive(Debug, Default, Deserialize)]
pub struct SmtpFilterConfig {
    /// Indicates whether SMTP filter should produce individual stats for
    /// each of the SMTP verbs and reply codes.
    pub detailed_stats: bool,
}

impl TryFrom<&[u8]> for SmtpFilterConfig {
    type Error = extension::Error;

    /// Parses filter configuration from JSON.
    fn try_from(value: &[u8]) -> extension::Result<Self> {
        serde_json::from_slice(value).map_err(extension::Error::from)
    }
}
