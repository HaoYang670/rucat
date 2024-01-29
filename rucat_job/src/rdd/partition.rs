pub(super) type PartitionIndex = u8;

/// An identifier for a partition in an RDD.
pub(super) enum Partition {
    Dummy,
}

impl Partition {
    /// Get the identifier for a partition in an RDD
    pub fn index(&self) -> PartitionIndex {
        match self {
            Partition::Dummy => 0,
        }
    }
}
