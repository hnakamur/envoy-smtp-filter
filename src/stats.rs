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

use envoy::extension::Result;
use envoy::host::stats::{Counter, Stats};

use crate::smtp::agent::StatsSink;
use crate::smtp::spec::core::ReplyCode;

// SMTP stats.
pub struct SmtpFilterStats<'a> {
    detailed: bool,
    stats: &'a dyn Stats,
    connections_total: Box<dyn Counter>,
    connections_errors_total: Box<dyn Counter>,
    connects_total: Box<dyn Counter>,
    connects_replies_total: Box<dyn Counter>,
    connects_replies_positive_total: Box<dyn Counter>,
    connects_replies_negative_total: Box<dyn Counter>,
    commands_total: Box<dyn Counter>,
    commands_replies_total: Box<dyn Counter>,
    commands_replies_positive_total: Box<dyn Counter>,
    commands_replies_negative_total: Box<dyn Counter>,
    transaction_commits_total: Box<dyn Counter>,
    transaction_commits_replies_total: Box<dyn Counter>,
    transaction_commits_replies_positive_total: Box<dyn Counter>,
    transaction_commits_replies_negative_total: Box<dyn Counter>,
    mails_total: Box<dyn Counter>,
    mails_sent_total: Box<dyn Counter>,
    mails_rejected_total: Box<dyn Counter>,
}

impl<'a> SmtpFilterStats<'a> {
    pub fn new(detailed: bool, stats: &'a dyn Stats) -> Result<Self> {
        Ok(SmtpFilterStats {
            detailed,
            stats,
            connections_total: stats.counter("smtp.connections.total")?,
            connections_errors_total: stats.counter("smtp.connections.parse_errors.total")?,
            connects_total: stats.counter("smtp.connects.total")?,
            connects_replies_total: stats.counter("smtp.connects.replies.total")?,
            connects_replies_positive_total: stats
                .counter("smtp.connects.replies.positive.total")?,
            connects_replies_negative_total: stats
                .counter("smtp.connects.replies.negative.total")?,
            commands_total: stats.counter("smtp.commands.total")?,
            commands_replies_total: stats.counter("smtp.commands.replies.total")?,
            commands_replies_positive_total: stats
                .counter("smtp.commands.replies.positive.total")?,
            commands_replies_negative_total: stats
                .counter("smtp.commands.replies.negative.total")?,
            transaction_commits_total: stats.counter("smtp.transactions.commits.total")?,
            transaction_commits_replies_total: stats
                .counter("smtp.transactions.commits.replies.total")?,
            transaction_commits_replies_positive_total: stats
                .counter("smtp.transactions.commits.replies.positive.total")?,
            transaction_commits_replies_negative_total: stats
                .counter("smtp.transactions.commits.replies.negative.total")?,
            mails_total: stats.counter("smtp.mails.total")?,
            mails_sent_total: stats.counter("smtp.mails.sent.total")?,
            mails_rejected_total: stats.counter("smtp.mails.rejected.total")?,
        })
    }

    pub fn is_detailed(&self) -> bool {
        self.detailed
    }
}

impl<'a> StatsSink for SmtpFilterStats<'a> {
    fn on_smtp_connect(&self) -> Result<()> {
        self.connections_total.inc()?;
        self.connects_total.inc()
    }

    fn on_smtp_connect_reply(&self, code: ReplyCode) -> Result<()> {
        self.connects_replies_total.inc()?;
        if code.response_type().is_positive() {
            self.connects_replies_positive_total.inc()?;
        } else {
            self.connects_replies_negative_total.inc()?;
        }
        if self.detailed {
            self.stats
                .counter(&format!("smtp.connects.reply.{}.total", code))?
                .inc()?;
        }
        Ok(())
    }

    fn on_smtp_command(&self, verb: &str) -> Result<()> {
        self.commands_total.inc()?;
        if self.detailed {
            self.stats
                .counter(&format!("smtp.command.{}.total", verb))?
                .inc()?;
        }
        Ok(())
    }

    fn on_smtp_command_reply(&self, verb: &str, code: ReplyCode) -> Result<()> {
        self.commands_replies_total.inc()?;
        if code.response_type().is_positive() {
            self.commands_replies_positive_total.inc()?;
        } else {
            self.commands_replies_negative_total.inc()?;
        }
        if self.detailed {
            self.stats
                .counter(&format!("smtp.command.{}.replies.total", verb))?
                .inc()?;
            self.stats
                .counter(&format!("smtp.command.{}.reply.{}.total", verb, code))?
                .inc()?;
            if code.response_type().is_positive() {
                self.stats
                    .counter(&format!("smtp.command.{}.replies.positive.total", verb))?
                    .inc()?;
            } else {
                self.stats
                    .counter(&format!("smtp.command.{}.replies.negative.total", verb))?
                    .inc()?;
            }
        }
        Ok(())
    }

    fn on_smtp_transaction_commit(&self) -> Result<()> {
        self.transaction_commits_total.inc()?;
        self.mails_total.inc()
    }

    fn on_smtp_transaction_commit_reply(&self, code: ReplyCode) -> Result<()> {
        self.transaction_commits_replies_total.inc()?;
        if code.response_type().is_positive() {
            self.transaction_commits_replies_positive_total.inc()?;
        } else {
            self.transaction_commits_replies_negative_total.inc()?;
        }
        if code.response_type().is_positive() {
            self.mails_sent_total.inc()?;
        } else {
            self.mails_rejected_total.inc()?;
        }
        if self.detailed {
            self.stats
                .counter(&format!("smtp.transactions.commits.reply.{}.total", code))?
                .inc()?;
        }
        Ok(())
    }

    fn on_smtp_parse_error(&self) -> Result<()> {
        self.connections_errors_total.inc()
    }
}
