use super::rdd::RDD;

pub(super) struct Dependency {
    rdd: RDD,
    dependency_type: DependencyVariant,
}

impl Dependency {
    pub fn get_rdd(&self) -> &RDD {
        &self.rdd
    }
}

/// [Dependency] can be converted to [RDD]
pub(super) enum DependencyVariant {
    Narrow,
    Shuffle,
    OneToOne,
    Range,
}
