use rucat_common::error::{Result, RucatError};

#[derive(PartialEq, Eq)]
enum StorageMode {
    DiskOnly,
    MemoryOnly,
    MemoryAndDisk,
}

impl StorageMode {
    /// whether to drop the RDD to disk if it falls out of memory
    fn use_disk(&self) -> bool {
        self == &Self::DiskOnly || self == &Self::MemoryAndDisk
    }
    /// whether to use memory
    fn use_memory(&self) -> bool {
        self == &Self::MemoryOnly || self == &Self::MemoryAndDisk
    }
}

/// Storage of an RDD
#[derive(PartialEq, Eq)]
pub struct StorageLevel {
    mode: StorageMode,
    /// whether to replicate the RDD partitions on mulitple nodes
    replication: u8,
}

impl StorageLevel {
    const MAX_REPLICATION: u8 = 32;
    fn new(mode: StorageMode, replication: u8) -> Result<Self> {
        if replication == 0 {
            RucatError::IllegalArgument("replication should > 0".to_string()).into()
        } else if replication > Self::MAX_REPLICATION {
            RucatError::IllegalArgument(format!("replication should < {}.", Self::MAX_REPLICATION))
                .into()
        } else {
            Result::Ok(Self { mode, replication })
        }
    }
}
