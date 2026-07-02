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
    shell
        .jobs
        .retain_mut(|job| match job.child.try_wait().unwrap() {
            Some(_) => {
                println!("[{}]+  Done                 {}", job.id, job.token);
                false
            }
            None => true,
        });
}
