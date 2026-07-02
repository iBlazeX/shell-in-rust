use std::process::Child;

pub struct Job {
    pub id: usize,
    pub child: Child,
    pub token: String,
    pub status: JobStatus,
}
#[derive(Debug, PartialEq)]
pub enum JobStatus {
    Running,
    Done,
}
