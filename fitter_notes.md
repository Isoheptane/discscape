Suppose we have:
- Current Geometry $G=(r_0, p, N)$
- Measured points $r_i$ and $\Delta n_i$, where $n_i$ is the revolution count error $\cfrac{\theta}{2\pi}$

We want to minimalize $n_\text{new}-n_\text{cur}-\Delta n_i$ on specific radius.

### Length of Spiral
Since $N \gg 1$, we can use concentric circles for total length approximation:

$L(r,p,N)=2\pi r_0N+\pi pN^2$

For a specific location at position $x$ on spiral, we can get normalized position $s = \cfrac{x}{L(r,p,N)}$.

### Radius to Normalized Position
$n=\cfrac{r-r_0}{p}$, $x=2\pi r_0 n+\pi p n^2$

Thus the normalized positions is

$$
\begin{aligned}
s
&= \cfrac{2\pi r_0 n + \pi p n^2}{2\pi r_0 N + \pi p N^2} \\
&= \cfrac{p(2r_0 n + pn^2)}{p(2r_0 N + pN^2)} \\
&= \cfrac{(r_0 + pn)^2 - r_0^2}{(r_0 + pN)^2 - r_0^2} \\
&= \cfrac{r^2 - r_0^2}{pN(2r_0 + pN)}
\end{aligned}
$$

### Normalized Position to Radius
Solve
$$
\begin{aligned}
s &=\cfrac{r^2-r_0^2}{pN(2r_0+pN)} \\
spN(2r_0+pN) &=r^2-r_0^2 \\
r^2&=spN(2r_0+pN)+r_0^2 \\
r&=\sqrt{spN(2r_0+pN)+r_0^2}
\end{aligned}
$$

### Theta to Normalized Position

Thus the normalized positions is
$$
\begin{aligned}
s &=\cfrac{2\pi r_0 n+\pi p n^2}{2\pi r_0 N + \pi p N^2} \\
s &=\cfrac{2r_0 n+p n^2}{2r_0 N + p N^2}
\end{aligned}
$$

### Normalized Position to Theta
Solve 

$$
\begin{aligned}
s &=\cfrac{2r_0 n+p n^2}{2r_0 N + p N^2} \\
(2r_0 N + p N^2)s &=2r_0 n+p n^2 \\
p n^2+2r_0n-(2r_0 N + p N^2)s&=0
\end{aligned}
$$

Thus
$$
\begin{aligned}
n&=\cfrac{-2r_0+\sqrt{4r_0^2+4ps(2r_0 N + p N^2)}}{2p} \\
n&=\cfrac{\sqrt{r_0^2+ps(2r_0 N + p N^2)} - r_0}{p} \\ \\
n&=\sqrt{\cfrac{r_0^2}{p^2}+\cfrac{s(2r_0N+pN^2)}{p}}-\cfrac{r0}{p}
\end{aligned}
$$

## Analyze 
For same normalized position $s$, we can see the radius change be

$$
\begin{aligned}
r(s)&=\sqrt{r_0^2+spN(2r_0+pN)}\\
n(s)&=\cfrac{1}{p}\left(\sqrt{r_0^2+ps(2r_0 N + p N^2)}-r_0\right)=\cfrac{r(s)-r0}{p} \\
\end{aligned}
$$

Assume $N$ $p$ change are relatively small, the changes are near linear.

Let $u=r^2=r_0^2+2r_0pNs+p^2N^2s$,

$$
\cfrac{\partial r}{\partial v}=\cfrac{\partial r}{\partial u} \cdot \cfrac{\partial u}{\partial v}=\cfrac{1}{2r}\cdot\cfrac{\partial u}{\partial v}
$$

---

#### Calculate $\cfrac{\partial n}{\partial N}$

$n=\cfrac{r-r_0}{p}$, thus $\cfrac{\partial n}{\partial N}=\cfrac{1}{p}\cdot\cfrac{\partial r}{\partial N}$

$$
\begin{aligned}
\cfrac{\partial u}{\partial N}
&=2r_0ps+2p^2Ns
=2ps(r_0+pN)

\\

\cfrac{\partial n}{\partial N}
&=\cfrac{1}{p}\cdot\cfrac{1}{2r}\cdot\cfrac{\partial u}{\partial N}
=
s\cfrac{r_0+pN}{r(s)}
\end{aligned}
$$

- $\left.\frac{\partial n}{\partial N}\right|_{s=0}=0$, starting points is not interfered by N
- $\left.\frac{\partial n}{\partial N}\right|_{s=1}=1$, ending point is interfered by N with 1:1 scale.

---

#### Calculate $\cfrac{\partial n}{\partial p}$

$n=\cfrac{r-r_0}{p}$, thus $\cfrac{\partial n}{\partial p}=\cfrac{p \frac{\partial r}{\partial p}-(r-r_0)}{p^2}=\cfrac{1}{p}\left(\cfrac{\partial r}{\partial p}-n(s) \right)$

$$
\begin{aligned}
\cfrac{\partial u}{\partial p}
&=2r_0Ns+2pN^2s=2Ns(r_0+pN)
\\
\cfrac{\partial r}{\partial p}
&=\cfrac{1}{2r}\cdot \cfrac{\partial u}{\partial p}
=\cfrac{Ns(r_0+pN)}{r(s)}=N\cfrac{\partial n}{\partial N} \\
\cfrac{\partial n}{\partial p}
&=\cfrac{1}{p}\left(N\cfrac{\partial n}{\partial N}-n(s) \right)=\cfrac{1}{p}\left(N\cfrac{\partial n}{\partial N}-\cfrac{r(s)-r_0)}{p}\right)
\end{aligned}
$$

- $\left.\frac{\partial n}{\partial p}\right|_{s=0}=0$, start point is not interfered by p
- $\left.\frac{\partial n}{\partial p}\right|_{s=1}=0$, end point is not interfered by p

---

Let the geometry delta be $\Delta N$, $\Delta p$. At any normalized position $s$, we have

$$
\Delta n(s)\approx \cfrac{\partial n}{\partial N}(s)\cdot \Delta N+\cfrac{\partial n}{\partial p}(s)\cdot \Delta p
$$



### Points
If a pivot point $(r_k, e_k)$ is set, then:
$$
\begin{aligned}
u_k\Delta N+v_k\Delta p=\Delta n_k \\
\Delta N = \cfrac{e_k - v_k\Delta p}{u_k}
\end{aligned}
$$

For minimal error:
$$
\sum_{i=1}^M (u_i\Delta N+v_i\Delta p-e_i)^2
$$

For any other points

$$
\begin{aligned}
e_i&=u_i\Delta N + v_i\Delta p \\
e_i&=u_i\left(\cfrac{e_k - v_k\Delta p}{u_k}\right) + v_i\Delta p \\
e_i&=\cfrac{u_i}{u_k}e_k +\left( v_i-\cfrac{u_i v_k}{u_k} \right)\Delta p \\
\end{aligned}
$$

