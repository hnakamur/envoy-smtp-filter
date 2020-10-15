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
use std::rc::Rc;

use envoy::extension::{factory, ConfigStatus, ExtensionFactory, InstanceId, Result};
use envoy::host::{ByteString, Stats};

use super::config::SmtpFilterConfig;
use super::filter::SmtpFilter;
use super::stats::SmtpFilterStats;

/// Factory for creating SMTP Filter instances
/// (one filter instance per TCP connection).
pub struct SmtpFilterFactory<'a> {
    // Stats API implementation.
    stats: &'a dyn Stats,
    // Configuration shared by multiple filter instances.
    filter_config: Rc<SmtpFilterConfig>,
    // Stats shared by multiple filter instances.
    filter_stats: Rc<SmtpFilterStats<'a>>,
}

impl<'a> SmtpFilterFactory<'a> {
    /// Creates a new SmtpFilter factory.
    pub fn new(stats: &'a dyn Stats) -> Result<Self> {
        let config = SmtpFilterConfig::default();
        let filter_stats = SmtpFilterStats::new(config.detailed_stats, stats)?;
        // Inject dependencies on Envoy host APIs
        Ok(SmtpFilterFactory {
            stats,
            filter_config: Rc::new(config),
            filter_stats: Rc::new(filter_stats),
        })
    }

    /// Creates a new factory bound to the actual Envoy ABI.
    pub fn default() -> Result<Self> {
        Self::new(Stats::default())
    }
}

impl<'a> ExtensionFactory for SmtpFilterFactory<'a> {
    type Extension = SmtpFilter<'a>;

    /// The reference name for the SMTP Filter.
    ///
    /// This name appears in `Envoy` configuration as a value of `root_id` field.
    fn name() -> &'static str {
        "tetratelabs.filters.network.smtp"
    }

    /// Is called when Envoy creates a new Listener that uses Smtp Network Filter.
    fn on_configure(
        &mut self,
        config: ByteString,
        _ops: &dyn factory::ConfigureOps,
    ) -> Result<ConfigStatus> {
        let filter_config = if config.is_empty() {
            SmtpFilterConfig::default()
        } else {
            SmtpFilterConfig::try_from(config.as_bytes())?
        };
        self.filter_config = Rc::new(filter_config);
        if self.filter_config.detailed_stats != self.filter_stats.is_detailed() {
            let filter_stats = SmtpFilterStats::new(self.filter_config.detailed_stats, self.stats)?;
            self.filter_stats = Rc::new(filter_stats);
        }
        Ok(ConfigStatus::Accepted)
    }

    /// Is called to create a unique instance of SMTP Filter
    /// for each TCP connection.
    fn new_extension(&mut self, instance_id: InstanceId) -> Result<Self::Extension> {
        Ok(SmtpFilter::new(
            instance_id,
            Rc::clone(&self.filter_config),
            Rc::clone(&self.filter_stats),
        ))
    }
}
