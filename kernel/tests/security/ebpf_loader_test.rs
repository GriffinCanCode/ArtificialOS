/*!
 * eBPF Program Loader Unit Tests
 * Tests for program loading and management
 */

use ai_os_kernel::security::ebpf::{ProgramConfig, ProgramLoader, ProgramType};

#[test]
fn test_program_load() {
    let loader = ProgramLoader::new();

    let config = ProgramConfig {
        name: "test_prog".to_string(),
        program_type: ProgramType::SyscallEntry,
        auto_attach: false,
        enabled: true,
    };

    assert!(loader.load(config).is_ok());

    let programs = loader.list();
    assert_eq!(programs.len(), 1);
    assert_eq!(programs[0].name, "test_prog");
    assert!(!programs[0].attached);
}

#[test]
fn test_program_duplicate_load() {
    let loader = ProgramLoader::new();

    let config = ProgramConfig {
        name: "duplicate".to_string(),
        program_type: ProgramType::NetworkSocket,
        auto_attach: false,
        enabled: true,
    };

    assert!(loader.load(config.clone()).is_ok());
    // Loading same name should fail
    assert!(loader.load(config).is_err());
}

#[test]
fn test_program_attach_detach() {
    let loader = ProgramLoader::new();

    let config = ProgramConfig {
        name: "attachable".to_string(),
        program_type: ProgramType::FileOps,
        auto_attach: false,
        enabled: true,
    };

    assert!(loader.load(config).is_ok());

    // Should not be attached initially
    let info = loader.get_info("attachable").unwrap();
    assert!(!info.attached);

    // Attach
    assert!(loader.attach("attachable").is_ok());
    let info = loader.get_info("attachable").unwrap();
    assert!(info.attached);

    // Detach
    assert!(loader.detach("attachable").is_ok());
    let info = loader.get_info("attachable").unwrap();
    assert!(!info.attached);
}

#[test]
fn test_program_attach_already_attached() {
    let loader = ProgramLoader::new();

    let config = ProgramConfig {
        name: "test".to_string(),
        program_type: ProgramType::SyscallExit,
        auto_attach: false,
        enabled: true,
    };

    assert!(loader.load(config).is_ok());
    assert!(loader.attach("test").is_ok());

    // Attaching again should fail
    assert!(loader.attach("test").is_err());
}

#[test]
fn test_program_unload_attached() {
    let loader = ProgramLoader::new();

    let config = ProgramConfig {
        name: "test".to_string(),
        program_type: ProgramType::ProcessLifecycle,
        auto_attach: false,
        enabled: true,
    };

    assert!(loader.load(config).is_ok());
    assert!(loader.attach("test").is_ok());

    // Should not be able to unload while attached
    assert!(loader.unload("test").is_err());

    // Detach first
    assert!(loader.detach("test").is_ok());

    // Now unload should work
    assert!(loader.unload("test").is_ok());
}

#[test]
fn test_program_get_info() {
    let loader = ProgramLoader::new();

    let config = ProgramConfig {
        name: "info_test".to_string(),
        program_type: ProgramType::NetworkSocket,
        auto_attach: false,
        enabled: true,
    };

    assert!(loader.load(config).is_ok());

    let info = loader.get_info("info_test");
    assert!(info.is_some());

    let info = info.unwrap();
    assert_eq!(info.name, "info_test");
    assert_eq!(info.program_type, ProgramType::NetworkSocket);
    assert!(!info.attached);
    assert_eq!(info.events_captured, 0);

    // Non-existent program
    assert!(loader.get_info("nonexistent").is_none());
}

#[test]
fn test_program_list_multiple() {
    let loader = ProgramLoader::new();

    let types = vec![
        ProgramType::SyscallEntry,
        ProgramType::SyscallExit,
        ProgramType::NetworkSocket,
        ProgramType::FileOps,
        ProgramType::ProcessLifecycle,
    ];

    for (i, program_type) in types.iter().enumerate() {
        let config = ProgramConfig {
            name: format!("prog_{}", i),
            program_type: *program_type,
            auto_attach: false,
            enabled: true,
        };
        assert!(loader.load(config).is_ok());
    }

    let programs = loader.list();
    assert_eq!(programs.len(), 5);
}

#[test]
fn test_program_event_recording() {
    let loader = ProgramLoader::new();

    let config = ProgramConfig {
        name: "event_prog".to_string(),
        program_type: ProgramType::SyscallEntry,
        auto_attach: false,
        enabled: true,
    };

    assert!(loader.load(config).is_ok());

    // Record some events
    for _ in 0..10 {
        loader.record_event("event_prog");
    }

    let info = loader.get_info("event_prog").unwrap();
    assert_eq!(info.events_captured, 10);
}
