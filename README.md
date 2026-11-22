# discscape

Renders images as visible patterns on the recording side of optical discs (currently CD and DVD).

`discscape` supports configurable disc profiles, spiral geometry calibration, phase correction to compensate for recording geometry errors, and multi-channel optical pattern multiplexing.

This is an independent implementation, not affiliated with any third-party project.

**discscape is experimental and under development.**

---

## Building & Installation

Build from source with Cargo:

```bash
cargo build --release
```

Or install `discscape` to your Cargo bianry path (`~/.cargo/bin`)

```bash
cargo install --path .
```

If you prefer not to install it globally, replace `discscape` in the examples below with `cargo run --release --`.

## Usage

### 1. Render a Single Image (`render`)
Render an image into disc data using a JSON disc profile:

```
discscape render picture.png image.iso -p disc_profile.json
```

For CD, specify the output sector format (`audio`, `mode1-2352`, `mode1-2048`, `mode2-2336`):

```
discscape render picture.png image.iso -p disc_profile.json --format audio
```

Write generated data to standard output using `-` as `<OUTPUT>`:
```
discscape render picture.png - -p disc_profile.json
```

#### Customizing Optical Patterns & Rendering Range

You can customize how luminance maps to optical modulation patterns (`shortest`, `short`, `medium`, `long`, `random`) and adjust gamma or sector ranges:
``` bash
# Invert light/dark patterns and adjust gamma
discscape render picture.png image.iso -p disc_profile.json \
  --light long --dark shortest --gamma 2.2

# Render only a specific LBA range (inclusive start, exclusive end)
discscape render picture.png image.iso -p disc_profile.json \
  --start-sector 100000 --end-sector 500000
```

Common options for `render`
- `-p, --profile <FILE>`: Path to the disc profile JSON
- `--format <FORMAT>`: Sector format for CD (`audio`, `mode1-2352`, `mode1-2048`, `mode2-2336`)
- `--light <PATTERN>` / `--dark <PATTERN>`: Modulation patterns for bright/dark pixels (defaults: `shortest` / `long`)
- `--gamma <GAMMA>`: Input luminance gamma correction (default: 2.2)
- `--random <SPEC>`: PRNG specs
  - `scramble` for using CD/DVD scramble table for random data, 
  - `splitmix64[:seed]` for using splitmix64 hash
- `--dither <SPEC>`: Dither specs
  - `splitmix64[:seed]` for using splitmix64 hash
- `--start-sector <N>` / `--end-sector <N>`: Partial disc LBA range.

### 2. Multiplex Multiple Images (`multiplex`)
Create hologram-like effects by embedding multiple input images onto a single disc, assigning each image to a distinct optical pattern via `-c, --channel <path>:<pattern>[:<weight>]`

``` bash
discscape multiplex output.iso -p disc_profile.json \
  -c layer_a.png:shortest:1.0 \
  -c layer_b.png:medium:1.0 \
  --dump long
```

- `-c, --channel <SPEC>`: Channel specification in `<path>:<pattern>[:<weight>]` format (can be specified multiple times).
- `--dump <PATTERN>`: Optical pattern used for unassigned energy (default: `long`)
- Supports `-p/--profile`, `--format`, `--start-sector`, `--end-sector`, `--gamma`, `--random`, `--dither-seed <SEED>`, and `-` for stdout.

### 3. Disc Profiles (profile)
A disc profile is a JSON document describing the recording medium, spiral geometry, optional phase correction, and optional media/recording metadata.

Subcommands under discscape profile:
- **`show`**: Inspect profile content and geometry information:
  ```bash
  discscape profile show disc_profile.json
  ```
- **`fit`**: Fit profile geometry (n and pitch) to spoke error measurements, with optional residual phase correction baking:
  ```bash
  discscape profile fit step0.json step1.json -m measurements.json
  # Bake residual error into the correction table (use --replace-correction or --accumulate-correction when updating existing corrections)
  discscape profile fit step1.json step2.json -m measurements.json --bake
  ```
- **`rebase`**: Absorb the residual correction table into `pitch` and `n` so that the residual error becomes zero at `--ref-radius` (defaults to `r0`) and at the outer radius:
  ```bash
  discscape profile rebase input.json output.json --ref-radius 25.0
  ```
  Use `-u, --unconstrained` to disable the strict zero-error constraint at the outer radius, or `--drop-correction` to omit the correction table from the output profile.
- **`scale`**: Scale profile geometry size by `<FACTOR>`:
  ```bash
  discscape profile scale input.json output.json 1.001
  ```
Note: `fit`, `rebase` and `scale`all accept `-` as `<OUTPUT>` to print the resulting JSON to stdout.

## Burning Discs
### DVD
Generate a `.iso` image and write it using `growisofs`
```bash
discscape render picture.png image.iso -p disc_profile.json
growisofs -dvd-compat -Z /dev/sr0=image.iso
```

Or pipe the output directly to `growisofs`:
```bash
discscape render picture.png - -p disc_profile.json | growisofs -dvd-compat -Z /dev/sr0=/dev/stdin
```

### CD
Generate a `.bin` image with the desired `--format` (for example, `audio`):
```bash
discscape render picture.png output.bin -p disc_profile.json --format audio
```

Create a matching `.cue` file (e.g., `toc.cue`):
```cue
FILE "output.bin" BINARY
  TRACK 01 AUDIO
    INDEX 01 00:00:00
```

Replace `AUDIO` with the track type matching your `--format` option:
- `--format audio` - `AUDIO`
- `--format mode1-2352` - `MODE1/2352`
- `--format mode1-2048` - `MODE1/2048`
- `--format mode2-2336` - `MODE2/2336`

Then burn the disc using `cdrdao` (adjust `--device` and `--speed` for your drive):
```bash
cdrdao write --device /dev/sr0 --speed 10 --eject -v 2 test.cue
```


## Documentation
- [Calibration Process](./doc/calibration-process.md) - Calibration Process and Guide
- [Optical Principle (zh-Hant)](./doc/optical-principle.zh-Hans.md) - Optical Principles
- [Disc Profile](./doc/profile.md) - Disc profile format description
- [Measurements](./doc/measurements.md) - Measurements format description