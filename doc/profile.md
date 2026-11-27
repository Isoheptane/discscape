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
        "recording": { "drive": "Example drive", "speed_x": 4 },
        "notes": ["This is a note entry"],
        "contributors": ["Contributor Name"]
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
}
```

| Field | Description |
| ----- | ----------- |
| `type` | `interpolation` |
| `method` | Interpolation method; `linear`, `cubic_bessel` |
| `extrapolation` | Extrapolation method; `zero`, `clamp`, `linear` |
| `samples` | Pairs of `[radius_mm, correction_rev]`; 1 rev = 360° |

## Optional Metadata
All fields in metadata are completely optional. But they can help user identify if their disc matches the profile.

### Media
```json
{
    "brand": "Verbatim",
    "description": "Verbatim DVD+R AZO 16x #43498 #95037",
    "mid": "MCC/004",
    "stamper_code": "ZE6369-DVR-T47D",
    "rated_speed": "16x"
}
```

| Field | Description |
| ----- | ----------- |
| `brand` | Brand name |
| `description` | Free form disc description |
| `mid` | Media Identification Code of disc. <br/> Use Start Time of Lead-In (MSF) in ATIP for CD <br/> Use MID for DVD  |
| `stamper_code` | Stamper code on the inner ring of the disc |
| `rated_speed` | Rated speed for specific media |

#### Media Identification Code
For CDs, use Start Time of Lead-In (MSF) in ATIP. To check the Start Time of Lead-In, use command `cdrdao disk-info` or `cdrecord -atip`. Write it in `MMmSSsFFf` form eg. `97m34s23f`.

For DVDs, use Media Identification Code. To check it, use command `dvd+rw-mediainfo`.

### Recording
Recording segment shows the drive and software used in calibration process.

```json
{
    "drive": "HP DVD A DS8ACSH",
    "firmware": "LHS3",
    "speed_x": 3,
    "software": "growisofs",
    "date": "2026-11-03"
},
```

| Field | Description |
| ----- | ----------- |
| `drive` | Optical drive name |
| `firmware` | Firmware name / version |
| `speed_x` | Recording speed |
| `software` | Recording software |
| `date` | Date of calibration |

### `metadata.notes`
This can be a string array or a string. Used to store extra notes of this profile.

### `metadata.contributors`
This can be a string array or a string. Used to store the contributors of this profile.