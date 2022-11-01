# :ocean: Kafkesc - Ksunami :ocean:

_**Waves of Kafka Records!**_

[![Build + Test](https://github.com/kafkesc/ksunami/actions/workflows/build+test.yml/badge.svg)](https://github.com/kafkesc/ksunami/actions/workflows/build+test.yml)

## :grey_question: Why?

Ksunami is a command line tool to produce volumes of (dummy) records against a [Kafka](https://kafka.apache.org/) cluster.

If you are experimenting with scalability and latency against Kafka, and are looking for ways to reproduce a continues
flow of records, following a very specific traffic pattern that repeats periodically, **Ksunami** is the tool for you.

Ksunami offers a way to set up the production of records, expressing the scenario as a sequence of "phases"
that repeat indefinitely.

## :bulb: Features

* Production described in 4 "phases" that repeat in circle: `min`, `up`, `max` and `down`
* All phases are configurable in terms of _seconds_ (duration) and _records per second_ (workload)
* `up` and `down` can be one of many transitions, each with a specific "shape" (ex. `linear`, `ease-in`, `spike-out`, ...)
* Records `key` and `payload` are configurable with fixed, from-file and randomly-generated values
* Records headers can be added to each record
* Kafka producer is fully configurable, including selecting a partitioner
* Built on top of the awesome [librdkafka](https://github.com/edenhill/librdkafka) :heart:,
  thanks to the Rust binding [rdkafka](https://crates.io/crates/rdkafka) :heart_eyes:

## :horse: Get started

### Cargo install

TODO

### Manually

TODO

## :dragon: Core concepts

### :traffic_light: The 4 phases

TODO

### :roller_coaster: Transitions

When moving between `min` and `max` phases, the phases `up` and `down` are traversed. Those phases are "transitional":
Ksunami allows to describe _"how"_ the transition between the phases happens. **Each transition has a name**,
and corresponds to a Cubic Bézier[^bezier] curve: leaving to the reader to learn about this class of curves,
the short story is that Cubic Bézier curves describe the _interpolation_ across **4 control points**, `P0, P1, P2, P3`.

Imagine we want to plot an `up` transition between the `min` and `max` phases, on a cartesian plane.
Time is expressed by `x`, and we consider the interval `x = [0..1]` as start and end of the transition.
The volume of records produced is instead expressed by `y`, also considered in the interval `y = [0..1]`.

Given this premise, `P0=(0,0)` and `P3=(1,1)` represent the start and end of the `up` transition;
`P0=(0,1)` and `P3=(1,0)` represent instead the `down` transition.

Our transition curve is encased in the bounding box `(0,0), (1,0), (1,1), (0,1)`, and we can describe various kinds of
transition, by placing `P1` and `P2` within this bounding box. The following is the current list of transition name
that Ksunami supports, plotted both for the `up` and `down` phases:

| Transition[^desmos] |           `--up <TRANSITION_TYPE>`           |           `--down <TRANSITION_TYPE>`           | 
|:-------------------:|:--------------------------------------------:|:----------------------------------------------:|
|       `none`        |                      -                       |                       -                        |
|      `linear`       |    ![](./images/up-transition_linear.png)    |    ![](./images/down-transition_linear.png)    |
|      `ease-in`      |   ![](./images/up-transition_ease-in.png)    |   ![](./images/down-transition_ease-in.png)    |
|     `ease-out`      |   ![](./images/up-transition_ease-out.png)   |   ![](./images/down-transition_ease-out.png)   |
|    `ease-in-out`    | ![](./images/up-transition_ease-in-out.png)  | ![](./images/down-transition_ease-in-out.png)  |
|     `spike-in`      |   ![](./images/up-transition_spike-in.png)   |   ![](./images/down-transition_spike-in.png)   |    
|     `spike-out`     |  ![](./images/up-transition_spike-out.png)   |  ![](./images/down-transition_spike-out.png)   |
|   `spike-in-out`    | ![](./images/up-transition_spike-in-out.png) | ![](./images/down-transition_spike-in-out.png) |

Yes! It's possible to define additional transition types, by adding values to the `Transition enum`.

## :racehorse: Usage

To begin, start with `ksunami -h` or `ksunami --help` for the short and long versions of the usage instructions.
_Go ahead, I'll wait!_.

### :wrench: Configuring the Producer

Additional to the obvious `-b, --brokers` for the bootstrap brokers, and `--client-id` for the client identifier,
it's possible to fine tune the Provider via `--partitioner` and `-c, --config`.

Possible values for the `--partitioner` argument are:

|  Partitioner name[^supp_part] | Description                                                                                                                           |
|------------------------------:|:--------------------------------------------------------------------------------------------------------------------------------------|
|                      `random` | Random distribution                                                                                                                   |
 |                  `consistent` | CRC32 hash of key (Empty and NULL keys are mapped to single partition)                                                                |
| `consistent_random` (default) | CRC32 hash of key (Empty and NULL keys are randomly partitioned)                                                                      |
|                     `murmur2` | Java Producer compatible Murmur2 hash of key (NULL keys are mapped to single partition)                                               |
|              `murmur2_random` | Java Producer compatible Murmur2 hash of key (NULL keys are randomly partitioned): equivalent to default partitioner in Java Producer |
|                       `fnv1a` | FNV-1a hash of key (NULL keys are mapped to single partition)                                                                         |
|                `fnv1a_random` | FNV-1a hash of key (NULL keys are randomly partitioned)                                                                               |

For example, to use a _purely random partitioner_:

```shell
$ ksunami ... --partitioner random ...
```

As per `-c,--config`, all the values supported by producers of the
[librdkafka](https://github.com/edenhill/librdkafka/blob/master/CONFIGURATION.md)
library are supported, as Ksunami is based on it.

For example, to set a _200ms producer lingering_ and to _limit the number of producer send retries to 5_:

```shell
$ ksunami ... -c linger.ms:200 ... --config message.send.max.retries:5 ...
```

### :factory: What goes into each Record

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

### :chart_with_upwards_trend: How many Records

TODO

### :microphone: Log verbosity

Ksunami follows the long tradition of `-v/-q` to control the verbosity of it's logging:

| Arguments | Log verbosity level |
|----------:|:--------------------|
|  `-qq...` | `OFF`               |
|      `-q` | `ERROR`             | 
 |         - | `WARN`              |
 |      `-v` | `INFO`              |
|     `-vv` | `DEBUG`             |
| `-vvv...` | `TRACE`             |

It uses [log](https://crates.io/crates/log) and [env_logger](https://crates.io/crates/env_logger),
and so logging can be configured and fine-tuned using the Environment Variable `KSUNAMI_LOG`.
Please take a look at [env_logger doc](https://docs.rs/env_logger/latest/env_logger/#enabling-logging) for
more details.

## :rainbow: Examples

TODO

## :pushpin: TODOs

* [ ] Support for jitter in the size of keys/payload
* [ ] Support for jitter in the number of records per phase
* [ ] Support for jitter in the duration of each phase
* [ ] Surface "producer ack" config (?)
* [ ] Surface "compression" config (?)
* [ ] Support for sequential values for keys/payload (seq of ints? seq of strings? closed sequence? random amongst a
  set?)
* [ ] Publish a binary for Linux/macOS/Windows x AMD64/ARM64
* [ ] Publish a build via Homebrew
* [ ] Support for Tracing
* [ ] Support for OpenTelemetry

## :four_leaf_clover: License

[Apache License 2.0](./LICENSE)

## Notes

[^desmos]: Thanks to [this page](https://www.desmos.com/calculator/d1ofwre0fr)
([Desmos Graphing Calculator](https://www.desmos.com/calculator)) to provide an easy way to plot
Cubic Bézier curves.
[^bezier]: Cubic Bézier curves definition ([Wikipedia](https://en.wikipedia.org/wiki/B%C3%A9zier_curve#Cubic_B%C3%A9zier_curves)).
[^supp_part]: Ksunami, being based on [librdkafka](https://github.com/edenhill/librdkafka/blob/master/CONFIGURATION.md),
only supports the partitioners offered by that library
