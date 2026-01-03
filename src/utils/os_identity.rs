use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TargetOS {
  Windows,
  Linux,
  MacOS,
  BSD,
  Unknown
}

impl TargetOS {
  pub fn _is_posix(&self) -> bool {
    matches!(self, Self::Linux | Self::MacOS | Self::BSD)
  }
}

pub fn identification() -> TargetOS {
  match env::consts::OS {
    "linux" => TargetOS::Linux,
    "macos" => TargetOS::MacOS,
    "windows" => TargetOS::Windows,
    "freebsd" | "openbsd" | "netbsd" | "dragonfly" => TargetOS::BSD,
    _ => TargetOS::Unknown,
  }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_os() {
        let os = identification();
        println!("Current OS: {:?}", os);
        assert_ne!(os, TargetOS::Unknown);
    }

    #[test]
    fn test_posix_check() {
        assert!(TargetOS::Linux._is_posix());
        assert!(TargetOS::MacOS._is_posix());
        assert!(!TargetOS::Windows._is_posix());
    }
}
