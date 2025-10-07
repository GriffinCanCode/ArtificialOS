/*!
 * Scheduler Task - Autonomous Preemptive Scheduling
 *
 * Intelligent background task that enforces time-quantum-based preemption.
 * Unlike traditional cooperative schedulers, this provides true preemptive
 * behavior by running independently and adapting to system state.
 */

use super::preemption::PreemptionController;
use super::scheduler::Scheduler;
use log::{info, warn};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

/// Control messages for the scheduler task
#[derive(Debug, Clone)]
pub enum SchedulerCommand {
    /// Update the scheduling interval (new quantum in microseconds)
    UpdateQuantum(u64),
    /// Pause automatic scheduling
    Pause,
    /// Resume automatic scheduling
    Resume,
    /// Trigger immediate schedule check
    Trigger,
    /// Shutdown the scheduler task
    Shutdown,
}

/// Handle to the scheduler background task
pub struct SchedulerTask {
    command_tx: mpsc::UnboundedSender<SchedulerCommand>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl SchedulerTask {
    /// Spawn a new scheduler task that runs independently (basic mode)
    pub fn spawn(scheduler: Arc<RwLock<Scheduler>>) -> Self {
        Self::spawn_with_preemption(scheduler, None)
    }

    /// Spawn a scheduler task with optional OS-level preemption
    pub fn spawn_with_preemption(
        scheduler: Arc<RwLock<Scheduler>>,
        preemption: Option<Arc<PreemptionController>>,
    ) -> Self {
        let (command_tx, command_rx) = mpsc::unbounded_channel();

        let mode = if preemption.is_some() {
            "OS-level preemption"
        } else {
            "logical scheduling"
        };

        let handle = tokio::spawn(async move {
            run_scheduler_loop(scheduler, preemption, command_rx).await;
        });

        info!("Scheduler task spawned - autonomous {} enabled", mode);

        Self {
            command_tx,
            handle: Some(handle),
        }
    }

    /// Update the scheduling quantum (triggers immediate reconfiguration)
    pub fn update_quantum(&self, quantum_micros: u64) {
        let _ = self
            .command_tx
            .send(SchedulerCommand::UpdateQuantum(quantum_micros));
    }

    /// Pause automatic scheduling (processes can still yield manually)
    pub fn pause(&self) {
        let _ = self.command_tx.send(SchedulerCommand::Pause);
    }

    /// Resume automatic scheduling
    pub fn resume(&self) {
        let _ = self.command_tx.send(SchedulerCommand::Resume);
    }

    /// Trigger an immediate scheduling decision
    pub fn trigger(&self) {
        let _ = self.command_tx.send(SchedulerCommand::Trigger);
    }

    /// Shutdown the scheduler task gracefully
    pub async fn shutdown(mut self) {
        let _ = self.command_tx.send(SchedulerCommand::Shutdown);

        if let Some(handle) = self.handle.take() {
            if let Err(e) = handle.await {
                warn!("Scheduler task shutdown error: {}", e);
            } else {
                info!("Scheduler task shutdown complete");
            }
        }
    }
}

/// Core scheduler loop - intelligent and adaptive
async fn run_scheduler_loop(
    scheduler: Arc<RwLock<Scheduler>>,
    preemption: Option<Arc<PreemptionController>>,
    mut command_rx: mpsc::UnboundedReceiver<SchedulerCommand>,
) {
    let mut active = true;
    let initial_quantum = {
        let sched = scheduler.read();
        sched.stats().quantum_micros
    };

    let mut interval = tokio::time::interval(Duration::from_micros(initial_quantum));
    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mode = if preemption.is_some() {
        "with OS preemption"
    } else {
        "logical only"
    };
    info!("Scheduler loop started with {}μs quantum ({})", initial_quantum, mode);

    loop {
        tokio::select! {
            // Periodic scheduling tick (fires at quantum rate)
            _ = interval.tick() => {
                if active && !scheduler.read().is_empty() {
                    // Use preemption controller if available, otherwise use basic scheduler
                    let next_pid = if let Some(ref ctrl) = preemption {
                        ctrl.schedule()
                    } else {
                        scheduler.read().schedule()
                    };

                    if let Some(pid) = next_pid {
                        log::trace!("Scheduler tick: PID {} active", pid);
                    }
                }
            }

            // Handle control commands
            Some(cmd) = command_rx.recv() => {
                match cmd {
                    SchedulerCommand::UpdateQuantum(new_quantum_micros) => {
                        info!(
                            "Scheduler quantum updated: {}μs",
                            new_quantum_micros
                        );

                        // Create new interval with updated quantum
                        interval = tokio::time::interval(Duration::from_micros(new_quantum_micros));
                        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                    }

                    SchedulerCommand::Pause => {
                        info!("Scheduler task paused");
                        active = false;
                    }

                    SchedulerCommand::Resume => {
                        info!("Scheduler task resumed");
                        active = true;
                    }

                    SchedulerCommand::Trigger => {
                        if !scheduler.read().is_empty() {
                            if let Some(ref ctrl) = preemption {
                                ctrl.schedule();
                            } else {
                                scheduler.read().schedule();
                            }
                            log::trace!("Manual scheduler trigger");
                        }
                    }

                    SchedulerCommand::Shutdown => {
                        info!("Scheduler task shutting down");
                        break;
                    }
                }
            }
        }
    }
}

impl Drop for SchedulerTask {
    fn drop(&mut self) {
        // Attempt graceful shutdown if handle still exists
        if self.handle.is_some() {
            let _ = self.command_tx.send(SchedulerCommand::Shutdown);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::types::SchedulingPolicy;

    #[tokio::test]
    async fn test_scheduler_task_lifecycle() {
        let scheduler = Arc::new(RwLock::new(Scheduler::new(SchedulingPolicy::Fair)));

        // Spawn task
        let task = SchedulerTask::spawn(scheduler.clone());

        // Let it run briefly
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Shutdown gracefully
        task.shutdown().await;
    }

    #[tokio::test]
    async fn test_quantum_update() {
        let scheduler = Arc::new(RwLock::new(Scheduler::new(SchedulingPolicy::RoundRobin)));
        let task = SchedulerTask::spawn(scheduler.clone());

        // Add a process
        scheduler.read().add(1, 5);

        // Update quantum
        task.update_quantum(5_000);

        // Let it run
        tokio::time::sleep(Duration::from_millis(20)).await;

        task.shutdown().await;
    }

    #[tokio::test]
    async fn test_pause_resume() {
        let scheduler = Arc::new(RwLock::new(Scheduler::new(SchedulingPolicy::Priority)));
        let task = SchedulerTask::spawn(scheduler.clone());

        scheduler.read().add(1, 5);
        scheduler.read().add(2, 3);

        // Pause
        task.pause();
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Resume
        task.resume();
        tokio::time::sleep(Duration::from_millis(10)).await;

        task.shutdown().await;
    }
}
