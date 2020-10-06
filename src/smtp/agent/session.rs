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

use std::collections::VecDeque;
use std::convert::TryFrom;

use bstr::{ByteSlice, ByteVec};
use envoy::error::format_err;
use envoy::extension::{Error, Result};
use envoy::host::log;
use envoy::host::ByteString;

use super::command::Command;
use super::stats::StatsSink;
use crate::smtp::spec::core::{
    Data, Ehlo, Expn, Helo, Help, Mail, Noop, Quit, Rcpt, Reply, ReplyLine, Rset, Vrfy, CR_LF,
};
use crate::smtp::spec::extensions::starttls::StartTls;
use crate::smtp::spec::unknown::Unknown;

/// Session represents a single SMTP session.
pub struct Session<S: StatsSink> {
    downstream_buffer: Vec<u8>,
    upstream_buffer: Vec<u8>,

    mode: Mode,

    next_reply: Option<Reply>,
    next_body: Vec<u8>,

    pending_replies: VecDeque<PendingReply>,
    active_transaction: Option<Transaction>,

    stats_sink: S,
}

/// PendingReply represents a pending reply from SMTP server
/// in response to connect, command or mail transaction commit.
#[derive(Debug)]
pub enum PendingReply {
    /// Pending reply to a connect.
    Connect,
    /// Pending reply to an SMTP command.
    Command(Command),
    /// Pending reply to a mail transaction commit.
    Commit(Transaction),
}

/// Transaction represents a single mail transaction.
#[derive(Debug, Default)]
pub struct Transaction {
    from: ByteString,
    to: Vec<ByteString>,
    body: ByteString,
}

/// Mode represents a mode the SMTP session is currently in.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum Mode {
    /// Mode in which an SMTP client is expected to wait for a reply to connect.
    Connect,
    /// Mode in which an SMTP client is expected to send SMTP commands.
    Command,
    /// Mode in which an SMTP client is expected to send mail data.
    Data,
    /// Mode in which observed traffic is not interpreted anymore, e.g.
    /// after encountering an parsing error or after switching to TLS.
    PassThrough,
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Connect
    }
}

