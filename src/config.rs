use serde::{Deserialize};
use std::sync::mpsc::Sender;

#[derive(Deserialize, Debug)]
pub struct Config {
  pub apis: Vec<Api>
}

#[derive(Clone, Deserialize, Debug)]
pub struct Api {
  pub name: String,
  pub url: String,
  pub interval: u32
}

#[derive(Debug)]
pub struct Job {
  pub interval: u32,
  pub sender: Sender<Api>,
  pub apis: Vec<Api>
}