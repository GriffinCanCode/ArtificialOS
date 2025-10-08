/*!
 * eBPF Program Loader
 * Platform-agnostic eBPF program loading and management
 */

use super::types::*;
use log::{debug, info, warn};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

/// Program loader for managing eBPF programs
#[derive(Clone)]
pub struct ProgramLoader {
    programs: Arc<RwLock<HashMap<String, LoadedProgram>>>,
}

/// Loaded eBPF program metadata
struct LoadedProgram {
    config: ProgramConfig,
    attached: bool,
    events_captured: u64,
    created_at: u64,
}

impl ProgramLoader {
    pub fn new() -> Self {
        Self {
            programs: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load a program
    pub fn load(&self, config: ProgramConfig) -> EbpfResult<()> {
        let mut programs = self.programs.write();

        if programs.contains_key(&config.name) {
            return Err(EbpfError::LoadFailed {
                reason: format!("Program {} already loaded", config.name),
            });
        }

        let program = LoadedProgram {
            config: config.clone(),
            attached: false,
            events_captured: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0), // Fallback to 0 if system time is before Unix epoch
        };

        programs.insert(config.name.clone(), program);
        info!("Loaded eBPF program: {}", config.name);
        Ok(())
    }

    /// Unload a program
    pub fn unload(&self, name: &str) -> EbpfResult<()> {
        let mut programs = self.programs.write();

        if let Some(program) = programs.get(name) {
            if program.attached {
                return Err(EbpfError::LoadFailed {
                    reason: format!("Program {} is still attached", name),
                });
            }
        }

        programs.remove(name).ok_or_else(|| EbpfError::ProgramNotFound {
            name: name.to_string(),
        })?;

        info!("Unloaded eBPF program: {}", name);
        Ok(())
    }

    /// Attach a program
    pub fn attach(&self, name: &str) -> EbpfResult<()> {
        let mut programs = self.programs.write();

        let program = programs.get_mut(name).ok_or_else(|| EbpfError::ProgramNotFound {
            name: name.to_string(),
        })?;

        if program.attached {
            return Err(EbpfError::AttachFailed {
                reason: format!("Program {} already attached", name),
            });
        }

        program.attached = true;
        info!("Attached eBPF program: {}", name);
        Ok(())
    }

    /// Detach a program
    pub fn detach(&self, name: &str) -> EbpfResult<()> {
        let mut programs = self.programs.write();

        let program = programs.get_mut(name).ok_or_else(|| EbpfError::ProgramNotFound {
            name: name.to_string(),
        })?;

        if !program.attached {
            return Err(EbpfError::DetachFailed {
                reason: format!("Program {} not attached", name),
            });
        }

        program.attached = false;
        info!("Detached eBPF program: {}", name);
        Ok(())
    }

    /// List all programs
    pub fn list(&self) -> Vec<ProgramInfo> {
        let programs = self.programs.read();
        programs
            .values()
            .map(|p| ProgramInfo {
                name: p.config.name.clone(),
                program_type: p.config.program_type,
                attached: p.attached,
                events_captured: p.events_captured,
                created_at: p.created_at,
            })
            .collect()
    }

    /// Get program info
    pub fn get_info(&self, name: &str) -> Option<ProgramInfo> {
        let programs = self.programs.read();
        programs.get(name).map(|p| ProgramInfo {
            name: p.config.name.clone(),
            program_type: p.config.program_type,
            attached: p.attached,
            events_captured: p.events_captured,
            created_at: p.created_at,
        })
    }

    /// Increment event counter for a program
    pub fn record_event(&self, name: &str) {
        let mut programs = self.programs.write();
        if let Some(program) = programs.get_mut(name) {
            program.events_captured += 1;
        }
    }
}

impl Default for ProgramLoader {
    fn default() -> Self {
        Self::new()
    }
}
