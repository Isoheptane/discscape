# Disc Profile Format
`discscape` uses JSON disc profiles to describe the recording medium, spiral geometry, and optional phase correction. A profile is supplied to the renderer with `--profile`.

A disc profile describes the disc and its calibration; it does not specify the input image, output path, sector output format, or other render-command options.

### Examp Profile

## Disc Profile Template
The following example is a template based on one CD-RW configuration. **Do not assume its geometry or correction values are accurate for another disc or recording setup.** Replace them with values appropriate to your media and calibration.

```json
{
    "disc": {
        "type": "CD-RW",
        "capacity_sectors": 359849
    },
    "geometry": {
        "r0_mm": 25.0,
        "pitch_um": 1.500185,
        "n": 22110.0
    },
    "correction": {
        "type": "interpolation",
        "method": "linear",
        "extrapolation": "clamp",
        "samples": [
            [25.0,  0.000000],
            [30.0,  0.002319],
            [35.0,  0.001036],
            [40.0,  0.000211],
            [45.0, -0.001328],
            [50.0, -0.001119],
            [55.0, -0.000572],
            [58.0,  0.000000]
        ]
    },
    "metadata": {
        "media": {
            "brand": "Mitsubishi",
            "description": "Mitsubishi SERL CD-RW 4-12x",
            "rated_speed": "4-12x"
        },
        "recording": {
            "drive": "HP DVD A DS8ACSH",
            "firmware": "LHS3",
            "speed_x": 10,
            "software": "cdrdao",
            "date": "2026-11-03"
        }
    }
}
```

### `disc`: Recording Medium
| Field | Type | Description |
| --- | --- | --- |
| `type` | string | Specific medium type, such as `CD-R`, `CD-RW`, `DVD-R`, `DVD+R`, `DVD-RW`, or `DVD+RW`. |
| `capacity_sectors` | integer | Number of addressable logical sectors in the target disc model. |

`capacity_sectors`is a count of sectors, not a size in bytes.

### `geometry`: spiral-track geometry

| Field | Type | Unit | Description |
| --- | --- | --- | --- |
| `r0_mm` | number | mm | Radius at the start of the modeled recording spiral. |
| `pitch_um` | number | µm/revolution | Radial spacing between successive spiral turns. |
| `n` | number | revolutions | Number of spiral turns in the modeled region; it may be fractional. |

These values describe the geometric model used for rendering for a specific disc. They should not be treated as universal defaults for all discs of the same type.

### `correction`: optional phase correction
Phase correction adjusts the angular alignment of the rendered pattern as a function of radius. Omit `correction` or set it to `null` to use the uncorrected geometry.

The current example uses interpolation:

| Field | Type | Description |
| --- | --- | --- |
| `type` | string | Correction representation; currently illustrated by `interpolation`. |
| `method` | string | Interpolation method; currently `linear`. |
| `extrapolation` | string | Behavior outside the sample radius range; `clamp` uses the nearest endpoint value. |
| `samples` | array | Pairs of `[radius_mm, correction_rev]`. |

Each sample contains a radius in **millimeters** and a phase correction in **revolutions** (`1 rev = 360°`). Positive correction values represents clockwise correction.

Each sample contains a radius in **millimeters** and a phase correction in **revolutions** (`1 rev = 360°`). Positive correction values represents clockwise correction.

Samples should be sorted by strictly increasing radius, without duplicate radii, and should cover the radii relevant to the intended render. With linear interpolation, values between sample points are interpolated along straight-line segments. With `clamp`, radii outside the sample range use the first or last correction value rather than extending the slope.

Other correction representations, such as spline, polynomial, and lookup tables, may be introduced in future versions. They are **not part of this example's currently documented format**.

## `metadata`: recording and media notes

`metadata` records the origin and conditions of a profile. It does **not** select a recording drive or perform a disc burn, and its descriptive fields should not affect the rendered sector data.

### `metadata.media`

| Field | Type | Description |
| --- | --- | --- |
| `brand` | string | Media brand or manufacturer label. |
| `description` | string | Human-readable description of the disc. |
| `rated_speed` | string | Advertised recording speed range, e.g. `4-12x`. |
| `mid` | string | Optional media identification code, if known. |

### `metadata.recording`

| Field | Type | Description |
| --- | --- | --- |
| `drive` | string | Recorder model used for the referenced recording or calibration. |
| `firmware` | string | Recorder firmware revision. |
| `speed_x` | number | Recording speed multiplier. |
| `software` | string | Software used to record the disc. |
| `date` | string | Date of the referenced recording, conventionally `YYYY-MM-DD`. |