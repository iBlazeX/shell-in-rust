use crate::Shell;
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

pub fn reap(shell: &mut Shell) {
    let len = shell.jobs.len();

    for (i, job) in shell.jobs.iter_mut().enumerate() {
        let marker = match i {
            x if x + 1 == len => "+",
            x if x + 2 == len => "-",
            _ => " ",
        };

        if job.child.try_wait().unwrap().is_some() {
            println!("[{}]{}  Done                 {}", job.id, marker, job.token);
            job.status = JobStatus::Done;
        }
    }

    shell.jobs.retain(|job| job.status != JobStatus::Done);
}
