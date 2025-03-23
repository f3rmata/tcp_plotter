# Simple Tcp Plotter

A simple tool to plot your tcp transmitted cordinates.

## Usage

Using cargo to build this application.

```
cargo build --release
```

Transmit your cordinates data in one line json. End with `\n`
note: x -> i32, y -> f64

---

Example data:

```json
[{"x": 1, "y": 1.0}, {"x": 2, "y": 2.0}]
```
