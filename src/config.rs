use serde::{Deserialize};

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
  pub apis: Vec<Api>
}