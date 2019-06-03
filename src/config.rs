use serde::{Deserialize};

#[derive(Deserialize, Debug)]
pub struct Config {
  global: Global,
  pub apis: Vec<Api>
}

#[derive(Deserialize, Debug)]
pub struct Global {
  interval: i32
}

#[derive(Deserialize, Debug)]
pub struct Api {
  name: String,
  pub url: String
}