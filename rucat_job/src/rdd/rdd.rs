use super::storage_level::StorageLevel;

trait Partition {
    /// Get the identifier for a partition in an RDD
    fn index(&self) -> u8;
}

trait RDD<T> {
    /// Make `Partition` an associated type because each type of
    /// [RDD] only has one type of [Partition]
    type Partition: Partition;
    fn get_dependencies(&self) -> Vec<Dependency>;
    fn get_partitions(&self) -> Vec<Self::Partition>;
    fn get_storage_level(&self) -> Option<StorageLevel>;
    fn compute(&self, split: Self::Partition) -> impl Iterator<Item = T>;
}

/// [Dependency] cannnot be generic because there is no way for RDD to know the type of its each [Dependency]
/// [Dependency] can be converted to [RDD]
enum Dependency {
    Narrow,
    Shuffle,
    OneToOne,
    Range,
}
