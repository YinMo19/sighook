use libc::c_int;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SigHookError {
    InvalidAddress,
    PageSizeUnavailable,
    ProtectWritableFailed {
        kr: libc::kern_return_t,
        errno: c_int,
    },
    ProtectExecutableFailed {
        kr: libc::kern_return_t,
        errno: c_int,
    },
    SigEmptySetFailed {
        signum: c_int,
        errno: c_int,
    },
    SigActionFailed {
        signum: c_int,
        errno: c_int,
    },
    InstrumentSlotsFull,
    BranchOutOfRange,
    MmapFailed {
        errno: c_int,
    },
    TrampolineProtectFailed {
        errno: c_int,
    },
}

impl fmt::Display for SigHookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SigHookError::InvalidAddress => write!(f, "invalid address"),
            SigHookError::PageSizeUnavailable => write!(f, "page size unavailable"),
            SigHookError::ProtectWritableFailed { kr, errno } => {
                write!(
                    f,
                    "mach_vm_protect writable failed (kr={kr}, errno={errno})"
                )
            }
            SigHookError::ProtectExecutableFailed { kr, errno } => {
                write!(
                    f,
                    "mach_vm_protect executable failed (kr={kr}, errno={errno})"
                )
            }
            SigHookError::SigEmptySetFailed { signum, errno } => {
                write!(f, "sigemptyset failed (signum={signum}, errno={errno})")
            }
            SigHookError::SigActionFailed { signum, errno } => {
                write!(f, "sigaction failed (signum={signum}, errno={errno})")
            }
            SigHookError::InstrumentSlotsFull => write!(f, "instrument slots are full"),
            SigHookError::BranchOutOfRange => write!(f, "branch target out of range"),
            SigHookError::MmapFailed { errno } => write!(f, "mmap failed (errno={errno})"),
            SigHookError::TrampolineProtectFailed { errno } => {
                write!(f, "trampoline mprotect failed (errno={errno})")
            }
        }
    }
}

impl std::error::Error for SigHookError {}
