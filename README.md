## discscape
Renders images as visible patterns on the data side of optical discs (currently CD and DVD).

This is an independent implementation, not affiliated with any third-party project.

**discscape is experimental and under development.** The command-line interface is available in the source code, but no prebuilt executables are provided. Some features are incomplete, and generated images should be treated as experimental.

## Usage
Build and run from source with Cargo
```bash
cargo run --release -- --help
```

### Render
Render an image using JSON disc profile
```bash
discscape render picture.png \
    --profile disc.json \
    --output picture.iso 
```

For CD, specify the output sector format
```bash
discscape render picture.png \
    --profile disc.json \
    --output picture.iso \
    --format audio
```

To write the generated data to standard output, use `-` for output filename:
```bash
discscape render picture.png \
    --profile disc.json \
    -o -
```

### Disc Profiles
A disc profile is a JSON document describing the recoding medium, spiral geometry, optional phase correction, and optional media/recording metadata.

To inspect a profile and view its derived geometry information:
```
discscape profile show disc.json
```

## Documentation
- [Calibration Notes](./notes.md) - Experimental calibration procedure and measurement notes.
- [Geometry Fitting Notes](./fitter_notes.md) - Mathematical background for geometry fitting.
- [Disc Profile](./disc_profile.md) - Disc profile format description