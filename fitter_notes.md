Suppose we have:
- Current Geometry $G=(r_0, p, N)$
- New Geometry $G'=(r_0, p', N')$
- Measured points $r_i$ and $\Delta n_i$, where $n_i$ is the revolution count error $\cfrac{\theta}{2\pi}$

We want to minimalize $n_\text{new}-n_\text{cur}-\Delta n_i$ on specific radius.

### Length of Spiral
Since $N \gg 1$, we can use concentric circles for total length approximation:

$L(r,p,N)=2\pi r_0N+\pi pN^2$

For a specific location at position $x$ on spiral, we can get normalized position $\tilde x = \cfrac{x}{L(r,p,N)}$.

### Radius to Normalized Position
$n=\cfrac{r-r_0}{p}$, $x=2\pi r_0 n+\pi p n^2$

Thus the normalized positions is

$$
\begin{aligned}
\tilde{x}
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
\tilde{x} &=\cfrac{r^2-r_0^2}{pN(2r_0+pN)} \\
\tilde{x}pN(2r_0+pN) &=r^2-r_0^2 \\
r^2&=\tilde{x}pN(2r_0+pN)+r_0^2 \\
r&=\sqrt{\tilde{x}pN(2r_0+pN)+r_0^2}
\end{aligned}
$$

### Theta to Normalized Position

Thus the normalized positions is
$$
\begin{aligned}
\tilde{x} &=\cfrac{2\pi r_0 n+\pi p n^2}{2\pi r_0 N + \pi p N^2} \\
\tilde{x} &=\cfrac{2r_0 n+p n^2}{2r_0 N + p N^2}
\end{aligned}
$$

### Normalized Position to Theta
Solve 

$$
\begin{aligned}
\tilde{x} &=\cfrac{2r_0 n+p n^2}{2r_0 N + p N^2} \\
(2r_0 N + p N^2)\tilde{x} &=2r_0 n+p n^2 \\
p n^2+2r_0n-(2r_0 N + p N^2)\tilde{x}&=0
\end{aligned}
$$

Thus
$$
\begin{aligned}
n&=\cfrac{-2r_0+\sqrt{4r_0^2+4p\tilde{x}(2r_0 N + p N^2)}}{2p} \\
n&=\cfrac{\sqrt{r_0^2+p\tilde{x}(2r_0 N + p N^2)} - r_0}{p} \\ \\
n&=\sqrt{\cfrac{r_0^2}{p^2}+\cfrac{\tilde{x}(2r_0N+pN^2)}{p}}-\cfrac{r0}{p}
\end{aligned}
$$

## Geometry Changes
For same normalized position $\tilde x$, we can see the radius change be

$$
\begin{aligned}
r(\tilde{x})&=\sqrt{r_0^2+\tilde{x}pN(2r_0+pN)}\\
n(\tilde{x})&=\cfrac{1}{p}\left(\sqrt{r_0^2+p\tilde{x}(2r_0 N + p N^2)}-r_0\right)=\cfrac{r(\tilde{{x}})-r}{p} \\
\end{aligned}
$$

Assume $N$ $p$ change are relatively small, the changes are near linear.

Let $u=r^2=r_0^2+2r_0pN\tilde{x}+p^2N^2\tilde{x}$,

$$
\cfrac{\partial r}{\partial v}=\cfrac{\partial r}{\partial u} \cdot \cfrac{\partial u}{\partial v}=\cfrac{1}{2r}\cdot\cfrac{\partial u}{\partial v}
$$

#### Calculate $\cfrac{\partial n}{\partial N}$ ($u_k$)

$n=\cfrac{r-r_0}{p}$, thus $\cfrac{\partial n}{N}=\cfrac{1}{p}\cdot\cfrac{\partial r}{\partial N}$

$$
\begin{aligned}
\cfrac{\partial u}{\partial N}
&=2r_0p\tilde{x}+2p^2N\tilde{x}
=2p\tilde{x}(r_0+pN)

\\

\cfrac{\partial n}{\partial N}
&=\cfrac{1}{p}\cdot\cfrac{1}{2r}\cdot\cfrac{\partial u}{\partial N}
=
\tilde{x}\cfrac{r_0+pN}{r(\tilde{x})}
\end{aligned}
$$

- $\left.\cfrac{\partial n}{\partial N}\right|_0=0$
- $\left.\cfrac{\partial n}{\partial N}\right|_1=1$


#### Calculate $\cfrac{\partial n}{\partial p}$ ($v_k$)

$n=\cfrac{r-r_0}{p}$, thus $\cfrac{\partial n}{\partial p}=\cfrac{p \frac{\partial r}{\partial p}-(r-r_0)}{p^2}=\cfrac{1}{p}\left(\cfrac{\partial r}{\partial p}-n(\tilde{x}) \right)$

$$
\begin{aligned}
\cfrac{\partial u}{\partial p}
&=2r_0N\tilde{x}+2pN^2\tilde{x}=2N\tilde{x}(r_0+pN)
\\
\cfrac{\partial r}{\partial p}
&=\cfrac{1}{2r}\cdot \cfrac{\partial u}{\partial p}
=\cfrac{N\tilde{x}(r_0+pN)}{r(\tilde{x})}=N\cfrac{\partial n}{\partial N} \\
\cfrac{\partial n}{\partial p}
&=\cfrac{1}{p}\left(N\cfrac{\partial n}{\partial N}-n(\tilde{x}) \right)=\cfrac{1}{p}\left(N\cfrac{\partial n}{N}-\cfrac{r(\tilde{x})-r_0)}{p}\right)
\end{aligned}
$$

- $\left.\cfrac{\partial n}{\partial p}\right|_0=0$
- $\left.\cfrac{\partial n}{\partial p}\right|_1=0$

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

