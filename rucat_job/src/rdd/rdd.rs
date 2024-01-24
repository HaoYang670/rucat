use std::task::Context;

use super::{partition::PartitionIndex, storage_level::StorageLevel};
use crate::{rdd::partition::Partition, task_context::TaskContext};

/// Element types of RDD
enum RDDElem {
    U8(u8),
}

/// define RDD as (Dependencies, RDDVariant) where
/// RDDVariant is the enum of all kinds of RDDs.
/// Unlilke the RDD in Spark, we don't define the context as an argument of RDD as it is `global`
struct RDD {
    dependencies: Vec<Dependency>,
    storage_level: Option<StorageLevel>,
    rdd_core: RDDVariant,
}

impl RDD {
    fn get_dependencies(&self) -> &Vec<Dependency> {
        &self.dependencies
    }

    fn compute(&self, split: Partition, context: Context) -> impl Iterator<Item = RDDElem> {
        std::iter::empty()
    }
}

enum RDDVariant {
    /// An RDD that applies the provided function to every partition of the parent RDD.
    MapPartitionRDD {
        f: Box<
            dyn Fn(
                TaskContext,
                PartitionIndex,
                dyn Iterator<Item = RDDElem>,
            ) -> dyn Iterator<Item = RDDElem>,
        >,
    },
}

/// [Dependency] cannnot be generic because there is no way for RDD to know the type of its each [Dependency]
/// [Dependency] can be converted to [RDD]
enum Dependency {
    Narrow,
    Shuffle,
    OneToOne,
    Range,
}
