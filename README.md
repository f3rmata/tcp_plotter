# Simple Tcp Plotter

A simple tool to plot your tcp transmitted cordinates.

## Usage

using cargo to build this application.

```
cargo build --release
```

transmit your cordinates data in one line json. End with `\n`
note: x -> i32, y -> f64

example data:

```json
[{"x": 1, "y": 1.0}, {"x": 2, "y": 2.0}]
```
