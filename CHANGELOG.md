# vX.Y.Z (20YY-MM-DD) [UNRELEASED]

## Breaking Changes

## Features

## Enhancements

* Providing binary as part of the release process: `x86_64-apple-darwin` (#36)
* Providing binary as part of the release process: `x86_64-unknown-linux-gnu` (#35)
* Introducing (this) `CHANGELOG.md`, and adding it to the release packages (#19)

## Bug Fixes

## Notes

* Added usage instructions (`-h` / `--help`) to `README.md`
* Multiple dependencies updates

# v0.1.0 -> v0.1.4 (2022-11-01)

## Features

* Kafka records production described in 4 "phases" that repeat in circle: `min`, `up`, `max` and `down`
* All phases are configured in terms of _seconds_ (i.e. _duration of a phase_) and _records per second_ (i.e. _workload during a phase_)
* `up` and `down` can be one of many transitions, each with a specific "shape" (ex. `linear`, `ease-in`, `spike-out`, ...)
* Records `key` and `payload` are configurable with _fixed_, _from-file_ and _randomly-generated_ values
* Records `headers` can be added to each record
* Internal Kafka producer is fully configurable, including selecting a partitioner
* Complete control of verbosity via `-v` / `-q`
* Extensive usage instructions via `-h` (compact) / `--help` (extended)

## Notes

* First functional release of Ksunami
* This changelog is being written retroactively