/*!
 * Process Command Validation
 * Security validation for process commands and arguments
 */

use super::types::{ProcessError, ProcessResult};
use std::path::Path;

/// Validate command for security issues
pub(super) fn validate_command(command: &str) -> ProcessResult<()> {
    // Empty command
    if command.is_empty() {
        return Err(ProcessError::PermissionDenied(
            "Command cannot be empty".to_string(),
        ));
    }

    // Path traversal in command itself
    if command.contains("..") {
        return Err(ProcessError::PermissionDenied(
            "Command contains path traversal".to_string(),
        ));
    }

    // Check for absolute path vs command name
    let is_absolute = Path::new(command).is_absolute();

    // Shell injection prevention - check for dangerous characters
    let dangerous_chars = [';', '|', '&', '\n', '\r', '\0', '`', '$', '(', ')'];
    if dangerous_chars.iter().any(|&c| command.contains(c)) {
        return Err(ProcessError::PermissionDenied(
            "Command contains shell injection characters".to_string(),
        ));
    }

    // URL encoding bypass attempts
    let encoded_dangerous = ["%3b", "%7c", "%26", "%24", "%60"];
    let command_lower = command.to_lowercase();
    for pattern in &encoded_dangerous {
        if command_lower.contains(pattern) {
            return Err(ProcessError::PermissionDenied(
                "Command contains encoded shell metacharacters".to_string(),
            ));
        }
    }

    // Allow absolute paths or command names
    if is_absolute {
        validate_absolute_command_path(command)?;
    }

    Ok(())
}

/// Validate absolute command path
fn validate_absolute_command_path(command: &str) -> ProcessResult<()> {
    if contains_path_traversal(command) {
        return Err(ProcessError::PermissionDenied(
            "Command path contains traversal".to_string(),
        ));
    }

    // Check if path is in allowed directories
    let allowed_paths = [
        "/bin/",
        "/usr/bin/",
        "/usr/local/bin/",
        "/sbin/",
        "/usr/sbin/",
    ];

    let is_allowed = allowed_paths
        .iter()
        .any(|prefix| command.starts_with(prefix));

    if !is_allowed {
        return Err(ProcessError::PermissionDenied(format!(
            "Command path not in allowed directories: {}",
            command
        )));
    }

    Ok(())
}

/// Validate command argument for security
pub(super) fn validate_argument(arg: &str) -> ProcessResult<()> {
    // Check for direct .. usage
    if arg.contains("..") {
        return Err(ProcessError::PermissionDenied(
            "Argument contains path traversal".to_string(),
        ));
    }

    check_encoded_traversal(arg)?;
    check_shell_injection(arg)?;
    check_encoded_metacharacters(arg)?;

    Ok(())
}

/// Check for encoded traversal patterns
fn check_encoded_traversal(arg: &str) -> ProcessResult<()> {
    let bypass_patterns = [
        "%2e%2e", // URL encoded ..
        "%252e",  // Double encoded .
        "..%2f",  // Encoded slash variants
        "%2e.",   // Partial encoding
        ".%2e",   // Partial encoding
        "..\\",   // Windows-style path traversal
        "\\..",   // Windows-style path traversal
        "%5c..",  // Encoded backslash
        "..%5c",  // Encoded backslash
    ];

    let arg_lower = arg.to_lowercase();
    for pattern in &bypass_patterns {
        if arg_lower.contains(&pattern.to_lowercase()) {
            return Err(ProcessError::PermissionDenied(
                "Argument contains path traversal attempt".to_string(),
            ));
        }
    }

    Ok(())
}

/// Check for shell injection characters
fn check_shell_injection(arg: &str) -> ProcessResult<()> {
    let dangerous_chars = [';', '|', '&', '\n', '\r', '\0', '`', '$'];
    if dangerous_chars.iter().any(|&c| arg.contains(c)) {
        return Err(ProcessError::PermissionDenied(
            "Argument contains shell injection characters".to_string(),
        ));
    }
    Ok(())
}

/// Check for encoded shell metacharacters
fn check_encoded_metacharacters(arg: &str) -> ProcessResult<()> {
    let encoded_dangerous = [
        "%3b", // ;
        "%7c", // |
        "%26", // &
        "%24", // $
        "%60", // `
    ];

    let arg_lower = arg.to_lowercase();
    for pattern in &encoded_dangerous {
        if arg_lower.contains(pattern) {
            return Err(ProcessError::PermissionDenied(
                "Argument contains encoded shell metacharacters".to_string(),
            ));
        }
    }

    Ok(())
}

/// Helper to detect path traversal patterns
pub(super) fn contains_path_traversal(path: &str) -> bool {
    // Try to normalize the path and see if it goes upward
    let parts: Vec<&str> = path.split('/').collect();
    let mut depth = 0;

    for part in parts {
        match part {
            ".." => {
                if depth > 0 {
                    depth -= 1;
                } else {
                    // Attempting to go above root
                    return true;
                }
            }
            "." | "" => {
                // Current dir or empty, no change
            }
            _ => {
                depth += 1;
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_traversal_detection() {
        // Should detect traversal
        assert!(contains_path_traversal("../../etc/passwd"));
        assert!(contains_path_traversal("../../../root"));

        // Should not detect traversal
        assert!(!contains_path_traversal("/usr/bin/ls"));
        assert!(!contains_path_traversal("./subdir/file"));
        assert!(!contains_path_traversal("dir/../file")); // Normalizes within
    }

    #[test]
    fn test_command_validation() {
        // Valid commands
        assert!(validate_command("/bin/ls").is_ok());
        assert!(validate_command("/usr/bin/python3").is_ok());

        // Invalid commands
        assert!(validate_command("").is_err());
        assert!(validate_command("ls; rm -rf /").is_err());
        assert!(validate_command("cmd && evil").is_err());
        assert!(validate_command("../../bin/bad").is_err());
    }

    #[test]
    fn test_argument_validation() {
        // Valid arguments
        assert!(validate_argument("-la").is_ok());
        assert!(validate_argument("/home/user/file.txt").is_ok());

        // Invalid arguments
        assert!(validate_argument("../../etc/passwd").is_err());
        assert!(validate_argument("file; rm -rf /").is_err());
        assert!(validate_argument("arg | evil").is_err());
        assert!(validate_argument("%2e%2e/etc/passwd").is_err());
    }
}
