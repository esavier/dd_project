use std::{collections::HashMap, sync::Arc};
use tokio::sync::{Mutex, Notify};
use hifitime::prelude::*;

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize, derivative::Derivative)]
#[derivative(Default(new="true"))]
pub struct WriteStatistics {
  pub total_bytes_written: u64,
  pub started_at: hifitime::Epoch,
  pub last_write_at: hifitime::Epoch,
  pub total_writes: u64,
  pub total_errors: u64,
}


#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize, derivative::Derivative)]
#[derivative(Default(new="true"))]
pub struct ReadStatistics {
  pub total_bytes_read: u64,
  pub started_at: hifitime::Epoch,
  pub last_read_at: hifitime::Epoch,
  pub total_reads: u64,
  pub total_errors: u64,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum TaskStatus {
  Pending,
  Running,
  Completed(TaskStatusMessage),
  Failed(TaskStatusMessage),
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize, derivative::Derivative)]
#[derivative(Default(new="true"))]
pub struct TaskStatusMessage {
  pub code: i64,
  pub message: String,
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize, serde::Serialize, derivative::Derivative)]
#[derivative(Default(new="true"))]
pub struct Task {
  #[derivative(Default(value="TaskStatus::Pending"))]
  pub status: TaskStatus,       // set by state.new_task()
  pub started_at: hifitime::Epoch,      // set by state.new_task()
  pub completed_at: Option<hifitime::Epoch>,    // updated by complete() or fail()
  pub worktime: hifitime::Duration,            // updated by ping() or display()
  pub last_updated_at: hifitime::Epoch, // usually updated by ping()
  pub return_code: i32,         // set by complete() or fail()
}

pub struct DdContext {  
  pub write_statistics: Arc<Mutex<WriteStatistics>>,
  pub read_statistics: Arc<Mutex<ReadStatistics>>,
  pub task_status: HashMap<String, Arc<Mutex<Task>>>,
  pub main_notifications: Arc<Notify>,
  pub sink_notifications: Arc<Notify>,
  pub source_notifications: Arc<Notify>,
}


impl DdContext {
  pub fn new() -> Self {
    DdContext {
      write_statistics: Arc::new(Mutex::new(WriteStatistics::new())),
      read_statistics: Arc::new(Mutex::new(ReadStatistics::new())),
      task_status: HashMap::new(),
      main_notifications: Arc::new(Notify::new()),
      source_notifications: Arc::new(Notify::new()),
      sink_notifications: Arc::new(Notify::new()),

    }
  }

  pub async fn new_task(&mut self, name: &str) -> Arc<Mutex<Task>> {
    let task = Task {
      status: TaskStatus::Pending,
      started_at: Epoch::now().unwrap(),
      worktime: 0.nanoseconds(),
      completed_at: None,
      last_updated_at: Epoch::now().unwrap(),
      return_code: 0,
    };
    self.task_status.insert(name.to_string(), Arc::new(Mutex::new(task)));
    self.task_status.get(name).unwrap().clone()
  }

  pub async fn are_tasks_pending(&self) -> bool {
    for (_, task) in self.task_status.iter() {
      let task = task.lock().await;
      match task.status {
        TaskStatus::Pending => return true,
        TaskStatus::Running => return true,
        _ => continue,
      }
    }
    false
  }

  pub async fn display_statistics(&self) {
    let read_statistics = self.read_statistics.lock().await;
    let write_statistics = self.write_statistics.lock().await;
    println!("Read Statistics: {}", serde_json::to_string_pretty(&*read_statistics).unwrap());
    println!("Write Statistics: {}", serde_json::to_string_pretty(&*write_statistics).unwrap());
  }

  pub async fn display_tasks(&self) {
    for (name, task) in self.task_status.iter() {
      let task = task.lock().await;
      println!("Task: {}, Status: {:?}", name, task.status);
    }
  }
}

impl Default for DdContext {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadStatistics {
  pub fn init(&mut self) {
    self.total_bytes_read = 0;
    self.started_at = Epoch::now().unwrap();
    self.last_read_at = Epoch::now().unwrap();
    self.total_reads = 0;
    self.total_errors = 0;
  }

  pub fn add_read(&mut self, bytes_read: u64) {
    self.total_reads += 1;
    self.total_bytes_read += bytes_read;
    self.last_read_at = Epoch::now().unwrap();
  }

  pub fn add_error(&mut self) {
    self.total_errors += 1;
  }
}

impl WriteStatistics {
  pub fn init(&mut self) {
    self.total_bytes_written = 0;
    self.started_at = Epoch::now().unwrap();
    self.last_write_at = Epoch::now().unwrap();
    self.total_writes = 0;
    self.total_errors = 0;
  }
  
  pub fn add_write(&mut self, bytes_written: u64) {
    self.total_writes += 1;
    self.total_bytes_written += bytes_written;
    self.last_write_at = Epoch::now().unwrap();
  }

  pub fn add_error(&mut self) {
    self.total_errors += 1;
  }
}

impl Task {

  pub fn ping(&mut self) {
    self.last_updated_at = Epoch::now().unwrap();
    self.update_worktime();
  }

  pub fn update_worktime(&mut self) {
    if self.completed_at.is_none() {
      self.worktime = self.last_updated_at - self.started_at
    } else {
      // safe_unwrap
      self.worktime = self.completed_at.unwrap() - self.started_at
    }
  }

  pub fn change_state(&mut self, status: TaskStatus) {
    self.ping();
    self.update_worktime();
    self.status = status;
  }

  pub fn complete(&mut self, return_code: i64) {
    self.completed_at = Some(Epoch::now().unwrap());
    self.ping();

    self.status = TaskStatus::Completed(TaskStatusMessage {
      code: return_code,
      message: "Task completed successfully".to_string(),
    });
  }

  pub fn fail(&mut self, return_code: i64) {
    self.completed_at = Some(Epoch::now().unwrap());
    self.ping();

    self.status = TaskStatus::Failed(TaskStatusMessage {
      code: return_code,
      message: "Task failed".to_string(),
    });
  }

  pub fn display(&mut self) {
    println!("{}", serde_json::to_string_pretty(self).unwrap());
  }



}