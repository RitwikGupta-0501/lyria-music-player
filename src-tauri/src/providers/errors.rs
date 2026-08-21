/// Structured error taxonomy for the Lua provider sandbox.
///
/// All variants ultimately serialize to `Result<T, String>` for the Tauri IPC
/// bridge, but with a structured prefix so the frontend can display the right
/// toast / error message without parsing free-form text.
#[derive(Debug)]
pub enum SandboxError {
    InstructionLimitExceeded { script: String },
    MemoryLimitExceeded { script: String },
    ExecutionTimeout { script: String, timeout_secs: u64 },
    ResponseTooLarge { url: String, limit_bytes: usize },
    ForbiddenUrl { url: String, reason: String },
    ScriptError { script: String, message: String },
    NetworkError { message: String },
}

impl std::fmt::Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InstructionLimitExceeded { script } =>
                write!(f, "SANDBOX_INSTRUCTION_LIMIT: provider '{}' exceeded CPU budget", script),
            Self::MemoryLimitExceeded { script } =>
                write!(f, "SANDBOX_MEMORY_LIMIT: provider '{}' exceeded memory budget", script),
            Self::ExecutionTimeout { script, timeout_secs } =>
                write!(f, "SANDBOX_TIMEOUT: provider '{}' timed out after {}s", script, timeout_secs),
            Self::ResponseTooLarge { url, limit_bytes } =>
                write!(f, "SANDBOX_RESPONSE_TOO_LARGE: response from '{}' exceeded {}B", url, limit_bytes),
            Self::ForbiddenUrl { url, reason } =>
                write!(f, "SANDBOX_FORBIDDEN_URL: '{}' blocked — {}", url, reason),
            Self::ScriptError { script, message } =>
                write!(f, "SANDBOX_SCRIPT_ERROR: provider '{}' — {}", script, message),
            Self::NetworkError { message } =>
                write!(f, "SANDBOX_NETWORK_ERROR: {}", message),
        }
    }
}

impl std::error::Error for SandboxError {}

