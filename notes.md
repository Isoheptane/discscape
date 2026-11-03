## CD Family
### CD-DA Audio Track
Driver won't swap byte pair before forming a F1 frame, use BE (no swap) in code.

Example CUE file:

```cue
FILE "disc-art.pcm" BINARY
  TRACK 01 AUDIO
    INDEX 01 00:00:00
```

### CD-ROM (MODE1/2048, MODE1/2352, MODE2/2336)
Driver will swap byte pair before forming a F1 frame, use LE (swap) in code, so two swap cancels each other.

If byte order is wrong, double image may appear in 

Example CUE file:

```cue
FILE "payload.bin" BINARY
  TRACK 01 MODE1/2048 
    INDEX 01 00:00:00
```

Replace `MODE1/2048` with desired mode.

## Calibration process
Sign convention: When viewed from data side, clockwise is positive. The calibration process should use a spoke image. 

### Idea
Imagine we are painting a string, and the goal is to let it form a good looking spoke image when it winds back to a spiral. The spiral is described by three geometric parameters: start radius ($r_0$), pitch ($p$) and revolutions count ($N$).

Initially, we start with a guessed or good enough geometry $G_{\text{cur}}=\set{r_{0\text{cur}}, p_\text{cur}, N_\text{cur}}$ that is burned into a CD. Then measure the spoke offset relative to the innermost visible part of the calibration image. Suppose at radius $r$, spoke is curved with positive phase error $\Delta\theta$, this means that:
- Our current setup does not properly reflect true geometry of the disc
- At radius $r$, we paint the spoke late for $\Delta\theta$, and we should paint it earlier
- It means that our image sampling should look ahead for $\Delta\theta$ at radius $r$.
- Go find the corresponding normalized position at $r$. If at that position old geometry gives unwrapped angle $\theta_{\text{cur}}$, then our new geometry should give $\theta_{\text{new}}$ that ideally makes $\theta_{\text{new}} - \theta_{\text{cur}} = \Delta\theta$, or minimizes the phase residuals across all measure measurement points $\theta_{\text{new}} - \theta_{\text{cur}} - \Delta\theta$.

### Initial Geometry
To calibrate for a specific disc, you need to guess a geometry first. You can use the geometry of a existing disc, or use conventional values.

We need to fix $r_0$ for initial calibration process. The true value can vary, but fixing a initial $r_0$ makes later jobs easier. Here are some valuses you can reference:

- CD (80 minute)
  - $r_0=25.00\ \text{mm}$ (True value between 24.80 ~ 25.00 mm)
  - $p=1.51\ \mu\text{m}$ (True value between 1.5 ~ 1.7 µm)
  - $N=22110.0$
- DVD (4.7GB)
  - $r_0=24.00\ \text{mm}$ (True value between 23.80 ~ 24.00 mm)
  - $p=0.74\ \mu\text{m}$
  - $N=46000.0$

You may need to search in a area and find geometry that the spoke is clearly visible. If you find a geometry than produces visible spokes (even if heavily distorted), continue to next step.

