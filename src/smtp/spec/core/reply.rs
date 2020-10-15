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
use std::fmt;

use envoy::error::format_err;
use envoy::extension::{Error, Result};
use envoy::host::ByteString;

/// Represents an SMTP Reply.
#[derive(Debug)]
pub struct Reply {
    lines: Vec<ReplyLine>,
}

impl Reply {
    pub fn new(line: ReplyLine) -> Self {
        Reply { lines: vec![line] }
    }

    pub fn append(&mut self, line: ReplyLine) {
        self.lines.push(line)
    }

    pub fn code(&self) -> ReplyCode {
        self.lines
            .first()
            .map(|line| line.code())
            .unwrap_or_else(|| ReplyCode {
                x: ReplyType::PermanentNegativeCompletionReply,
                y: ReplyCategory::Syntax,
                z: ReplyGradation(0),
            })
    }
}

/// Represents an SMTP Reply type.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ReplyType {
    PositiveCompletionReply,
    PositiveIntermediateReply,
    TransientNegativeCompletionReply,
    PermanentNegativeCompletionReply,
}

impl ReplyType {
    pub fn is_positive(&self) -> bool {
        use ReplyType::*;
        match self {
            PositiveCompletionReply | PositiveIntermediateReply => true,
            TransientNegativeCompletionReply | PermanentNegativeCompletionReply => false,
        }
    }
}

impl fmt::Display for ReplyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ReplyType::*;
        let digit = match self {
            PositiveCompletionReply => '2',
            PositiveIntermediateReply => '3',
            TransientNegativeCompletionReply => '4',
            PermanentNegativeCompletionReply => '5',
        };
        write!(f, "{}", digit)
    }
}

impl TryFrom<u8> for ReplyType {
    type Error = Error;

    fn try_from(octet: u8) -> Result<Self> {
        use ReplyType::*;
        match octet {
            b'2' => Ok(PositiveCompletionReply),
            b'3' => Ok(PositiveIntermediateReply),
            b'4' => Ok(TransientNegativeCompletionReply),
            b'5' => Ok(PermanentNegativeCompletionReply),
            _ => Err(format_err!("not a valid reply type: {}", octet)),
        }
    }
}

/// Represents an SMTP Reply category.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum ReplyCategory {
    Syntax,
    Information,
    Connections,
    X3Z,
    X4Z,
    MailSystem,
}

impl fmt::Display for ReplyCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ReplyCategory::*;
        let digit = match self {
            Syntax => '0',
            Information => '1',
            Connections => '2',
            X3Z => '3',
            X4Z => '4',
            MailSystem => '5',
        };
        write!(f, "{}", digit)
    }
}

impl TryFrom<u8> for ReplyCategory {
    type Error = Error;

    fn try_from(octet: u8) -> Result<Self> {
        use ReplyCategory::*;
        match octet {
            b'0' => Ok(Syntax),
            b'1' => Ok(Information),
            b'2' => Ok(Connections),
            b'3' => Ok(X3Z),
            b'4' => Ok(X4Z),
            b'5' => Ok(MailSystem),
            _ => Err(format_err!("not a valid reply category: {}", octet)),
        }
    }
}

/// Represents an SMTP Reply category gradation.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ReplyGradation(u8);

impl fmt::Display for ReplyGradation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<u8> for ReplyGradation {
    type Error = Error;

    fn try_from(octet: u8) -> Result<Self> {
        match octet {
            b'0'..=b'9' => Ok(ReplyGradation(octet - b'0')),
            _ => Err(format_err!("not a valid reply gradation: {}", octet)),
        }
    }
}

/// Represents SMTP Reply code.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct ReplyCode {
    x: ReplyType,
    y: ReplyCategory,
    z: ReplyGradation,
}

impl ReplyCode {
    pub fn response_type(&self) -> ReplyType {
        self.x
    }
}

impl fmt::Display for ReplyCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.x, self.y, self.z)
    }
}

impl TryFrom<Vec<u8>> for ReplyCode {
    type Error = Error;

    fn try_from(line: Vec<u8>) -> Result<Self> {
        if line.len() != 3 {
            return Err(format_err!(
                "not a valid reply code: {}",
                ByteString::from(line)
            ));
        }
        let response_type = ReplyType::try_from(line[0])?;
        let category = ReplyCategory::try_from(line[1])?;
        let gradation = ReplyGradation::try_from(line[2])?;
        Ok(ReplyCode {
            x: response_type,
            y: category,
            z: gradation,
        })
    }
}

/// Represents a single line of the SMTP Reply.
#[derive(Debug)]
pub struct ReplyLine {
    code: ReplyCode,
    last: bool,
    text: ByteString,
}

impl TryFrom<Vec<u8>> for ReplyLine {
    type Error = Error;

    fn try_from(mut line: Vec<u8>) -> Result<Self> {
        if line.len() < 3 {
            return Err(format_err!(
                "not a valid reply line: {}",
                ByteString::from(line)
            ));
        }
        let code = ReplyCode::try_from(line.drain(0..3).collect::<Vec<u8>>())?;
        let sep = line.drain(0..1).collect::<Vec<u8>>();
        let last = match sep[..] {
            [] | [b' '] => true,
            [b'-'] => false,
            _ => {
                return Err(format_err!(
                    "not a valid reply line: {}",
                    ByteString::from(line)
                ))
            }
        };
        Ok(ReplyLine {
            code,
            last,
            text: line.into(),
        })
    }
}

impl ReplyLine {
    pub fn code(&self) -> ReplyCode {
        self.code
    }

    pub fn is_end_line(&self) -> bool {
        self.last
    }
}
