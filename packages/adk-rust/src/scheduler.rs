use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub payload: String,
    pub strict_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: Uuid,
    pub result: String,
    pub success: bool,
    pub verified: bool,
}

/// Represents the Two-Machine scheduling environment.
/// Machine A (e.g., local planner/validator) and Machine B (e.g., heavy execution/inference).
pub struct TwoMachineScheduler {
    pub machine_a_tx: mpsc::Sender<Job>,
    pub machine_b_tx: mpsc::Sender<Job>,
    pub results_rx: Arc<Mutex<mpsc::Receiver<JobResult>>>,
}

impl TwoMachineScheduler {
    pub fn new() -> (Self, mpsc::Receiver<Job>, mpsc::Receiver<Job>, mpsc::Sender<JobResult>) {
        let (tx_a, rx_a) = mpsc::channel(100);
        let (tx_b, rx_b) = mpsc::channel(100);
        let (res_tx, res_rx) = mpsc::channel(100);

        let scheduler = Self {
            machine_a_tx: tx_a,
            machine_b_tx: tx_b,
            results_rx: Arc::new(Mutex::new(res_rx)),
        };

        (scheduler, rx_a, rx_b, res_tx)
    }

    /// Dispatch job based on requirement profile.
    pub async fn dispatch(&self, job: Job) -> Result<(), &'static str> {
        if job.strict_mode {
            // Strict verification jobs go to Machine A (The Validator)
            self.machine_a_tx.send(job).await.map_err(|_| "Failed to send to Machine A")
        } else {
            // Generative/heavy jobs go to Machine B (The Executor)
            self.machine_b_tx.send(job).await.map_err(|_| "Failed to send to Machine B")
        }
    }

    pub async fn poll_result(&self) -> Option<JobResult> {
        let mut rx = self.results_rx.lock().await;
        rx.recv().await
    }
}
