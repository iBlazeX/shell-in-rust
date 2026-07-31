use std::collections::HashMap;

use crate::jobs::Job;

pub struct Shell {
    pub jobs: Vec<Job>,
    pub next_job_id: usize,
    pub history: Vec<String>,
    pub vars: HashMap<String, String>,
}
