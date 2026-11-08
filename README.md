# discscape

Renders images as visible patterns on the recording side of optical discs (currently CD and DVD).

discscape supports configurable disc profiles, spiral geometry calibration, and phase correction to compensate for recording geometry errors.

This is an independent implementation, not affiliated with any third-party project.

**discscape is experimental and under development.** ...

## Usage
Build and run from source with Cargo
```bash
cargo run --release -- --help
```

### Render
Render an image using JSON disc profile
```bash
discscape render picture.png image.iso --profile disc_profile.json
```

For CD, specify the output sector format
```bash
discscape render picture.png image.bin --profile disc_profile.json --format audio
```

To write the generated data to standard output, use `-` for output filename:
```bash
discscape render picture.png - --profile disc_profile.json
```

### Disc Profiles
A disc profile is a JSON document describing the recording medium, spiral geometry, optional phase correction, and optional media/recording metadata.

To inspect a profile and view its derived geometry information:
```
discscape profile show disc_profile.json
```

## Burning Discs
### DVD
You can generate a .iso image and then write to a DVD using `growisofs`
```bash
discscape render picture.png image.iso --profile disc_profile.json
growisofs -dvd-compat -Z /dev/sr0=image.iso
```

Or, you can use pipe command to pipe data to `growisofs`
```bash
discscape render picture.png - --profile disc_profile.json | growisofs -dvd-compat -Z /dev/sr0=/dev/stdin
```

## Documentation
- [Calibration Notes](./notes.md) - Experimental calibration procedure and measurement notes.
- [Geometry Fitting Notes](./fitter_notes.md) - Mathematical background for geometry fitting.
- [Disc Profile](./doc/profile.md) - Disc profile format description
- [Measurements](./doc/measurements.md) - Measurements format description