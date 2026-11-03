## Calibration process
Sign convention: When viewed from data side, clockwise is positive.

The calibration process should use a spoke image. 

### Idea
Imagine we are painting a string, and the goal is to let it form a good looking spoke image when it winds back to a spiral. The spiral is described by three geometric parameters: start radius ($r_0$), pitch ($p$) and revolutions count ($N$).

Initially, we start with a guessed or good enough geometry $G_{\text{cur}}=\{r_{0\text{cur}}, p_\text{cur}, N_\text{cur}\}$ that is burned onto a disc. Then measure the spoke offset relative to the innermost visible part of the calibration image. Suppose at radius $r$, spoke is curved with positive phase error $\Delta\theta$, this means that:
- Our current setup does not properly reflect true geometry of the disc
- At radius $r$, we paint the spoke late for $\Delta\theta$, and we should paint it earlier
- It means that our image sampling should look ahead for $\Delta\theta$ at radius $r$.
- Go find the corresponding normalized position at $r$. If at that position old geometry gives unwrapped angle $\theta_{\text{cur}}$, then our new geometry should give $\theta_{\text{new}}$ that ideally makes $\theta_{\text{new}} - \theta_{\text{cur}} = \Delta\theta$, or minimizes the phase residuals across all measure measurement points $\theta_{\text{new}} - \theta_{\text{cur}} - \Delta\theta$.

### Initial Geometry
To calibrate for a specific disc, you need to guess a geometry first. You can use the geometry of a existing disc, or use conventional values.

We need to fix $r_0$ for initial calibration process. The true value can vary, but fixing a initial $r_0$ makes later jobs easier. Here are some valuses you can reference:

- CD (80 minute)
  - $r_0=25.00\ \text{mm}$ (range between 24.80 ~ 25.00 mm)
  - $p=1.51\ \mu\text{m}$ (range between 1.5 ~ 1.7 µm)
  - $N=22110.0$
- DVD (4.7GB)
  - $r_0=24.00\ \text{mm}$ (range between 23.80 ~ 24.00 mm)
  - $p=0.74\ \mu\text{m}$
  - $N=46000.0$

You may need to search in a area and find geometry that the spoke is clearly visible. If you find a geometry that produces visible spokes (even if heavily distorted), continue to next step.

### Coarse Revolution Calibration
Burn a disc and inspect how many revolutions the spokes have rotated (estimate fractional turns if they are not heavily distorted).

If the spokes rotate $m$ revolutions clockwise from the inner to the outer radius, our assumed $N$ is too small, so increase $N$ by $m$. If the spokes rotate counterclockwise, decrease $N$ by $m$.

If you cannot determine the exact number of revolutions, identify the winding direction and adjust $N$ until the revolutions become countable.

Repeat this process until the accumulated spoke rotation error is sufficiently small or close to zero revolutions.

### Fine Revolution Calibration Pitch Calibration
Once the coarse revolution count $N$ is calibrated, the remaining distortion mainly comes from an incorrect $p$, which can produce bow-shaped distortion of up to several revolutions.

If the bow leans clockwise, increase $p$, and vice versa.

[Here, we should use a calibration tool that calculates the corrected $p$ and $N$, but it is still under construction. Use the method described in the Idea section.]

Measure the phase error at different radii and use the tool to calculate the corrected $p$ and $N$. Burn discs with the new geometry. Repeat this process until the spokes follow the radial direction. You may still see S-shaped distortions, which are residual errors potentially caused by:
- Local variation in effective spiral pitch
- Drive-dependent behavior

The geometry is now calibrated.

For any remaining residual errors, apply additional phase correction.

### Radius Calibration (Optional)
Remember that our initial $r_0$ was chosen as a nominal value.

Once you have calibrated the geometry, you can optionally calibrate the actual radius. This can be done by limiting the program-area end radius or by direct measurement.

You can also skip this step. The image burned onto the disc might be slightly larger or smaller, but skipping this step does not introduce angular distortion.

#### Limit Program Area End Radius
Calculate the end radius of the geometry: $r_\text{end}=r_0+pN$. If $r_\text{end}$ exceeds 58 mm, we can scale down the whole image.

The scaling factor is $s=\cfrac{58\ \text{mm}}{r_\text{end}}$.

#### Measure and Calibrate
The spoke image should include rings at specified radii. For example, choose the ring at a radius of 55 mm. Manually measure its actual radius $r_\text{real}$.

The scaling factor is $s=\cfrac{r_\text{real}}{55\ \text{mm}}$.

#### Scale the Geometry
Multiply $r_0$ and $p$ by the scaling factor $s$ to obtain the new geometry, keeping $N$ unchanged. If you have residual phase correction samples, scale their radius coordinates by $s$ as well.

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

If byte order is wrong, double image may appear in data side.

Example CUE file:

```cue
FILE "payload.bin" BINARY
  TRACK 01 MODE1/2048 
    INDEX 01 00:00:00
```

Replace `MODE1/2048` with desired mode.

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