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

use std::ops::Deref;
use std::rc::Rc;

use envoy::extension::Result;

use crate::smtp::spec::core::ReplyCode;

pub trait StatsSink {
    fn on_smtp_connect(&self) -> Result<()> {
        Ok(())
    }

    fn on_smtp_connect_reply(&self, _code: ReplyCode) -> Result<()> {
        Ok(())
    }

    fn on_smtp_command(&self, _verb: &str) -> Result<()> {
        Ok(())
    }

    fn on_smtp_command_reply(&self, _verb: &str, _code: ReplyCode) -> Result<()> {
        Ok(())
    }

    fn on_smtp_transaction_commit(&self) -> Result<()> {
        Ok(())
    }

    fn on_smtp_transaction_commit_reply(&self, _code: ReplyCode) -> Result<()> {
        Ok(())
    }

    fn on_smtp_parse_error(&self) -> Result<()> {
        Ok(())
    }
}

impl<T: StatsSink> StatsSink for Rc<T> {
    fn on_smtp_connect(&self) -> Result<()> {
        self.deref().on_smtp_connect()
    }

    fn on_smtp_connect_reply(&self, code: ReplyCode) -> Result<()> {
        self.deref().on_smtp_connect_reply(code)
    }

    fn on_smtp_command(&self, verb: &str) -> Result<()> {
        self.deref().on_smtp_command(verb)
    }

    fn on_smtp_command_reply(&self, verb: &str, code: ReplyCode) -> Result<()> {
        self.deref().on_smtp_command_reply(verb, code)
    }

    fn on_smtp_transaction_commit(&self) -> Result<()> {
        self.deref().on_smtp_transaction_commit()
    }

    fn on_smtp_transaction_commit_reply(&self, code: ReplyCode) -> Result<()> {
        self.deref().on_smtp_transaction_commit_reply(code)
    }

    fn on_smtp_parse_error(&self) -> Result<()> {
        self.deref().on_smtp_parse_error()
    }
}
