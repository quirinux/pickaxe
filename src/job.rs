use serde::{Serialize, Deserialize};
use url::Url;
use std::path::Path;

const DEFAULT_CONCURRENCY : usize = 1;
const DEFAULT_DURATION : usize = 10;
const DEFAULT_TIMEOUT : usize = 60;
const DEFAULT_RATIO : usize = 10;

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Job {
  #[serde(default = "default_concurrency")]
  pub concurrency : usize,
  #[serde(default = "default_duration")]
  pub duration : usize,
  #[serde(default = "default_time_out")]
  pub time_out : usize,
  #[serde(default = "default_ratio")]
  pub ratio : usize,
  #[serde(default)]
  pub url : String,
}

impl Default for Job {
  fn default() -> Self {
    Job{
      concurrency: DEFAULT_CONCURRENCY,
      duration: DEFAULT_DURATION,
      time_out: DEFAULT_TIMEOUT,
      ratio: DEFAULT_RATIO,
      url: "".to_string(),
    }
  }
}

fn default_concurrency() -> usize {
  DEFAULT_CONCURRENCY
}

fn default_duration() -> usize {
  DEFAULT_DURATION
}

fn default_time_out() -> usize {
  DEFAULT_TIMEOUT
}

fn default_ratio() -> usize {
  DEFAULT_RATIO
}

fn from_file(file_path: &str) -> Result<Job, Box<dyn std::error::Error>> {
  let f = std::fs::File::open(file_path)?;
  let job : Job = serde_yaml::from_reader(&f)?;
  Url::parse(&job.url)?;
  Ok(job)
}

pub fn new(target: String) -> Result<Job, Box<dyn std::error::Error>> {
  if  Path::new(&target).exists() {
    from_file(&target)
  } else {
    Url::parse(&target)?;
    Ok(Job{
      url: target,
      ..Job::default()
    })
  }
}


