use clap::ValueEnum;

/// The possible `partitioner` configuration value that the [`librdkafka`](https://github.com/edenhill/librdkafka) library can handle.
///
/// The documentation is lifted directly from the `librdkafka` configuration
/// [page](https://github.com/edenhill/librdkafka/blob/master/CONFIGURATION.md).
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[value(rename_all = "snake_case")]
pub enum PartitionerConfig {
    /// Random distribution.
    Random,

    /// CRC32 hash of key (Empty and NULL keys are mapped to single partition).
    Consistent,

    /// CRC32 hash of key (Empty and NULL keys are randomly partitioned).
    ConsistentRandom,

    /// Java Producer compatible Murmur2 hash of key (NULL keys are mapped to single partition).
    Murmur2,

    /// Java Producer compatible Murmur2 hash of key (NULL keys are randomly partitioned).
    /// This is functionally equivalent to the default partitioner in the Java Producer.
    Murmur2Random,

    /// FNV-1a hash of key (NULL keys are mapped to single partition).
    Fnv1a,

    /// FNV-1a hash of key (NULL keys are randomly partitioned).
    Fnv1aRandom,
}

impl PartitionerConfig {
    pub fn name(&self) -> String {
        match self {
            PartitionerConfig::Random => "random".to_string(),
            PartitionerConfig::Consistent => "consistent".to_string(),
            PartitionerConfig::ConsistentRandom => "consistent_random".to_string(),
            PartitionerConfig::Murmur2 => "murmur2".to_string(),
            PartitionerConfig::Murmur2Random => "murmur2_random".to_string(),
            PartitionerConfig::Fnv1a => "fnv1a".to_string(),
            PartitionerConfig::Fnv1aRandom => "fnv1a_random".to_string(),
        }
    }
}
