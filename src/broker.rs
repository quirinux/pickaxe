use crate::job;
use crate::worker;
use crate::task;
use crate::result;
use crossbeam;
use crossbeam_channel::{unbounded, tick, Receiver};
use std::time::{Duration, Instant};
use std::thread;

#[derive(Debug)]
pub struct Broker {
  job: job::Job,
  task_sender: crossbeam::channel::Sender<task::Task>,
  task_receiver: crossbeam::channel::Receiver<task::Task>,
  result_sender: crossbeam::channel::Sender<worker::WorkResult>,
  result_receiver: crossbeam::channel::Receiver<worker::WorkResult>,
  result: result::Result,
  started: Instant,
}

pub fn new(job: job::Job) -> Broker {
  let (ts, tr) = unbounded();
  let (rs, rr) = unbounded();
  Broker{
    job: job,
    task_sender: ts,
    task_receiver: tr,
    result_sender: rs,
    result_receiver: rr,
    result: result::new(),
    started: Instant::now(),
  }
}


impl Broker {


  fn spawn_workers(&mut self) {
    for t in 0..self.job.concurrency {
      let recv = self.task_receiver.clone();
      let send = self.result_sender.clone();
      thread::Builder::new()
        .name(t.to_string())
        .stack_size(128 * 1024)
        .spawn(move || worker::new(send, recv).run());
    }
  }

  fn kill_workers(&mut self) {}

  fn gather_result(&mut self) {
    for wr in self.result_receiver.try_iter() {
      self.result.work_result_handler(wr);
    }
  }

  fn schedule_task(&self) {
    let task = task::Task{
      url: self.job.url.clone(),
      time_out: self.job.time_out,
    };
    self.task_sender.send(task);
  }

  fn is_time_up(&self) -> bool {
    self.started.elapsed() >= Duration::from_secs(self.job.duration as u64)
  }

  pub fn run(&mut self) {
    let mut ratio = self.job.concurrency * self.job.ratio;
    self.started = Instant::now();
    // println!("ratio={}, task_sender.len:{}", ratio, self.task_sender.len());
    self.spawn_workers();
    loop{
        if self.task_sender.len() == 0 {
          // increasing the queue in 10% everytime it gets empty
          //println!("Increasing the queue from:{} to:{}", ratio, ratio+(ratio/10));
          ratio += ratio/10; 
        }

        for _ in 0..ratio-self.task_sender.len() {
      if !self.is_time_up() {
          self.schedule_task();
        }
      }
      if self.is_time_up() {
        self.result.duration = self.started.elapsed();
        drop(self.task_sender.clone());
        self.kill_workers();
        self.gather_result();
        drop(self.result_sender.clone());
        self.result.ratio = ratio;
        self.result.concurrency_level = self.job.concurrency;
        break;
      } else {
        self.gather_result();
      }
    } 
  }

  pub fn get_result(&self) -> &result::Result {
    &self.result
  }
}
