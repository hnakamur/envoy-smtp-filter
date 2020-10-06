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

use std::rc::Rc;

use envoy::extension::{filter::network, InstanceId, NetworkFilter, Result};
use envoy::host::log;

use crate::config::SmtpFilterConfig;
use crate::smtp::agent::{Mode, Session};
use crate::stats::SmtpFilterStats;

/// Envoy SMTP Filter.
pub struct SmtpFilter<'a> {
    // SMTP Filter instance id.
    instance_id: InstanceId,
    // Configuration shared by multiple filter instances.
    config: Rc<SmtpFilterConfig>,
    session: Session<Rc<SmtpFilterStats<'a>>>,
}

impl<'a> SmtpFilter<'a> {
    /// Creates a new instance of SMTP Filter.
    pub fn new(
        instance_id: InstanceId,
        config: Rc<SmtpFilterConfig>,
        stats: Rc<SmtpFilterStats<'a>>,
    ) -> Self {
        // Inject dependencies on Envoy host APIs
        SmtpFilter {
            instance_id,
            config,
            session: Session::new(stats),
        }
    }
}

impl<'a> NetworkFilter for SmtpFilter<'a> {
    /// Called when a new TCP connection is opened.
    fn on_new_connection(&mut self) -> Result<network::FilterStatus> {
        log::debug!(
            "#{} new TCP connection starts with config: {:?}",
            self.instance_id,
            self.config,
        );
        self.session.on_new_conection()?;
        Ok(network::FilterStatus::Continue)
    }

    fn on_downstream_data(
        &mut self,
        data_size: usize,
        _end_of_stream: bool,
        ops: &dyn network::DownstreamDataOps,
    ) -> Result<network::FilterStatus> {
        if self.session.mode() == Mode::PassThrough {
            // has fallen back into no-op mode, e.g. due to a parsing error or
            // because of STARTTLS command
            return Ok(network::FilterStatus::Continue);
        }
        let new_data = ops.downstream_data(0, data_size)?;
        log::debug!("#{} -> {}", self.instance_id, new_data);
        self.session.on_downstream_data(new_data)?;
        Ok(network::FilterStatus::Continue)
    }

    fn on_upstream_data(
        &mut self,
        data_size: usize,
        _end_of_stream: bool,
        ops: &dyn network::UpstreamDataOps,
    ) -> Result<network::FilterStatus> {
        if self.session.mode() == Mode::PassThrough {
            // has fallen back into no-op mode, e.g. due to a parsing error or
            // because of STARTTLS command
            return Ok(network::FilterStatus::Continue);
        }
        let new_data = ops.upstream_data(0, data_size)?;
        log::debug!("#{} <- {}", self.instance_id, new_data);
        self.session.on_upstream_data(new_data)?;
        Ok(network::FilterStatus::Continue)
    }
}
