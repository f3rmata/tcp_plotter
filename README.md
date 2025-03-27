# Simple Tcp Plotter

A simple tool to plot your tcp transmitted cordinates.

## Usage

Using cargo to build this application.

```
cargo build --release
```

Transmit your cordinates data in one line json. End with `\n`  
note: x -> f64, y -> f64

---

Example data:

```json
[{"x": 0.5, "y": 0.5}, {"x": 0.6, "y": 0.2}]
```
