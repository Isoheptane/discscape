# Phase Error Measurement Format

A measurement file records observed phase errors at selected disc radii. It is used with `discscape profile fit` to refine an existing disc profile.

## Example
```json
{
  "unit": "deg",
  "ref_radius": 55.0,
  "samples": [
    [25.0, -11.1],
    [30.0, -51.8],
    [35.0, -49.6],
    [40.0, -20.7],
    [45.0, 24.0],
    [50.0, 87.9],
    [55.0, 158.2]
  ],
  "base_profile": "dvd_profile.json",
  "notes": "Example phase measurements"
}
```

## Fields

| Field | Required | Description |
| ----- | -------- | ----------- |
| `unit` | Yes | Error units: `deg` (degrees), `rev` (revolutions), `arc_mm` (tangential arc length in mm) |
| `samples` | Yes | Pairs of `[radius_mm, error]`; the second value uses unit.
| `ref_radius` | No | Phase reference radius, in mm.
| `base_profile` | No | Name or path of the profile used for measurement; informational only. |
| `notes` | No | Free-form measurement notes. |

When `ref_radius` is omitted, fitting uses the profile's starting radius (`geometry.r0_mm`). The CLI option `--ref-radius` overrides the value in this file.