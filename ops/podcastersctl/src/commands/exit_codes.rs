use core::fmt;

#[derive(Debug)]
pub enum _ExitCode {
    Success = 0,
    OperationalFailure = 1,
    CliMisuse = 2,
    ConfigInvalid = 3,
    DependencyError = 4,
    DeploymentFailure = 5,
    BackupFailure = 6,
}

impl fmt::Display for _ExitCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "Success"),
            Self::OperationalFailure => write!(f, "Operational Failure"),
            Self::CliMisuse => write!(f, "CLI/Parser misuse"),
            Self::ConfigInvalid => write!(f, "Configuration Invalid"),
            Self::DependencyError => write!(f, "Dependency Error"),
            Self::DeploymentFailure => write!(f, "Deployment Failure"),
            Self::BackupFailure => write!(f, "Backup failure"),
        }
    }
}