### Coarse Revolution Calibration
Burn a disc and inspect how much turns have the spokes rotate (estimate fractional turns if it's not heavily distorted)

If spokes turns $m$ revolutions in clockwise direction from from inner to outer. It means that our $N$ is too small for real disc, increase $N$ by $m$. If spokes turns in counter-clockwise direction, decrease $N$ by $m$.

Repeat this process until the accumulated spoke rotation error is near zero revolutions.

### Radius Calibration (Optional)
Remember our initial $r_0$ is chosen as nominal value?

Once you have calibrated the revolution count $N$. You can optionally calibrate for real radius. Here, you can calibrate by limiting program-area end radius or by measurement.

You can also skip this step. The image burned on the disc might be slightly larger or smaller if this step is skipped, but skipping this step does not result in distortion.

#### Limit Program Area end radius
Calculate the end radius of the geometry: $r_\text{end}=r_0+pN$. If $r_\text{end}$ exceeds 58 mm, we can scale down the whole image.

The scaling factor $s=\cfrac{58\ \text{mm}}{r_\text{end}}$. Use new geometry:
- $r_0'=sr_0$
- $p'=sp$
- $N'=N$

#### Measure and calibrate
Spoke image should have rings for specified radius. For a example, we choose the ring at 55 mm radius. Manually measure the real radius $r_\text{real}$ at 55 mm radius ring.

The scaling factor $s=\cfrac{r_\text{real}}{55\ \text{mm}}$. Use new geometry:
- $r_0'=sr_0$
- $p'=sp$
- $N'=N$

### Pitch Calibration
Once revolution count $N$ is calibrated, the remaining distortion mainly comes from incorrect $p$. Which can produce bow shaped distortion as much as a few revolutions.

[ Here we should use a tool for calibration, which gives calibrated p and N, but it is still under construction. Use the method in Idea section. ]

Measure the phase error at different radius, use tool to calculate corrected $p$ and $N$. Burn discs with new geometry. Repeat this process until the spokes follows radial direction. You may still see S shaped distortions, they are residual errors that potentially from
- Local variation in effective spiral pitch
- Drive-dependent behavior

Now, geometry is successfully calibrated.

For residual errors, calibrate them with additional phase correction.

## DVD Family

### Data Format
2048 bytes of main data are transformed successively to
- Data Frame
  - Add Sector ID, IED and RSV
- Scrambled Frame
  - Add XOR Scramble
- ECC Block
  - 16 Srcambled Frame forms as ECC Block
- 16 Recording Frames
  - A ECC Block is split / interleaved into
- EFMPlus
  - Data goes into 16-bit Code Words and to physical sectors

### Data Frames
The structure is added to
- Sector ID (4 Bytes)
- IED (2 Bytes)
- Reserved (6 Bytes)
- Main Data (2048 Bytes)
- EDC (4 Bytes)

To form a 12 rows * 172B data frame.

#### Sector ID
Sector ID have 32 bits
```
(msb) b31        b24   b23                   b0 (lsb)
| Sector Information | <- Physical Sector Number -> |
```
The sector information bits are
| bit | Meaning | Indicates |
| --- | :- | :- |
| b31 | Sector format type | 1: Zoned format ; 0: CLD format |
| b30 | Tracking method | 1: Groove tracking; 0: Pit tracking |
| b29 | Reflectivity | 1: Reflectivity <= 40%; 0: Reflectivity > 40% | 
| b28 | Reserved | Shall be set to 0 |
| b27 / b26 | Zone Type | 00: Data Zone; 01: Lead-in Zone; 10: Lead-out zone |
| b25 | Data Type | 1: Linking Data; 0: Rewritable / Recordable data;
| b24 | Layer number | 0: Layer 0; 1: Layer 1|

For fast checking, use these values:
- DVD-R: `0x00` (ECMA-279)
- DVD-RW: `0x20` (ECMA-338)
- DVD+R: `0x00` (ECMA-349)
- DVD+RW: `0x62` (ECMA-337)

#### ID Error Dection Code (IED)
*To be completed, I will write this later*

#### Reserved
Application dependent, can be used for copyright management information (CPR_MAI).
Default to `0x00` for all 6 bytes.

#### Error Detection Code (EDC)
This is on the tail of a data frame. It is also near the end of a 91 bytes sync frame, right before EFMPlus state machine reset. No need for predicting this.

### Scrambling
Scrambling uses a LFSR for generating scrambling sequence:

```
  +-------------[XOR]-------------------------------------+
  |               |                                       V
|r14|r13|r12|r11|r10|r9 |r8 |r7 |r6 |r5 |r4 |r3 |r2 |r1 |r0 |
```

Every shift takes r14 and r10 and insert it to r0 after shifting high.

The LFSR initial values are determined by b7-b4 (which is `(PSN & 0xF0 >> 4)`)

| Preset number | Initial value | Preset number| Initial value |
| :-: | :-:| :-: | :-: |
| `0` | `0001` | `8` | `0010` |
| `1` | `5500` | `9` | `5000` |
| `2` | `0002` | `A` | `0020` |
| `3` | `2A00` | `B` | `2001` |
| `4` | `0004` | `C` | `0040` |
| `5` | `5400` | `D` | `4002` |
| `6` | `0008` | `E` | `0080` |
| `7` | `2800` | `F` | `0005` |

For getting next scrambling byte, take r7-r0 as the scrambling byte. The part of the initial value of r7-r0 is taken out as scrambling byte S_0.

Scrambled Frame is generated by XOR every main data byte in Data Frame with generated scrambling sequence.

### ECC Block
(In this section, data frame means a scrambled data frame)

```
---  +--------------------------+--------------+
 |   | Data Frame 0  (12R*172B) | PI (12R*10B) |
 |   +--------------------------+--------------+
192  | ....                     | PI (12R*10B) |
 |   +--------------------------+--------------+
 |   | Data Frame 15 (12R*172B) | PI (12R*10B) |
---  +--------------------------+--------------+
     | PO (16R*172B)            | PI (16R*10B) |
     +--------------------------+--------------+
```

ECC block forms a 208 rows * 182 bytes ECC block.

PO and PI calculation are omitted.

### Recording Frames
Basically, we take a data frame's rows and append one PO row in ECC Block:

```
+--------------------------+--------------+
| Data Frame N  (12R*172B) | PI (12R*10B) |
+--------------------------+--------------+
| PO Row N (1R*172B)       | PI (1R*10B)  |
+--------------------------+--------------+
```

A Recording frame is a 13 row * 182 bytes block.

### EFM Plus Modulation

#### Sync Frame
EFM Plus state machine resets to 1 on every 91 bytes after each SYNC channel bits, thus a recording rrame can be split into 26 sync frames.

```
+------+---------------------+------+---------------------+
| SYNC | Row  1 Part 1 (91B) | SYNC | Row  1 Part 2 (91B) |
+------+---------------------+------+---------------------+
| SYNC | ...                 | SYNC | ...                 |
+------+---------------------+------+---------------------+
| SYNC | Row 12 Part 1 (91B) | SYNC | Row 12 Part 2 (91B) |
+------+---------------------+------+---------------------+
| SYNC | Row 13 Part 1 (91B) | SYNC | Row 13 Part 2 (91B) |
+------+---------------------+------+---------------------+

```

Where SYNC is 32 channel-bits long, and parts are 91*16=1456 channel-bits long.