/*!
 * eBPF Program Loader
 * Platform-agnostic eBPF program loading and management
 */

use super::types::*;
use crate::core::sync::StripedMap;
use log::info;
use std::sync::Arc;

/// Program loader for managing eBPF programs
#[derive(Clone)]
pub struct ProgramLoader {
    programs: Arc<StripedMap<String, LoadedProgram>>,
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
            programs: Arc::new(StripedMap::new(16)),
        }
    }

    /// Load a program
    pub fn load(&self, config: ProgramConfig) -> EbpfResult<()> {
        let name_str = config.name.to_string();
        if self.programs.contains_key(&name_str) {
            return Err(EbpfError::LoadFailed {
                reason: format!("Program {} already loaded", config.name).into(),
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

        self.programs.insert(config.name.to_string(), program);
        info!("Loaded eBPF program: {}", config.name);
        Ok(())
    }

    /// Unload a program
    pub fn unload(&self, name: &str) -> EbpfResult<()> {
        // Check if attached before removing
        if let Some(attached) = self.programs.get(&name.to_string(), |p| p.attached) {
            if attached {
                return Err(EbpfError::LoadFailed {
                    reason: format!("Program {} is still attached", name).into(),
                });
            }
        }

        self.programs
            .remove(&name.to_string())
            .ok_or_else(|| EbpfError::ProgramNotFound { name: name.into() })?;

        info!("Unloaded eBPF program: {}", name);
        Ok(())
    }

    /// Attach a program
    pub fn attach(&self, name: &str) -> EbpfResult<()> {
        let result = self.programs.get_mut(&name.to_string(), |program| {
            if program.attached {
                Err(EbpfError::AttachFailed {
                    reason: format!("Program {} already attached", name).into(),
                })
            } else {
                program.attached = true;
                Ok(())
            }
        });

        match result {
            Some(Ok(())) => {
                info!("Attached eBPF program: {}", name);
                Ok(())
            }
            Some(Err(e)) => Err(e),
            None => Err(EbpfError::ProgramNotFound { name: name.into() }),
        }
    }

    /// Detach a program
    pub fn detach(&self, name: &str) -> EbpfResult<()> {
        let result = self.programs.get_mut(&name.to_string(), |program| {
            if !program.attached {
                Err(EbpfError::DetachFailed {
                    reason: format!("Program {} not attached", name).into(),
                })
            } else {
                program.attached = false;
                Ok(())
            }
        });

        match result {
            Some(Ok(())) => {
                info!("Detached eBPF program: {}", name);
                Ok(())
            }
            Some(Err(e)) => Err(e),
            None => Err(EbpfError::ProgramNotFound { name: name.into() }),
        }
    }

    pub fn list(&self) -> Vec<ProgramInfo> {
        let mut result = Vec::with_capacity(16);
        self.programs.iter(|_name, p| {
            result.push(ProgramInfo {
                name: p.config.name.clone(),
                program_type: p.config.program_type,
                attached: p.attached,
                events_captured: p.events_captured,
                created_at: p.created_at,
            });
        });
        result
    }

    /// Get program info
    pub fn get_info(&self, name: &str) -> Option<ProgramInfo> {
        self.programs.get(&name.to_string(), |p| ProgramInfo {
            name: p.config.name.clone(),
            program_type: p.config.program_type,
            attached: p.attached,
            events_captured: p.events_captured,
            created_at: p.created_at,
        })
    }

    /// Increment event counter for a program
    pub fn record_event(&self, name: &str) {
        self.programs.get_mut(&name.to_string(), |program| {
            program.events_captured += 1;
        });
    }
}

impl Default for ProgramLoader {
    fn default() -> Self {
        Self::new()
    }
}
