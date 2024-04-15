# vNEXT (YYYY-MM-DD)

## Features

Setup GitHub Actions Workflow to publish Docker image kafkesc/ksunami at every release ([i#206](https://github.com/kafkesc/ksunami/issues/206))

## Notes

Multiple dependencies upgrades since previous release

# v0.1.8 (2023-06-18)

## Notes

Multiple dependencies updates since previous release

# v0.1.7 (2023-03-13)

## Features

* Docker: Ksunami is now available as image on Docker-Hub: [kafkesc/ksunami](https://hub.docker.com/r/kafkesc/ksunami) ([i#15](https://github.com/kafkesc/ksunami/issues/15))

## Notes

Multiple dependencies updates since previous release

# v0.1.6 (2023-01-18)

## Enhancements

* Reworked warning message that informs user when the internal records channel capacity is less than 20% ([commit](https://github.com/kafkesc/ksunami/commit/a8f7bee444ae59f5509ad4170c4f10c76a1ceb13))

## Notes

* Multiple dependencies updates
* Fixed annoying CI build shield ([commit](https://github.com/kafkesc/ksunami/commit/d07c1124b4630d4e495f1dd0413ba69d95d8db9f))
* Updated section about "License" in README
* Added section about "Contribution" in README
* Updated licenses model: Ksunami is now offered in dual license - both [Apache 2.0](LICENSE-APACHE) and [MIT](LICENSE-MIT)

# v0.1.5 (2022-12-14)

## Enhancements

* Providing binary as part of the release process: `x86_64-apple-darwin` ([i#36](https://github.com/kafkesc/ksunami/issues/36))
* Providing binary as part of the release process: `x86_64-unknown-linux-gnu` ([i#35](https://github.com/kafkesc/ksunami/issues/35))
* Introducing (this) `CHANGELOG.md`, and adding it to the release packages ([i#19](https://github.com/kafkesc/ksunami/issues/19))
* Publishing to [crates.io](https://crates.io/crates/ksunami) when a tag is pushed ([i#20](https://github.com/kafkesc/ksunami/issues/20))
* Add examples to `README.md` ([i#40](https://github.com/kafkesc/ksunami/issues/40))

## Notes

* Added usage instructions (`-h` / `--help`) to `README.md`
* Multiple dependencies updates
* Published first blogpost announcing Ksunami ([i#11](https://github.com/kafkesc/ksunami/issues/11))

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