impl<S> Session<S>
where
    S: StatsSink,
{
    pub fn new(stats_sink: S) -> Self {
        Session {
            downstream_buffer: Vec::<u8>::new(),
            upstream_buffer: Vec::<u8>::new(),
            mode: Mode::Connect,
            next_reply: None,
            next_body: Vec::<u8>::new(),
            pending_replies: VecDeque::<PendingReply>::new(),
            active_transaction: None,
            stats_sink,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn on_new_conection(&mut self) -> Result<()> {
        self.stats_sink.on_smtp_connect()?;
        self.pending_replies.push_back(PendingReply::Connect);
        Ok(())
    }

    pub fn on_downstream_data(&mut self, new_data: ByteString) -> Result<()> {
        match self.mode {
            Mode::Connect | Mode::Command | Mode::Data => {
                self.downstream_buffer.extend(new_data.into_bytes());
            }
            Mode::PassThrough => return Ok(()), // don't even append new data to the buffer
        }
        loop {
            let mode = self.mode;
            match mode {
                Mode::Connect | Mode::Command => {
                    match self.next_command() {
                        Ok(Some(cmd)) => {
                            self.stats_sink.on_smtp_command(cmd.verb())?;
                            self.pending_replies.push_back(PendingReply::Command(cmd));
                            continue; // to the next command
                        }
                        Ok(None) => return Ok(()), // wait for a complete command
                        Err(err) => return self.fallback(err),
                    }
                }
                Mode::Data => {
                    match self.next_body() {
                        Some(body) => {
                            self.active_transaction
                                .get_or_insert_with(Default::default)
                                .body = body.into();
                            if let Some(tx) = self.active_transaction.take() {
                                log::debug!("committing transaction: {:?}", tx);
                                self.pending_replies.push_back(PendingReply::Commit(tx));
                            }
                            self.stats_sink.on_smtp_transaction_commit()?;
                            self.mode = Mode::Command;
                            continue; // to the next command
                        }
                        None => return Ok(()), // wait until body is complete
                    }
                }
                Mode::PassThrough => return Ok(()), // do nothing
            }
        }
    }

    pub fn on_upstream_data(&mut self, new_data: ByteString) -> Result<()> {
        match self.mode {
            Mode::Connect | Mode::Command | Mode::Data => {
                self.upstream_buffer.extend(new_data.into_bytes());
            }
            Mode::PassThrough => return Ok(()), // don't append new data to the buffer
        }
        loop {
            let mode = self.mode;
            match mode {
                Mode::Connect | Mode::Command | Mode::Data => {
                    match self.next_reply() {
                        Ok(Some(reply)) => match self.handle_reply(reply) {
                            Ok(()) => continue, // to the next reply
                            Err(err) => return self.fallback(err),
                        },
                        Ok(None) => return Ok(()), // wait for a complete reply
                        Err(err) => return self.fallback(err),
                    }
                }
                Mode::PassThrough => return Ok(()), // do nothing
            }
        }
    }

    fn reset(&mut self) {
        self.active_transaction = None
    }

    fn fallback(&mut self, err: Error) -> Result<()> {
        log::error!(
            "falling back into no-op mode due to a protocol parsing error: {}",
            err
        );
        self.stats_sink.on_smtp_parse_error()?;
        self.mode = Mode::PassThrough;
        Ok(())
    }

    fn next_command(&mut self) -> Result<Option<Command>> {
        match next_line(&mut self.downstream_buffer) {
            Some(line) => Command::try_from(line).map(Option::from),
            None => Ok(None),
        }
    }

    fn next_body(&mut self) -> Option<Vec<u8>> {
        loop {
            match next_line(&mut self.downstream_buffer) {
                Some(line) => {
                    let end = !self.next_body.is_empty() && line == b"."; // <CR><LF>.<CR><LF>
                    self.next_body.extend(line);
                    self.next_body.push_str(CR_LF);
                    if end {
                        return Some(self.next_body.drain(..).collect());
                    }
                    continue; // to the next line
                }
                None => return None,
            }
        }
    }

    fn next_reply(&mut self) -> Result<Option<Reply>> {
        loop {
            match next_line(&mut self.upstream_buffer) {
                Some(next) => {
                    log::debug!("next reply line: {}", next.as_bstr());
                    let line = ReplyLine::try_from(next)?;
                    let end_line = line.is_end_line();
                    if let Some(reply) = self.next_reply.as_mut() {
                        reply.append(line);
                    } else {
                        self.next_reply = Some(Reply::new(line));
                    }
                    if end_line {
                        return Ok(self.next_reply.take());
                    }
                }
                None => return Ok(None),
            }
        }
    }

    fn handle_reply(&mut self, reply: Reply) -> Result<()> {
        match self.pending_replies.pop_front() {
            Some(pending) => {
                use PendingReply::*;
                match pending {
                    Connect => {
                        self.stats_sink.on_smtp_connect_reply(reply.code())?;
                        self.mode = Mode::Command;
                        Ok(())
                    }
                    Command(cmd) => {
                        self.stats_sink
                            .on_smtp_command_reply(cmd.verb(), reply.code())?;
                        cmd.handle_reply(self, reply)?;
                        Ok(())
                    }
                    Commit(_) => {
                        self.stats_sink
                            .on_smtp_transaction_commit_reply(reply.code())?;
                        Ok(())
                    }
                }
            }
            None => Err(format_err!(
                "received a reply while no command is pending: {:?}",
                reply
            )),
        }
    }
}

fn next_line(buffer: &mut Vec<u8>) -> Option<Vec<u8>> {
    match buffer.find(CR_LF) {
        Some(index) => {
            let line: Vec<u8> = buffer.drain(0..index).collect();
            buffer.drain(0..CR_LF.len());
            Some(line)
        }
        None => None,
    }
}

trait ReplyHandler {
    fn handle_reply<S: StatsSink>(&self, session: &mut Session<S>, reply: Reply) -> Result<()>;
}

impl ReplyHandler for Command {
    fn handle_reply<S: StatsSink>(&self, session: &mut Session<S>, reply: Reply) -> Result<()> {
        use Command::*;
        match self {
            Helo(helo) => helo.handle_reply(session, reply),
            Ehlo(ehlo) => ehlo.handle_reply(session, reply),
            Mail(mail) => mail.handle_reply(session, reply),
            Rcpt(rcpt) => rcpt.handle_reply(session, reply),
            Data(data) => data.handle_reply(session, reply),
            Rset(rset) => rset.handle_reply(session, reply),
            Vrfy(vrfy) => vrfy.handle_reply(session, reply),
            Expn(expn) => expn.handle_reply(session, reply),
            Help(help) => help.handle_reply(session, reply),
            Noop(noop) => noop.handle_reply(session, reply),
            Quit(quit) => quit.handle_reply(session, reply),
            StartTls(stls) => stls.handle_reply(session, reply),
            Unknown(unknown) => unknown.handle_reply(session, reply),
        }
    }
}

impl ReplyHandler for Helo {
    fn handle_reply<S: StatsSink>(&self, session: &mut Session<S>, reply: Reply) -> Result<()> {
        log::debug!("handling reply to {}: {:?}", Self::VERB, reply);
        if reply.code().response_type().is_positive() {
            session.reset();
        }
        Ok(())
    }
}

impl ReplyHandler for Ehlo {
    fn handle_reply<S: StatsSink>(&self, session: &mut Session<S>, reply: Reply) -> Result<()> {
        log::debug!("handling reply to {}: {:?}", Self::VERB, reply);
        if reply.code().response_type().is_positive() {
            session.reset();
        }
        Ok(())
    }
}

impl ReplyHandler for Mail {
    fn handle_reply<S: StatsSink>(&self, session: &mut Session<S>, reply: Reply) -> Result<()> {
        log::debug!("handling reply to {}: {:?}", Self::VERB, reply);
        if reply.code().response_type().is_positive() {
            session
                .active_transaction
                .get_or_insert_with(Default::default)
                .from = self.from().clone();
        }
        Ok(())
    }
}

impl ReplyHandler for Rcpt {
    fn handle_reply<S: StatsSink>(&self, session: &mut Session<S>, reply: Reply) -> Result<()> {
        log::debug!("handling reply to {}: {:?}", Self::VERB, reply);
        if reply.code().response_type().is_positive() {
            session
                .active_transaction
                .get_or_insert_with(Default::default)
                .to
                .push(self.to().clone());
        }
        Ok(())
    }
}

impl ReplyHandler for Data {
    fn handle_reply<S: StatsSink>(&self, session: &mut Session<S>, reply: Reply) -> Result<()> {
        log::debug!("handling reply to {}: {:?}", Self::VERB, reply);
        if reply.code().response_type().is_positive() {
            session
                .active_transaction
                .get_or_insert_with(Default::default)
                .body = ByteString::new();
            session.mode = Mode::Data;
        }
        Ok(())
    }
}

impl ReplyHandler for Rset {
    fn handle_reply<S: StatsSink>(&self, session: &mut Session<S>, reply: Reply) -> Result<()> {
        log::debug!("handling reply to {}: {:?}", Self::VERB, reply);
        if reply.code().response_type().is_positive() {
            session.reset();
        }
        Ok(())
    }
}

impl ReplyHandler for Vrfy {
    fn handle_reply<S: StatsSink>(&self, _session: &mut Session<S>, reply: Reply) -> Result<()> {
        log::debug!("handling reply to {}: {:?}", Self::VERB, reply);
        Ok(())
    }
}

impl ReplyHandler for Expn {
    fn handle_reply<S: StatsSink>(&self, _session: &mut Session<S>, reply: Reply) -> Result<()> {
        log::debug!("handling reply to {}: {:?}", Self::VERB, reply);
        Ok(())
    }
}

impl ReplyHandler for Help {
    fn handle_reply<S: StatsSink>(&self, _session: &mut Session<S>, reply: Reply) -> Result<()> {
        log::debug!("handling reply to {}: {:?}", Self::VERB, reply);
        Ok(())
    }
}

impl ReplyHandler for Noop {
    fn handle_reply<S: StatsSink>(&self, _session: &mut Session<S>, reply: Reply) -> Result<()> {
        log::debug!("handling reply to {}: {:?}", Self::VERB, reply);
        Ok(())
    }
}

impl ReplyHandler for Quit {
    fn handle_reply<S: StatsSink>(&self, _session: &mut Session<S>, reply: Reply) -> Result<()> {
        log::debug!("handling reply to {}: {:?}", Self::VERB, reply);
        Ok(())
    }
}

impl ReplyHandler for StartTls {
    fn handle_reply<S: StatsSink>(&self, session: &mut Session<S>, reply: Reply) -> Result<()> {
        log::debug!("handling reply to {}: {:?}", Self::VERB, reply);
        if reply.code().response_type().is_positive() {
            session.mode = Mode::PassThrough;
        }
        Ok(())
    }
}

impl ReplyHandler for Unknown {
    fn handle_reply<S: StatsSink>(&self, session: &mut Session<S>, reply: Reply) -> Result<()> {
        log::debug!(
            "handling reply to unknown command {}: {:?}",
            self.verb(),
            reply
        );
        if reply.code().response_type().is_positive() {
            session.mode = Mode::PassThrough;
        }
        Ok(())
    }
}
