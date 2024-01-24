/// The deterministic level of RDD's output
enum DeterministicLevel {
    /// The RDD output is always the same data set in the same order after a rerun.
    Determinate,
    /// The RDD output is always the same data set but the order can be different after a rerun.
    Unordered,
    /// The RDD output can be different after a rerun.
    Indeterminate,
}
