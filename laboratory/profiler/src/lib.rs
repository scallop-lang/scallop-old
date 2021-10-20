use std::collections::*;
use std::time::*;
use std::fs::File;
use std::io::prelude::*;

pub struct Profiler {
  file: File,
  pub times: HashMap<String, f64>,
  pub start_times: HashMap<String, Instant>,
}

impl Profiler {
  pub fn new(log_path: &str) -> Self {
    Self {
      file: File::create(log_path).unwrap(),
      times: HashMap::new(),
      start_times: HashMap::new(),
    }
  }

  pub fn log(&mut self, s: &str) {
    println!("{}", s);
    self.file.write_all(s.as_bytes()).unwrap();
    self.file.write_all(b"\n").unwrap();
  }

  pub fn start(&mut self, name: &str) {
    if !self.start_times.contains_key(name) {
      self.start_times.insert(name.to_string(), Instant::now());
    }
  }

  pub fn end(&mut self, name: &str) {
    match self.start_times.remove(name) {
      Some(start) => {
        self.increment(name, start.elapsed().as_secs_f64());
      },
      _ => {}
    }
  }

  fn increment(&mut self, name: &str, secs: f64) {
    *self.times.entry(name.to_string()).or_default() += secs;
  }

  pub fn log_profile_record(&mut self) {
    self.log("Profiler record:");
    let mut times = HashMap::new();
    for (name, duration) in &self.times {
      let duration = match &self.start_times.get(name) {
        Some(start) => duration + start.elapsed().as_secs_f64(),
        None => duration.clone(),
      };
      times.insert(name.clone(), duration);
    }
    for (name, start) in &self.start_times {
      if !self.times.contains_key(name) {
        times.insert(name.clone(), start.elapsed().as_secs_f64());
      }
    }
    for (name, duration) in times {
      self.log(&format!("- {}: {:7.4}s", name, duration));
    }
  }

  pub fn print(&self) {
    println!("Profiler record:");
    for (name, duration) in &self.times {
      println!("- {}: {:7.4}s", name, duration);
    }
  }
}
