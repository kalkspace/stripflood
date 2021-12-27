# stripflood

Control a WS281x LED strip using HTTP

## Run

```
cargo build
sudo ./target/debug/stripflood
```

Will launch on port 3000.

## Use

```
curl -v 10.1.2.227:3000/batch -d '{ "pixels": [[255,50,0],[255,50,0],[255,50,0]], "offset": 6, "step": 5, "loop": true }' -H "content-type: application/json"
```

JSON payload:
- `pixels`: Array of RGB-tuples to apply
- `offset`: At which offset to start applying colors (optional, default `0`)
- `step`: Distance between colors on the strip (optional, default `1`)
- `loop`: Whether to repeat the colors until the end of the strip (optional, default `false`)
