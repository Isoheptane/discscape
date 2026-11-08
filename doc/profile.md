# Disc Profile Format

A disc profile is a JSON document describing the recording medium, spiral geometry and optional phase correction. Pass is to `discscape render` with `--profile`

## Example
```json
{
  "disc": {
    "type": "DVD+R",
    "capacity_sectors": 2295104
  },
  "geometry": {
    "r0_mm": 24.0,
    "pitch_um": 0.74,
    "n": 46000.0
  },
  "correction": {
    "type": "interpolation",
    "method": "linear",
    "extrapolation": "clamp",
    "samples": [[25.0, 0.0], [40.0, -0.02], [55.0, 0.0]]
  },
  "metadata": {
    "media": { "brand": "Example brand" },
    "recording": { "drive": "Example drive", "speed_x": 4 }
  }
}
```
*Illustrative values only; do not use this geometry as calibrated preset*

## Fields

| Field | Required | Description |
| ----- | -------- | ----------- |
| `disc.type` | Yes | `CD-R`, `CD-RW`, `DVD-R`, `DVD+R`, `DVD-RW`, or `DVD+RW` |
| `disc.capacity_sectors` | Yes | Addressable logical sector count. |
| `geometry.r0_mm` | Yes | Spiral starting radius, in mm. |
| `geometry.pitch_um` | Yes | Spiral track pitch, in µm per revolution. |
| `geometry.n` | Yes | Number of spiral revolutions; may be fractional.
| `correction` | No | Radius-dependent angular correction; omit or use `null` for none |
| `metadata` | No | Descriptive media and recording details, not render options. |

## Correction
### Interpolation Correction
```json
{
    "type": "interpolation",
    "method": "linear",
    "extrapolation": "clamp",
    "samples": [[25.0, 0.0], [40.0, -0.02], [55.0, 0.0]]
},
```

| Field | Description |
| ----- | ----------- |
| `type` | `interpolation` |
| `method` | Interpolation method; `linear` |
| `extrapolation` | Extrapolation method; `clamp`, `linear` |
| `samples` | Pairs of `[radius_mm, correction_rev]`; 1 rev = 360°

## Optional Metadata
- `metadata.media`
  - `brand`
  - `description`
  - `rated_speed`
  - `mid`
- `metadata.recording`
  - `drive`
  - `firmware`
  - `speed_x`
  - `software`
  - `date`
- `metadata.notes`