use reqwest;
use crate::task;
use crossbeam::{RecvError};
use std::time::{Duration, Instant};

pub enum WorkResult {
  Ok{
    status_code: reqwest::StatusCode,
    duration: Duration,
    url: url::Url,
  },
  Fail(),
  TimeOut(),
}

pub struct Worker {
  sender: crossbeam::channel::Sender<WorkResult>,
  receiver: crossbeam::channel::Receiver<task::Task>,
}

pub fn new(sender: crossbeam::channel::Sender<WorkResult>, receiver: crossbeam::channel::Receiver<task::Task>) -> Worker {
  Worker{
    sender: sender,
    receiver: receiver,
  }
}

impl Worker {
  pub fn run(self) {
    let client = reqwest::blocking::Client::new();
    loop {
      match self.receiver.recv() {
        Ok(task) => {
          // let mut res = reqwest::blocking::get("https://www.rust-lang.org/").unwrap();
          //let mut res = reqwest::blocking::get(&task.url).unwrap();
          let duration = Instant::now();
          let time_out = Duration::from_secs(task.time_out as u64);
          match client.get(&task.url).timeout(time_out).send() {
            Ok(res) => self.sender.send(WorkResult::Ok{ status_code: res.status(), duration: duration.elapsed(), url: res.url().clone()}),
            Err(e) => {
              if e.is_timeout() {
                self.sender.send(WorkResult::TimeOut())
              } else {
                self.sender.send(WorkResult::Fail())
              }
            },
          };
        },
        Err(RecvError) => break,
      }
    }
  }
}
