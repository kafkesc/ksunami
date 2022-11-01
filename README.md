# Kafkesc - Ksunami

<div style="text-align: center;">
<span style="font-weight: bold; font-style: italic; font-size: 1.5em;">Waves of Kafka Records!</span>

[![CI](https://img.shields.io/github/workflow/status/kafkesc/ksunami/CI?label=CI&logo=Github&style=flat-square)](https://github.com/kafkesc/ksunami/actions/workflows/ci.yml)
[![Apache 2.0](https://img.shields.io/crates/l/ksunami?logo=apache&style=flat-square)](https://github.com/kafkesc/ksunami/blob/main/LICENSE)
[![Crates.io downloads](https://img.shields.io/crates/d/ksunami?logo=rust&style=flat-square)](https://crates.io/crates/ksunami)
[![](https://img.shields.io/crates/v/ksunami?label=latest%20version&logo=rust&style=flat-square)](https://crates.io/crates/names/versions)
</div>

## What's Ksunami?

Ksunami is a command-line tool to produce a constant, configurable, cyclical stream of (dummy) records
against a [Kafka](https://kafka.apache.org/) Cluster Topic.

If you are experimenting with scalability and latency against Kafka, and are looking for ways to reproduce a continues
stream of records, following a specific traffic pattern that repeats periodically, **Ksunami** is the tool for you.

Ksunami offers a way to set up the production of records, expressing the scenario as a sequence of "phases"
that repeat indefinitely. Records content is configurable but random: the purpose of the tool is to help with
performance and scalability testing of your infrastructure.

## Features

* Production described in 4 "phases" that repeat in circle: `min`, `up`, `max` and `down`
* All phases are configurable in terms of _seconds_ (duration) and _records per second_ (workload)
* `up` and `down` can be one of many transitions, each with a specific "shape" (ex. `linear`, `ease-in`, `spike-out`, ...)
* Records `key` and `payload` are configurable with fixed, from-file and randomly-generated values
* Records headers can be added to each record
* Kafka producer is fully configurable, including selecting a partitioner
* Built on top of the awesome [librdkafka](https://github.com/edenhill/librdkafka)

## Get started

### Cargo install

If you already have the [Rust Toolchain](https://rustup.rs/) installed, just:

```shell
$ cargo install ksunami
```

### Manually

**TODO**

## Examples

**TODO**

## Core concepts

### The 4 phases

Ksunami is designed around the idea that the user has a specific "workload pattern" that they want to reproduce
against their Kafka cluster. It might be steady/stable and never changing, or it can be a regular flux of records,
interleaved with dramatic spikes that happen at regular intervals. Or it can be that you have a new customer that will
bring lots more traffic to your Kafka cluster.

We have elected to describe such a workload in 4 phases, `min, up, max, down`, that repeat indefinitely:

![](https://github.com/kafkesc/ksunami/raw/main/images/workload-phases.png)

Each phase is associated with a `*-sec` duration argument, to choose how long each should be.
Additionally, `min` and `max` are associated with an amount of _records/sec_ (i.e. _workload_), while `up` and `down`
are associated with a _transition_.

### Transitions

When moving between `min` and `max` phases, the phases `up` and `down` are traversed. Those phases are "transitional":
Ksunami allows to describe _"how"_ the transition between the phases happens. **Each transition has a name**,
and corresponds to a [Cubic Bézier curve](https://en.wikipedia.org/wiki/B%C3%A9zier_curve#Cubic_B%C3%A9zier_curves):
leaving to the reader to learn about this class of curves,
the short story is that Cubic Bézier curves describe the _interpolation_ across **4 control points**, `P0, P1, P2, P3`.

Imagine we want to plot an `up` transition between the `min` and `max` phases, on a cartesian plane.
Time is expressed by `x`, and we consider the interval `x = [0..1]` as start and end of the transition.
The volume of records produced is instead expressed by `y`, also considered in the interval `y = [0..1]`.

Given this premise, `P0=(0,0)` and `P3=(1,1)` represent the start and end of the `up` transition;
`P0=(0,1)` and `P3=(1,0)` represent instead the start and end of the `down` transition.

Our transition curve is _encased_ in the bounding box `(0,0), (1,0), (1,1), (0,1)`, and we can describe various kinds of
curves, by placing `P1` and `P2` within this bounding box. The following is the current list of transition name
that Ksunami supports, plotted both for the `up` and `down` phases:

|   Transition   |                                `--up <TRANSITION_TYPE>`                                |                                `--down <TRANSITION_TYPE>`                                | 
|:--------------:|:--------------------------------------------------------------------------------------:|:----------------------------------------------------------------------------------------:|
|     `none`     |                                           -                                            |                                            -                                             |
|    `linear`    |    ![](https://github.com/kafkesc/ksunami/raw/main/images/up-transition_linear.png)    |    ![](https://github.com/kafkesc/ksunami/raw/main/images/down-transition_linear.png)    |
|   `ease-in`    |   ![](https://github.com/kafkesc/ksunami/raw/main/images/up-transition_ease-in.png)    |   ![](https://github.com/kafkesc/ksunami/raw/main/images/down-transition_ease-in.png)    |
|   `ease-out`   |   ![](https://github.com/kafkesc/ksunami/raw/main/images/up-transition_ease-out.png)   |   ![](https://github.com/kafkesc/ksunami/raw/main/images/down-transition_ease-out.png)   |
| `ease-in-out`  | ![](https://github.com/kafkesc/ksunami/raw/main/images/up-transition_ease-in-out.png)  | ![](https://github.com/kafkesc/ksunami/raw/main/images/down-transition_ease-in-out.png)  |
|   `spike-in`   |   ![](https://github.com/kafkesc/ksunami/raw/main/images/up-transition_spike-in.png)   |   ![](https://github.com/kafkesc/ksunami/raw/main/images/down-transition_spike-in.png)   |    
|  `spike-out`   |  ![](https://github.com/kafkesc/ksunami/raw/main/images/up-transition_spike-out.png)   |  ![](https://github.com/kafkesc/ksunami/raw/main/images/down-transition_spike-out.png)   |
| `spike-in-out` | ![](https://github.com/kafkesc/ksunami/raw/main/images/up-transition_spike-in-out.png) | ![](https://github.com/kafkesc/ksunami/raw/main/images/down-transition_spike-in-out.png) |

Please note: in the pictures above `P0` and `P3` don't change, but all variations are generated by moving `P1` and `P2`. 

**Yes!** It's possible to define additional variations, by picking new `P1` and `P2` points,
and adding those to a new value in the `Transition enum`. PRs welcome.

## Usage

To begin, start with `ksunami -h` or `ksunami --help` for the short and long versions of the usage instructions.
_Go ahead, I'll wait!_.

### Configuring the Producer

Additional to the obvious `-b, --brokers` for the bootstrap brokers, and `--client-id` for the client identifier,
it's possible to fine tune the Provider via `--partitioner` and `-c, --config`.

#### Partitioner

Possible values for the `--partitioner` argument are:

|              Partitioner name | Description                                                                                                                           |
|------------------------------:|:--------------------------------------------------------------------------------------------------------------------------------------|
|                      `random` | Random distribution                                                                                                                   |
 |                  `consistent` | CRC32 hash of key (Empty and NULL keys are mapped to single partition)                                                                |
| `consistent_random` (default) | CRC32 hash of key (Empty and NULL keys are randomly partitioned)                                                                      |
|                     `murmur2` | Java Producer compatible Murmur2 hash of key (NULL keys are mapped to single partition)                                               |
|              `murmur2_random` | Java Producer compatible Murmur2 hash of key (NULL keys are randomly partitioned): equivalent to default partitioner in Java Producer |
|                       `fnv1a` | FNV-1a hash of key (NULL keys are mapped to single partition)                                                                         |
|                `fnv1a_random` | FNV-1a hash of key (NULL keys are randomly partitioned)                                                                               |

**NOTE:** Ksunami, being based on [librdkafka](https://github.com/edenhill/librdkafka), offers "only" the partitioners
provided by said library.

For example, to use a _purely random partitioner_:

```shell
$ ksunami ... --partitioner random ...
```

#### Additional configuration

As per `-c,--config`, all the values supported [librdkafka](https://github.com/edenhill/librdkafka/blob/master/CONFIGURATION.md)
are supported by Ksunami's producer.

For example, to set a _200ms producer lingering_ and to _limit the number of producer send retries to 5_:

```shell
$ ksunami ... -c linger.ms:200 ... --config message.send.max.retries:5 ...
```

### Records: destination and content

You can configure the content of each record produced by Ksunami:

* `-t, --topic <TOPIC>`: the destination topic to send the record to
* `-k, --key <KEY_TYPE:INPUT>` (optional): the key of the record
* `-p, --payload <PAYLOAD_TYPE:INPUT>` (optional): the payload of the record
* `--partition <PARTITION>` (optional): the specific partition inside the destination topic
* `--head <HEAD_KEY:HEAD_VAL>` (optional): one (or more) header(s) to decorate the record with

While for `--topic`, `--partition` and `--head` the input is pretty self-explanatory, `--key` and `--payload` support
a richer set of options.

The supported `KEY_TYPE/PAYLOAD_TYPE` are:

|          Format | Description                                                                             |
|----------------:|:----------------------------------------------------------------------------------------|
|    `string:STR` | `STR` is a plain string                                                                 |
|     `file:PATH` | `PATH` is a path to an existing file                                                    |
|  `alpha:LENGTH` | `LENGTH` is the length of a random alphanumeric string                                  |
|  `bytes:LENGTH` | `LENGTH` is the length of a random bytes array                                          |
|   `int:MIN-MAX` | `MIN` and `MAX` are limits of an inclusive range from which an integer number is picked |
| `float:MIN-MAX` | `MIN` and `MAX` are limits of an inclusive range from which a float number is picked    |

This allows to have a degree of flexibility to the content that is placed inside records.

For example, to produce records where the _key is an integer between 1 and 1000_
and the _payload is a random sequence of 100 bytes_:

```shell
$ ksunami ... --key int:1-1000 --payload bytes:100 
```

### Records: amount and duration

As seen above when we introduced the [4 phases](#the-4-phases), Ksunami sees a workload pattern as 
a set of durations, workload volume and transitions.

#### Min and Max

The `min` and `max` phases represent the range of workload that a user wants to describe, and once setup, Ksunami will
cyclically go from `min` to `max` to `min` and so forth. The workload is expressed as _records/sec_, and the duration of
each phase in seconds.

**Why seconds?** Because it's small enough to describe any meaningful Kafka workload. Using a smaller unit would have
yielded no real benefit. And using a larger unit, would lead to too coarse workload description. 

The arguments used to configure `min` and `max`:

|          Argument | Description                                                  | Default |
|------------------:|:-------------------------------------------------------------|:-------:|
| `--min <REC/SEC>` | Minimum amount of records/sec                                |         |
| `--min-sec <SEC>` | How long to produce at minimum records/sec, before ramp-up   |  `60`   |
| `--max <REC/SEC>` | Maximum amount of records/sec                                |         |
| `--max-sec <SEC>` | How long to produce at maximum records/sec, before ramp-down |  `60`   |

#### (Ramping) Up and Down

Again, as [seen above](#the-4-phases), between the `min` and `max` phases there are 2 transitional phases: `up` and `down`.

They exist to describe the mutation of workload, as time progresses.
Ksunami offers a collection of [transitions](#transitions), and they are provided as arguments: Ksunami takes care
of taking the `[0..1]` curves shown above, and transpose them to the actual records/sec workload the user is after.

The arguments used to configure `up` and `down`:

|                   Argument | Description                                              | Default  |
|---------------------------:|:---------------------------------------------------------|:--------:|
|   `--up <TRANSITION_TYPE>` | Ramp-up transition from minimum to maximum records/sec   | `linear` |
|           `--up-sec <SEC>` | How long the ramp-up transition should last              |   `10`   |
| `--down <TRANSITION_TYPE>` | Ramp-down transition from maximum to minimum records/sec |  `none`  |
|         `--down-sec <SEC>` | How long the ramp-down transition should last            |   `10`   |

### Log verbosity

Ksunami follows the long tradition of `-v/-q` to control the verbosity of it's logging:

| Arguments | Log verbosity level | Default |
|----------:|:--------------------|:-------:|
|  `-qq...` | `OFF`               |         |
|      `-q` | `ERROR`             |         |
 |         - | `WARN`              |    x    |
 |      `-v` | `INFO`              |         |
|     `-vv` | `DEBUG`             |         |
| `-vvv...` | `TRACE`             |         |

It uses [log](https://crates.io/crates/log) and [env_logger](https://crates.io/crates/env_logger),
and so logging can be configured and fine-tuned using the Environment Variable `KSUNAMI_LOG`.
Please take a look at [env_logger doc](https://docs.rs/env_logger/latest/env_logger/#enabling-logging) for
more details.

## Contributing

**TODO**

## TODOs

* [ ] Support for jitter in the size of keys/payload
* [ ] Support for jitter in the number of records per phase
* [ ] Support for jitter in the duration of each phase
* [ ] Surface "producer ack" config (?)
* [ ] Surface "compression" config (?)
* [ ] Support for sequential values for keys/payload (seq of ints? seq of strings? closed sequence? random amongst a set?)
* [ ] Support for JSON values, where each fields can be paired with an instance of `ValueGenerator` enum?
* [ ] Publish a binary for Linux/macOS/Windows x AMD64/ARM64
* [ ] Publish a build via Homebrew
* [ ] Publish a Docker image containing Ksunami
* [ ] Support for Tracing
* [ ] Support for OpenTelemetry
* [ ] Write a small but clear contribution guide: something not too stringent, but that establishes a modicum of guidance and "boundaries"

## License

[Apache License 2.0](./LICENSE)

## Thanks

* Thanks to [this page](https://www.desmos.com/calculator/d1ofwre0fr) on [Desmos Graphing Calculator](https://www.desmos.com/calculator),
  for providing an easy way to plot [Cubic Bézier curves](https://en.wikipedia.org/wiki/B%C3%A9zier_curve#Cubic_B%C3%A9zier_curves).
* Thanks to [flo_curves](https://crates.io/crates/flo_curves) for providing an easy Rust crate to manipulate Bézier curves.
* Thanks to [librdkafka](https://github.com/edenhill/librdkafka/blob/master/CONFIGURATION.md) for being an awesome Kafka library,
  used by pretty much all Kafka clients out there, and thanks to the Rust binding [rdkafka](https://crates.io/crates/rdkafka).
* Thanks to [clap](https://crates.io/crates/clap), for being the awesome-est CLI argument parser in existence.
