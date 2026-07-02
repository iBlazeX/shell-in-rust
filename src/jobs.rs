use std::process::Child;

pub struct Job {
    pub id: usize,
    pub child: Child,
    pub token: String,
    pub status: JobStatus,
}

pub enum JobStatus {
    Running,
    Stopped,
}
