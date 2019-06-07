use serde::{Deserialize};
use std::sync::mpsc::Sender;
use std::time::SystemTime;

#[derive(Deserialize, Debug)]
pub struct Config {
  pub apis: Vec<Api>
}

#[derive(Clone, Deserialize, Debug)]
pub struct Api {
  pub name: String,
  pub url: String,
  pub interval: u64
}

#[derive(Debug)]
pub struct Job {
  pub interval: u64,
  pub sender: Sender<JobResult>,
  pub apis: Vec<Api>
}

#[derive(Debug)]
pub struct JobResult {
  pub api: Api,
  pub time: SystemTime
}