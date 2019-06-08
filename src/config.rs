use serde::{Serialize, Deserialize};
use serde_json::{Value};
use std::sync::mpsc::Sender;
use chrono::prelude::*;

#[derive(Deserialize, Debug)]
pub struct Config {
  pub db: String,
  pub apis: Vec<Api>
}

#[derive(Clone, Serialize, Deserialize, Debug)]
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
  pub api_data: Value,
  pub api: Api,
  pub time: DateTime<Utc>
}