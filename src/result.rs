use std::time::{Duration};
use crate::worker::{WorkResult};
use std::collections::{BTreeMap};

#[derive(Debug, Default, Clone)]
pub struct Result {
  pub url: Option<url::Url>,
  pub success: usize,
  pub failed: usize,
  pub timed_out: usize,
  pub duration: std::time::Duration,
  pub status_code_summary: BTreeMap<reqwest::StatusCode, usize>,
  pub duration_summary: BTreeMap<std::time::Duration, usize>,
  pub ratio: usize,
  pub concurrency_level: usize,
}

pub fn new() -> Result {
  Result::default()
}


impl Result {

  fn truncate_duration(&self, d: Duration) -> Duration {
    let secs = d.as_secs();
    let mut millis = d.as_millis();
    let precision = 10_u128;
    while millis >= 10 {
      millis /= precision;
    }
    Duration::from_millis(secs * 1000 + (millis * 100) as u64)
  }

  pub fn work_result_handler(&mut self, wr: WorkResult) {
    match wr {
      WorkResult::Ok{status_code, duration, url} => {
        self.success += 1;
        *self.status_code_summary
          .entry(status_code)
          .or_insert(0) += 1;
        *self.duration_summary
          .entry(self.truncate_duration(duration))
          .or_insert(0) += 1;
        self.url = Some(url);
      },
      WorkResult::Fail() => self.failed += 1,
      WorkResult::TimeOut() => self.timed_out += 1,
    }
  }

  pub fn total(&self) -> usize {
    self.success + self.failed + self.timed_out
  }

  pub fn url_str(&self) -> String {
    match &self.url {
      Some(url) => String::from(url.as_str()),
      None => String::from(""),
    }
  }
}
