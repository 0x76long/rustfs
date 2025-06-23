use sha2::Sha256;

use std::collections::HashMap;
use std::ops::Sub;
use time::OffsetDateTime;
use tracing::{error, warn};

use crate::heal::data_usage_cache::TierStats;

pub type DailyAllTierStats = HashMap<String, LastDayTierStats>;

#[derive(Clone)]
pub struct LastDayTierStats {
    bins: [TierStats; 24],
    updated_at: OffsetDateTime,
}

impl Default for LastDayTierStats {
    fn default() -> Self {
        Self {
            bins: Default::default(),
            updated_at: OffsetDateTime::now_utc(),
        }
    }
}

impl LastDayTierStats {
    pub fn add_stats(&mut self, ts: TierStats) {
        let mut now = OffsetDateTime::now_utc();
        self.forward_to(&mut now);

        let now_idx = now.hour() as usize;
        self.bins[now_idx] = self.bins[now_idx].add(&ts);
    }

    fn forward_to(&mut self, t: &mut OffsetDateTime) {
        if t.unix_timestamp() == 0 {
            *t = OffsetDateTime::now_utc();
        }

        let since = t.sub(self.updated_at).whole_hours();
        if since < 1 {
            return;
        }

        let (idx, mut last_idx) = (t.hour(), self.updated_at.hour());

        self.updated_at = *t;

        if since >= 24 {
            self.bins = [TierStats::default(); 24];
            return;
        }

        while last_idx != idx {
            last_idx = (last_idx + 1) % 24;
            self.bins[last_idx as usize] = TierStats::default();
        }
    }

    fn merge(&self, m: LastDayTierStats) -> LastDayTierStats {
        let mut cl = self.clone();
        let mut cm = m.clone();
        let mut merged = LastDayTierStats::default();

        if cl.updated_at.unix_timestamp() > cm.updated_at.unix_timestamp() {
            cm.forward_to(&mut cl.updated_at);
            merged.updated_at = cl.updated_at;
        } else {
            cl.forward_to(&mut cm.updated_at);
            merged.updated_at = cm.updated_at;
        }

        for (i, _) in cl.bins.iter().enumerate() {
            merged.bins[i] = cl.bins[i].add(&cm.bins[i]);
        }

        merged
    }
}

#[cfg(test)]
mod test {}
