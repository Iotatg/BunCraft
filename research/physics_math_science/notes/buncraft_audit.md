# BunCraft「假物理 / 假数学」审计报告

> 目标：定位「哪里在用假物理 / 假数学，哪里缺真科学」，为后续引入 `nalgebra` / `kurbo` / `lyon` / `palette` / `avian` / `rapier` 做准备。
> 范围：任务书指定的 6 个 crate（`buncraft-core`、`buncraft-anime`、`buncraft-render`、`buncraft-brain`、`buncraft-api`、`buncraft-cli`）全部源码，以及 `docs/`、`README.md`。
> 方法：逐文件 `nl -ba` 通读 + 全仓 `rg -n` 反证检索。所有结论都带 `路径:行号`；检索不到的一律写「未找到」，不推断、不编造。
> 约束：**未修改任何源码**，本文件是唯一新增产物。

## 0. 基线与边界说明

| 项目 | 事实 | 证据 |
|---|---|---|
| 审计对象的依赖面 | 6 个被审计 crate 均无科学计算依赖（仅 `serde`/`tiny-skia`/`clap`/`axum`/`tokio` 等） | `crates/buncraft-core/Cargo.toml:9-12`；`crates/buncraft-render/Cargo.toml:9-12`；`crates/buncraft-anime/Cargo.toml:9-10`；`crates/buncraft-brain/Cargo.toml:9-14`；`crates/buncraft-cli/Cargo.toml:13-20`；`crates/buncraft-api/Cargo.toml:13-20` |
| 唯一图形/数学外部库 | `tiny-skia 0.11`（光栅化），6 个 crate 中无 `nalgebra`/`kurbo`/`lyon`/`palette`/`avian`/`rapier` | `Cargo.toml:32`；`Cargo.lock` 中 `name = "tiny-skia"` `version = "0.11.4"` |
| 测试基线（6 个被审计 crate） | 本次审计实测 `cargo test --workspace`（science crate 加入前）：11 组 `test result: ok`，非空组合计 **49 passed / 0 failed**；测试清单见 README | `README.md:163-172` |
| 上游仓库 | 11 个上游已克隆到 `research/upstream/`（含本任务 6 个目标库），精确 SHA 与许可证见 manifest | `research/physics_math_science/repo_manifest.md:3-6`、`research/physics_math_science/repo_manifest.md:10-20` |
| 上游真公式 | sRGB EOTF/OETF、kurbo 自适应 Gauss–Legendre 弧长、XPBD compliance 等已由组长核对 | `research/physics_math_science/notes/verified_formulas.md:9-38`、`research/physics_math_science/notes/verified_formulas.md:61-98` |

**边界说明（重要，避免与其他成员并发工作冲突）**：审计进行期间（约 17:20 之后）另有一位成员新建了 `crates/buncraft-science/`，并在本笔记定稿前被加入 workspace：`Cargo.toml:10`（`members` 共 7 项，`Cargo.toml:3-11`）、`Cargo.toml:26`（`[workspace.dependencies]` 路径依赖，`Cargo.toml:21-36`）。其 `crates/buncraft-science/src/lib.rs:21-24` 声明了 `color` / `curve` / `light` / `xpbd` 四个模块，已包含真科学实现：`crates/buncraft-science/src/color.rs:14-29`（sRGB EOTF/OETF）、`crates/buncraft-science/src/light.rs:26-30`（Lambert，带 `ρ/π` 归一化）、`crates/buncraft-science/src/xpbd.rs:6-16`（XPBD compliance `Δλ`）、`crates/buncraft-science/src/curve.rs:36`（积分 `|C'(t)|`）。

但：
1. 6 个被审计 crate 中**未找到**任何 `buncraft-science` / `buncraft_science` 引用（检索：`rg -n 'buncraft-science|buncraft_science' crates/buncraft-{core,render,anime,brain,cli,api}/` → 未找到）；其 `Cargo.toml:11` 的 `[dependencies]` 为空。因此本文 §1–§7 所有「未找到」「无任何物理模拟」「无线性光」结论**严格限定于任务书指定的 6 个 crate**，不受影响。
2. 定稿时 `cargo test --workspace` 已包含该 WIP crate，实测其 18 个测试中 14 通过 / 4 失败：`crates/buncraft-science/src/color.rs:155`、`crates/buncraft-science/src/color.rs:161`、`crates/buncraft-science/src/curve.rs:208`、`crates/buncraft-science/src/curve.rs:217`。**故定稿时刻整个 workspace 套件并非全绿**；§9 的「全绿」判据应以 6 个被审计 crate 的 49 个回归测试为闸门，或按 `-p` 分批验收，待该 crate 修复后再要求 `--workspace` 整体全绿。
3. 若该 crate 后续被 6 个 crate 引用，§2、§6、§8 的现状列需要复核更新；§8 的「建议落点」可直接改为复用 `buncraft-science`。

**后续更新（2026-11-16）**：上述 4 个失败已全部修复，`crates/buncraft-science` 现为 **18 passed / 0 failed**；整个 workspace 7 个 crate 合计 **67 passed / 0 failed**（`cargo test --release`）。修复内容：`curve.rs` 的 Gauss–Legendre 前因子由 `0.25` 改为正确的 `0.5`（映射 `t=(x+1)/2` 使 `dt=dx/2`）；`color.rs` 的 Oklab 往返与白色锚点容差放宽到 `1e-7`/`1e-6`，原因是 M1/M2 及其逆矩阵各自截断到 10 位小数，往返误差约 `1e-7`（远小于 1 个 8-bit 步长）。因此 §9 关于「`--workspace` 整体全绿」的闸门现在可以启用。**注意**：§9 建议直接引入 `palette`/`kurbo`/`avian` 作为依赖，但因上游为 edition 2024 / MSRV 1.85–1.89 而 BunCraft 为 1.80，实际采用的是零依赖独立实现 `buncraft-science`；§9 的 4 步应理解为「按同样的科学内容接线」，而非「必须添加这些依赖」。

---

## 1. 数学：`crates/buncraft-core/src/geom.rs`

### 1.1 `Vec2` 运算符（`geom.rs:8-67`）

`Vec2` 只有两个 `f32` 分量，坐标约定为「原点左上、+x 右、+y 下、单位 = 像素」（`geom.rs:1-4`）。**没有 `std::ops` 的 `Add`/`Sub`/`Mul`/`Neg` 实现**，全部是具名方法：

```rust
// crates/buncraft-core/src/geom.rs:21-35
    pub fn add(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x + o.x, self.y + o.y)
    }

    pub fn sub(self, o: Vec2) -> Vec2 {
        Vec2::new(self.x - o.x, self.y - o.y)
    }

    pub fn mul(self, s: f32) -> Vec2 {
        Vec2::new(self.x * s, self.y * s)
    }

    pub fn len(self) -> f32 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
```

- `add`/`sub`：逐分量加减（`geom.rs:21-27`）。`mul`：只有**标量乘**（`geom.rs:29-31`），无向量点乘/叉乘/逐分量乘、无除法、无取负、无 clamp/min/max、无 `length_squared`。
- `dist` = `|a−b|`（`geom.rs:37-39`）；`norm` = `v/|v|`，但 `|v| < 1e-6` 时直接返回 `Vec2::ZERO`（`geom.rs:41-48`）——退化输入给零向量，不是定义的法线/切线。
- `lerp` = `p + (q−p)·t`（`geom.rs:50-52`），坐标线性插值，不是弧长插值、不是 slerp。
- `perp` = `(-y, x)`（`geom.rs:54-57`），注释写 "Rotate 90 degrees counter-clockwise in screen space"（`geom.rs:54`）；但在 +y 向下的屏幕坐标系里 `(+1,0) → (0,+1)` 是**视觉上的顺时针**。注释与坐标系不一致，值得核对。全仓 `rg -n '\.perp\(|\.rot\('` 在 6 个 crate 中**未找到调用点**（仅 `geom.rs:55`、`geom.rs:59` 定义）；`crates/buncraft-core/src/lib.rs:18` 的 re-export 也未包含它们作为自由函数。
- `rot` 用标准二维旋转矩阵（`geom.rs:59-62`，`sin_cos()` 返回 `(sin, cos)`，公式 `x·c−y·s, x·s+y·c` 正确）。同样无调用点。
- `dot` = `x·x' + y·y'`（`geom.rs:64-66`）；仅 2 处真正使用，都在 `distance_to_polyline`：`geom.rs:305`（`len2`）、`geom.rs:309`（投影参数）。
- `Bounds`（`geom.rs:69-117`）是**轴对齐包围盒 AABB**：`empty()` 用 ±INFINITY（`geom.rs:76-81`）、`include` 逐轴 min/max（`geom.rs:87-92`）、`union`（`geom.rs:94-97`）、`center/width/height`（`geom.rs:103-116`）。使用处：`Stroke::bounds`（`stroke.rs:282-288`）、`FillRegion::bounds`（`document.rs:86-92`）、`Document::bounds`（`document.rs:422-428`）、`ink_coverage`（`render/src/lib.rs:571-578`）。**没有 OBB / 凸包 / 包围球 / 曲线精确包围盒**（AABB 只由采样点生成，会低估曲线控制点范围）。

### 1.2 `catmull_rom` / `catmull_rom_closed` / `catmull_segment`

```rust
// crates/buncraft-core/src/geom.rs:167-181
pub fn catmull_segment(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let t2 = t * t;
    let t3 = t2 * t;
    let x = 0.5
        * ((2.0 * p1.x)
            + (-p0.x + p2.x) * t
            + (2.0 * p0.x - 5.0 * p1.x + 4.0 * p2.x - p3.x) * t2
            + (-p0.x + 3.0 * p1.x - 3.0 * p2.x + p3.x) * t3);
    let y = 0.5
        * ((2.0 * p1.y)
            + (-p0.y + p2.y) * t
            + (2.0 * p0.y - 5.0 * p1.y + 4.0 * p2.y - p3.y) * t2
            + (-p0.y + 3.0 * p1.y - 3.0 * p2.y + p3.y) * t3);
    Vec2::new(x, y)
}
```

这是标准**均匀（uniform / α=0）Catmull–Rom** 基：`0.5·[2p1 + (−p0+p2)t + (2p0−5p1+4p2−p3)t² + (−p0+3p1−3p2+p3)t³]`。

- 采样方式：`samples = samples_per_segment.max(1)`（`geom.rs:128`）；对每段 `i` 取 `p0 = points[i−1]`（首段重复 `points[0]`，`geom.rs:132`）、`p1 = points[i]`、`p2 = points[i+1]`、`p3 = points[i+2]`（末段重复最后一个点，`geom.rs:135`）；`t = s / samples`（`geom.rs:137`），`s ∈ 0..samples`；最后补 `points[n−1]`（`geom.rs:141`）。共 `(n−1)·samples + 1` 点。
- **不是弧长等距**：`t` 是**均匀参数**，不是弦长参数化（chordal，α=1）也不是向心参数化（centripetal，α=0.5）。控制点间距不均时，每段拿到的采样数完全相同（`geom.rs:131-140`），高曲率段被欠采样、直段被过采样；且均匀 CR 在不等距锚点下会产生 overshoot / 自交。曲线的一阶导 `C'(t)`、二阶导、曲率、弧长函数 `s(t)` 与反函数 `t(s)` 全部**未找到**。
- 调用参数写死：`from_control_points` 用 10（`stroke.rs:306`）、`from_closed_control_points` 用 12（`stroke.rs:332`）。闭环版 `catmull_rom_closed`（`geom.rs:146-165`）只是把索引取模（`geom.rs:154-157`）并补 `out[0]`（`geom.rs:163`）。
- 唯一的曲线测试只验证「过锚点 / 闭合 / 重采样保端点 / 正方形面积」：`geom.rs:317-364`（`catmull_passes_through_anchors` `321-333`、`closed_curve_is_closed` `335-344`、`resample_keeps_endpoints` `346-353`、`polygon_area_of_square` `355-364`）。没有速度、曲率、弧长、连续性（C¹/G¹）断言。

### 1.3 `resample`（`geom.rs:183-213`）

```rust
// crates/buncraft-core/src/geom.rs:185-213
pub fn resample(points: &[Vec2], spacing: f32) -> Vec<Vec2> {
    if points.len() < 2 || spacing <= 0.01 {
        return points.to_vec();
    }
    let mut out = vec![points[0]];
    let mut carry = 0.0f32;
    for w in points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let seg = a.dist(b);
        if seg <= 1e-6 {
            continue;
        }
        let mut d = spacing - carry;
        while d < seg {
            out.push(a.lerp(b, d / seg));
            d += spacing;
        }
        carry = seg - (d - spacing);
        carry %= spacing;
    }
    if let Some(last) = points.last() {
        if out.last().map(|p| p.dist(*last)).unwrap_or(0.0) > spacing * 0.25 {
            out.push(*last);
        } else if let Some(slot) = out.last_mut() {
            *slot = *last;
        }
    }
    out
}
```

- 实际公式：沿**折线**累积弧长，用 `carry` 记录「跨段后已消耗的间距」；每段下一采样点位置 `d = spacing − carry`（`geom.rs:197`），逐次 `d += spacing`（`geom.rs:200`），插值 `a.lerp(b, d/seg)`（`geom.rs:199`，线性插值在**弦**上而不是曲线上）；段末 `carry = seg − (d − spacing)` 再 `%= spacing`（`geom.rs:202-203`）。这在折线意义下是近似正确的等距重采样。
- **但仍不是真·弧长等距，误差来源有 5 处**：
  1. 输入是 `catmull_rom` 的**折线近似**，累计的是**弦长**而非样条真实弧长；且上游折线本身是均匀 `t` 采样（`geom.rs:137`），高曲率处弦长系统性低估真实弧长。
  2. 采样点落在弦上（`geom.rs:199`），不在样条曲线上，因此输出点本身偏离 CR 曲线。
  3. `spacing <= 0.01` 直接原样返回（`geom.rs:186-188`），`seg <= 1e-6` 跳过但不更新 `carry`（`geom.rs:194-196`）。
  4. 端点修正（`geom.rs:205-211`）：末点距真实终点 `> 0.25·spacing` 就追加、否则**覆盖**最后一个点 —— 最后一段间距可落在 `[0, 1.25·spacing]`，不是严格 `spacing`。
  5. 流水线里被**重采样两次且步长不同**：作者期 `spacing = (brush.size * 0.45).clamp(1.0, 5.0)`（`stroke.rs:307`、`stroke.rs:333`、`stroke.rs:354`），渲染期又按 `spacing = (radius * 0.55).clamp(0.5, 3.0)` 插 stamp（`render/src/lib.rs:239-249`）。
- 另一个「假数学」：笔压 `pressure_profile(t, taper)` 的 `t` 是**采样点序号比例** `i/(n−1)`（`stroke.rs:315`、`stroke.rs:338`、`stroke.rs:362`），不是弧长比例；端点修正使间距不均时， taper 在空间上非线性。

### 1.4 `ellipse` / `ellipse_point` / `arc`（`geom.rs:255-291`）

```rust
// crates/buncraft-core/src/geom.rs:256-272
pub fn ellipse_point(center: Vec2, rx: f32, ry: f32, angle: f32) -> Vec2 {
    Vec2::new(
        center.x + rx * angle.cos(),
        center.y + ry * angle.sin(),
    )
}

/// Sample a full ellipse as a closed polyline.
pub fn ellipse(center: Vec2, rx: f32, ry: f32, samples: usize) -> Vec<Vec2> {
    let n = samples.max(8);
    let mut pts = Vec::with_capacity(n + 1);
    for i in 0..=n {
        let a = std::f32::consts::TAU * i as f32 / n as f32;
        pts.push(ellipse_point(center, rx, ry, a));
    }
    pts
}
```

- 公式是参数方程 `x = cx + rx·cosθ, y = cy + ry·sinθ`（`geom.rs:257-260`）。`n = samples.max(8)`（`geom.rs:265`），`i = 0..=n` 含端点（`geom.rs:267-270`），因此 `pts[0] == pts[n]`（闭合折线）。
- **采样是「角度等距」而非「弧长等距」**：`θ_i = 2πi/n`（`geom.rs:268`）。由该公式可解析推出 `ds/dθ = √(rx²sin²θ + ry²cos²θ)`，故 `θ=0` 处步长 ∝ `ry`、`θ=π/2` 处 ∝ `rx`，最大/最小弦长比为 `max(rx,ry)/min(rx,ry)`。例如代码里真实存在的 `rx=26, ry=40`（`buncraft-anime/src/lib.rs:351`）比值为 1.54；`rx = w·1.10, ry = h·0.56`（`buncraft-anime/src/lib.rs:203`）在默认 `w=186, h=470` 下约为 204.6/263.2 ≈ 1.29。**误差来源：角度参数与弧长参数不一致，没有任何 `inv_arclen` / 自适应求积。**
- `arc`（`geom.rs:275-291`）同样按角度线性 `a = start + (end−start)·t`（`geom.rs:287`）均匀采样，问题相同。
- 椭圆没有用二次/三次 Bézier 表示，只是折线；没有 `ellipse` 的曲率、切线、法线、焦距/离心率等概念。

### 1.5 `mirror_x`（`geom.rs:250-253`）

```rust
// crates/buncraft-core/src/geom.rs:250-253
/// Mirror points across a vertical axis.
pub fn mirror_x(points: &[Vec2], axis: f32) -> Vec<Vec2> {
    points.iter().map(|p| Vec2::new(2.0 * axis - p.x, p.y)).collect()
}
```

纯竖直轴反射 `x' = 2·axis − x, y' = y`。**没有通用仿射变换、没有旋转中心、没有齐次坐标、没有透视/投影矩阵、没有 3D 变换**。同一个反射被内联重复实现于 `geometry.rs:114`（`face_outline`）、`geometry.rs:180`（`hair_silhouette`）、`geometry.rs:393`（测试），调用点在 `buncraft-anime/src/lib.rs:378`、`lib.rs:885`、`lib.rs:1054`、`lib.rs:1653`。这直接决定了脸和头发是**严格左右镜像对称**的（§5）。

### 1.6 缺失项逐条判定（带反证检索）

| 能力 | 结论 | 证据 |
|---|---|---|
| 曲率 `κ(t)` | **未找到** | 全仓 `rg -n -i curvature` 在 6 个 crate 中无匹配；`geom.rs:317-364` 的测试只有锚点/闭合/面积/端点 |
| 切线 / 法线 | **未找到** | `rg -n -i normal` 仅命中 `crates/buncraft-brain/src/dataset.rs:78` 的 `normalise()`（归一化坐标）与 `crates/buncraft-core/src/rng.rs:40` 的 "Standard normal (Box-Muller)"（统计分布），均与曲面法线无关；表面法线、`Vec3`、法线矩阵一律未找到 |
| 叉积 | **未找到独立叉积函数** | 只有多边形里标量楔积 `a.x*b.y - b.x*a.y`（`geom.rs:224`、`geom.rs:243`），用于 shoelace 面积（`geom.rs:216-227`）与质心（`geom.rs:229-248`）；`Vec2::perp`（`geom.rs:55-57`）可当 90° 旋转但无调用点；**无 3D 叉积** |
| 包围盒 | **有，但仅 AABB** | `Bounds`（`geom.rs:69-117`）、`Stroke::bounds`（`stroke.rs:282-288`）、`Document::bounds`（`document.rs:422-428`）；OBB / 凸包 / 包围球 / 精确曲线包围盒**未找到** |
| 路径布尔运算 | **未找到** | `docs/limitations.md:17` 明写 "Path booleans. Intersections are layered, not merged."；`research/research_gaps.md:42-43`；`docs/roadmap.md:18` 把它列为待办。现有 `clip` 只是多边形掩膜（`render/src/lib.rs:113-119`、`render/src/lib.rs:149-173`、`render/src/lib.rs:188-202`），不是并/交/差集合运算 |
| 少数「真数学」 | 有，但很有限 | 点到折线距离用投影 + clamp（`geom.rs:294-315`）、shoelace 面积（`geom.rs:216-227`）、多边形质心（`geom.rs:229-248`）——公式正确，但都是初等解析几何 |

---

## 2. 色彩：`buncraft-core/src/stroke.rs` + `buncraft-anime`

### 2.1 `Rgba` 的定义与三个函数（`stroke.rs:50-101`）

```rust
// crates/buncraft-core/src/stroke.rs:50-56
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}
```

结构体只有 4 个 `u8`，**没有色彩空间标签、没有 ICC profile、没有线性/伽马标记、没有浮点通道、没有色域/白点信息**（`stroke.rs:50-56`）。

```rust
// crates/buncraft-core/src/stroke.rs:71-100
    pub fn with_alpha(self, a: f32) -> Self {
        Self {
            a: (a.clamp(0.0, 1.0) * 255.0).round() as u8,
            ..self
        }
    }

    pub fn alpha_f(self) -> f32 {
        self.a as f32 / 255.0
    }

    /// Perceived brightness in `0.0..=1.0` (Rec. 601 weights).
    ///
    /// Shading and highlighting need this to pick a tone step that is actually
    /// visible: a 40% white mix does nothing to near-white hair, and a 40%
    /// black mix does nothing to near-black hair.
    pub fn luminance(self) -> f32 {
        (0.299 * self.r as f32 + 0.587 * self.g as f32 + 0.114 * self.b as f32) / 255.0
    }

    pub fn mix(self, other: Rgba, t: f32) -> Rgba {
        let t = t.clamp(0.0, 1.0);
        let m = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
        Rgba::new(
            m(self.r, other.r),
            m(self.g, other.g),
            m(self.b, other.b),
            m(self.a, other.a),
        )
    }
```

- **`mix`（`stroke.rs:91-100`）**：逐通道 `a + (b−a)·t` 后 `round()`，`t` clamp（`stroke.rs:92`）。运算空间是**伽马编码（sRGB-like）的 8 位整数值**，不是线性光（linear radiance）。光强相加必须在线性空间做；直接在编码值上插值，黑→白 `t=0.5` 得到字节 128，其线性辐亮度约 0.214（sRGB EOTF，见 `research/physics_math_science/notes/verified_formulas.md:63-78`），**不是 0.5**。此外 `mix` 连 `a` 通道一起插值（`stroke.rs:98`），颜色混合会改变不透明度；也没有 Oklab / CIELAB 之类的感知均匀空间，所以色相会漂移。
- **`with_alpha`（`stroke.rs:71-76`）**：只改 `a`，RGB 原样保留（`..self`），**不做预乘 alpha、不做 RGB 裁剪**，不是 `premultiply` 操作。全仓仅 1 处调用：`buncraft-anime/src/lib.rs:1458`（`C_LIP.with_alpha(0.55)`）。
- **`luminance`（`stroke.rs:82-89`）**：`0.299R' + 0.587G' + 0.114B'` 除以 255，作用在**伽马编码字节**上，这是 Rec.601 **luma Y′**，不是光度学 luminance Y。正确做法是先做 sRGB 解码再按 Rec.709 线性权重 `0.2126/0.7152/0.0722`（对照 `crates/buncraft-science/src/color.rs:74-76`，该 crate 不在本次审计范围）。注释自称 "Perceived brightness"（`stroke.rs:82`）而函数名叫 `luminance`，术语混用。全仓唯一调用点：`buncraft-anime/src/lib.rs:1470`（用来决定高光色）。
- `hex()`（`stroke.rs:67-69`）**丢弃 alpha**：`format!("#{:02x}{:02x}{:02x}", ...)`，只输出 RGB；`alpha_f()`（`stroke.rs:78-80`）**未找到调用点**。测试只验证端点与绿>蓝（`stroke.rs:402-408`），无线性光/ΔE 断言。

### 2.2 `buncraft-anime` 里所有颜色常量（`Rgba::new` / `Rgba::rgb`）

模块级常量（`lib.rs:22-30`，全部 `a=255`）：

| 行 | 定义 | 值 |
|---|---|---|
| `buncraft-anime/src/lib.rs:22` | `C_SKETCH` | `Rgba::new(116, 136, 168, 255)` |
| `buncraft-anime/src/lib.rs:23` | `C_CONSTRUCTION` | `Rgba::new(148, 168, 200, 255)` |
| `buncraft-anime/src/lib.rs:24` | `C_INK` | `Rgba::new(44, 40, 50, 255)` |
| `buncraft-anime/src/lib.rs:25` | `C_SKIN` | `Rgba::new(250, 228, 214, 255)` |
| `buncraft-anime/src/lib.rs:26` | `C_SKIN_SHADOW` | `Rgba::new(228, 188, 182, 255)` |
| `buncraft-anime/src/lib.rs:27` | `C_SCLERA` | `Rgba::new(252, 252, 255, 255)` |
| `buncraft-anime/src/lib.rs:28` | `C_PUPIL` | `Rgba::new(36, 32, 42, 255)` |
| `buncraft-anime/src/lib.rs:29` | `C_LIP` | `Rgba::new(212, 136, 138, 255)` |
| `buncraft-anime/src/lib.rs:30` | `C_BLUSH` | `Rgba::new(240, 148, 160, 255)` |

内联 `Rgba::rgb(...)`（`rg -n 'Rgba::rgb' crates/buncraft-anime/src/` 全量）：

| 行 | 表达式 | 用途 |
|---|---|---|
| `buncraft-anime/src/lib.rs:1280` | `Rgba::rgb(198, 214, 238)` | 背景氛围洗色 halo，opacity 0.42 |
| `buncraft-anime/src/lib.rs:1417` | `Rgba::rgb(26, 20, 38)` | iris 暗部混色目标 |
| `buncraft-anime/src/lib.rs:1431` | `Rgba::rgb(255, 255, 255)` | iris 亮部混色目标 |
| `buncraft-anime/src/lib.rs:1477` | `Rgba::rgb(255, 255, 255)` | 浅发色高光目标 |
| `buncraft-anime/src/lib.rs:1479` | `Rgba::rgb(198, 210, 246)` | 深发色高光「冷色」目标 |
| `buncraft-anime/src/lib.rs:1502` | `Rgba::rgb(255, 255, 255)` | 眼神光（catchlight）纯白 |
| `buncraft-anime/src/lib.rs:1549` | `Rgba::rgb(214, 190, 196)` | 额头投影 |
| `buncraft-anime/src/lib.rs:1598` | `Rgba::rgb(206, 186, 196)` | 眼睑投影 |
| `buncraft-anime/src/lib.rs:1607` | `Rgba::rgb(40, 36, 54)` | 头发内侧阴影混色目标 |
| `buncraft-anime/src/lib.rs:1630` | `Rgba::rgb(226, 206, 208)` | 刘海下方投影 |

`mix(` 调用全量（`rg -n 'mix\(' crates/buncraft-anime/` 恰好 5 处，无遗漏）：

| 行 | 调用 | 系数 |
|---|---|---|
| `buncraft-anime/src/lib.rs:1417` | `self.p.eyes.color.mix(Rgba::rgb(26, 20, 38), 0.46)` | 0.46 |
| `buncraft-anime/src/lib.rs:1431` | `self.p.eyes.color.mix(Rgba::rgb(255, 255, 255), 0.52)` | 0.52 |
| `buncraft-anime/src/lib.rs:1477` | `hair.mix(Rgba::rgb(255, 255, 255), 0.80)` | 0.80 |
| `buncraft-anime/src/lib.rs:1479` | `hair.mix(Rgba::rgb(198, 210, 246), 0.58)` | 0.58 |
| `buncraft-anime/src/lib.rs:1607` | `self.p.hair.color.mix(Rgba::rgb(40, 36, 54), 0.45)` | 0.45 |

### 2.3 高光 / 阴影硬编码值（逐条）

- **高光选择逻辑**（`lib.rs:1470-1482`）：`let lum = hair.luminance();`（`1470`）；阈值 `lum > 0.72`（`1476`、`1482`）；浅发 `mix(白, 0.80)`（`1477`）、深发 `mix((198,210,246), 0.58)`（`1479`）；band 厚度 `1.0` / `0.82`（`1482`）。注释自己承认是经验判断："Near-white hair has no headroom left... dark hair takes a cool tint instead"（`lib.rs:1471-1475`）。没有 Fresnel、`N·H`、粗糙度、光源方向。
- **iris 高光/暗部**：`mix(白, 0.52)`（`lib.rs:1431`）、`mix((26,20,38), 0.46)`（`lib.rs:1417`）、纯白眼神光（`lib.rs:1502`）。
- **阴影硬编码**：`C_SKIN_SHADOW` 常量（`lib.rs:26`）用于颈部投影 `air(58.0, ..., 0.34)`（`lib.rs:1519`）与 hatch（`lib.rs:1539`）；额头 `(214,190,196)` @0.30（`lib.rs:1549`）；腮红 `C_BLUSH` @0.16（`lib.rs:1569`）/ @0.35（`lib.rs:1579`）；眼睑 `(206,186,196)` @0.45（`lib.rs:1598`）；头发内侧 `mix((40,36,54),0.45)` 后 @0.13（`lib.rs:1607`、`lib.rs:1616`）；刘海下 `(226,206,208)` @0.10（`lib.rs:1630`）；唇色 `C_LIP.with_alpha(0.55)`（`lib.rs:1458`）。
- **补充（范围外但同源）**：`CharacterProfile::from_prompt` 里还有一整套硬编码调色板：发色 `crates/buncraft-core/src/profile.rs:147-171`、瞳色 `profile.rs:196-210`、服装色 `profile.rs:258-268`、强调色阈值 `profile.rs:270-275`、肤色 `profile.rs:297`。它们同样是无色域/无 illuminant 的 sRGB 字节常量。

---

## 3. 渲染与合成：`crates/buncraft-render/src/lib.rs`

### 3.1 预乘 alpha 合成公式（`circle`，`lib.rs:305-350`）

```rust
// crates/buncraft-render/src/lib.rs:325-347
    let a0 = alpha.clamp(0.0, 1.0);
    let (cr, cg, cb) = (color.r as f32, color.g as f32, color.b as f32);
    let data = pm.data_mut();
    for py in y0..y1 {
        let dy = py as f32 + 0.5 - y;
        for px in x0..x1 {
            let dx = px as f32 + 0.5 - x;
            let d2 = dx * dx + dy * dy;
            if d2 > (r + 0.5) * (r + 0.5) {
                continue;
            }
            let d = d2.sqrt();
            let cov = (r + 0.5 - d).clamp(0.0, 1.0);
            let a = a0 * cov;
            if a <= 0.0005 {
                continue;
            }
            let idx = ((py * w + px) * 4) as usize;
            let inv = 1.0 - a;
            data[idx] = (cr * a + data[idx] as f32 * inv + 0.5) as u8;
            data[idx + 1] = (cg * a + data[idx + 1] as f32 * inv + 0.5) as u8;
            data[idx + 2] = (cb * a + data[idx + 2] as f32 * inv + 0.5) as u8;
            data[idx + 3] = (255.0 * a + data[idx + 3] as f32 * inv + 0.5) as u8;
```

- 公式是 `dst' = C·a + dst·(1−a)`（RGB）与 `A' = 255·a + A·(1−a)`，其中 `a = a0·cov`（`lib.rs:338`），`a0` 来自 `brush.opacity × pressure × grain` 等（`lib.rs:277`、`lib.rs:285`、`lib.rs:290`、`lib.rs:295`、`lib.rs:299`）。tiny-skia 0.11.4 的 `Pixmap` 确为**预乘 RGBA8** 存储（`/root/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tiny-skia-0.11.4/src/pixmap.rs:26`），所以这在「预乘空间」里是标准 SourceOver；`cr*a` 相当于预乘源。
- **但「假物理」在颜色空间**：`cr/cg/cb` 直接取 `u8` 字节（`lib.rs:326`），`mix` 也在同一编码空间（`stroke.rs:91-100`），整个合成在**伽马编码信号**上线性相加，而不是线性辐亮度相加。对照上游 sRGB EOTF/OETF（`research/physics_math_science/notes/verified_formulas.md:63-78` 明确指出 `lib.rs:343-347` 的 `cr*a + dst*inv` "physically wrong"）。`to_color`（`lib.rs:58-61`）同样直接用 `Color::from_rgba8(c.r, c.g, c.b, ...)`，无传递函数。
- **抗锯齿**：`cov = (r + 0.5 − d).clamp(0.0, 1.0)`（`lib.rs:337`）只是 1 像素宽的**线性覆盖度斜坡**，不是圆面积解析积分、不是超采样/MSAA；`+0.5` 是四舍五入近似（`lib.rs:344-347`）。路径填充分支才开 `paint.anti_alias = true`（`lib.rs:154`、`lib.rs:178`、`lib.rs:358`、`lib.rs:392`），掩膜也开 `true`（`lib.rs:200`）。图层合成处 `quality: FilterQuality::Nearest`（`lib.rs:102`），无滤波器。
- **`color.a` 被忽略的不一致（潜在缺陷）**：快速路径 `circle()` 只用 `color.r/g/b`（`lib.rs:326`），**从不乘 `color.a/255`**；而 `r > 26.0` 时切到 `circle_path`（`lib.rs:312-315`），它用 `to_color(color, alpha)`（`lib.rs:357`），`to_color` 里有 `(c.a as f32 / 255.0) * alpha_mul`（`lib.rs:59`）。`soft_circle` 也用 `to_color`（`lib.rs:382-384`）。因此当 `brush.color.a ≠ 255` 时，半径 26px 分界处会出现不连续。当前 6 个 crate 中 `with_alpha` 只有 `lib.rs:1458` 一处且用于 `Fill`（走 `draw_fill` → `to_color`，`lib.rs:153`），所以缺陷尚未显形，但接口不一致客观存在。

### 3.2 `soft_circle` 的 `RadialGradient` 停靠点（`lib.rs:368-400`）

```rust
// crates/buncraft-render/src/lib.rs:377-388
    let shader = RadialGradient::new(
        Point::from_xy(x, y),
        Point::from_xy(x, y),
        r,
        vec![
            GradientStop::new(0.0, to_color(color, alpha)),
            GradientStop::new(0.55, to_color(color, alpha * 0.55)),
            GradientStop::new(1.0, to_color(color, 0.0)),
        ],
        SpreadMode::Pad,
        Transform::identity(),
    );
```

两个圆心重合（`lib.rs:378-379`）、半径 `r`（`lib.rs:380`，由 `soft_circle(pm, x, y, r * 1.6, ...)` 传入，`lib.rs:300`）。停靠点仅 3 个：`0.0 → alpha`、`0.55 → alpha·0.55`、`1.0 → 0`（`lib.rs:382-384`），`SpreadMode::Pad`（`lib.rs:386`）。因此衰减是**半径上的分段线性函数**（0→0.55 线性降到 55%，0.55→1 线性降到 0），不是高斯 / 余弦 / 物理点扩散函数；插值同样发生在 sRGB 字节空间。半径 `r` 由 `base_r·scale` 得来（`lib.rs:271`），与压力无关（除 `alpha`）。`paint.anti_alias = true`（`lib.rs:392`）。

`to_svg` 里对应的单点 airbrush dab 也导出同样的 3 停靠点（`render/src/lib.rs:448-451`：`offset="0"` / `"0.55"` / `"1"`，透明度 `alpha` / `alpha*0.55` / `0`），半径 `s.brush.size * 0.8`（`lib.rs:444`）——与光栅端 `0.5·size·1.6 = 0.8·size`（`lib.rs:255`、`lib.rs:300`）在 `scale=1` 时一致。

### 3.3 `to_svg` 的路径与渐变导出（`lib.rs:408-533`）

- 画布背景用 `doc.canvas.background.hex()`（`lib.rs:418`），hex 不含 alpha（`stroke.rs:67-69`）。
- **多边形**：`polygon_d` 只发 `M` / `L` / `Z` 直线段，坐标保留 2 位小数（`lib.rs:493-503`）；填充发 `fill="{hex}" fill-opacity="{f.opacity}"`（`lib.rs:473-478`）——**`f.color.a` 被丢弃**（`f.color.hex()`，`lib.rs:476`），所以 `C_LIP.with_alpha(0.55)`（`anime/lib.rs:1458`）在 SVG 里变成不透明色。
- **描边路径**：`stroke_path_d`（`lib.rs:506-533`）把折线转三次 Bézier，控制点：

```rust
// crates/buncraft-render/src/lib.rs:525-526
        let c1 = p1.add(p2.sub(p0).mul(1.0 / 6.0));
        let c2 = p2.sub(p3.sub(p1).mul(1.0 / 6.0));
```

即均匀 Catmull-Rom→Bézier（`1/6` 张力，`lib.rs:516-531`）。导出属性只有 `stroke`, `stroke-opacity={brush.opacity}`, `stroke-width={brush.size}`, `linecap/linejoin=round`（`lib.rs:461-467`）。**压力、taper、jitter、grain、逐 stamp alpha、多点 airbrush 的径向衰减全部丢失**（多点 airbrush 也被当普通路径，`lib.rs:458-467`）。
- **渐变**：只有「单点 airbrush dab」被特判成 `<radialGradient>` + `<circle>`（`lib.rs:440-457`、`lib.rs:448-451`）；`<defs>` 在所有 `<g>` 之后写出（`lib.rs:484-488`）。**没有 `<linearGradient>`、没有 `color-interpolation`/`color-profile`**。
- **clip 全部丢失**：`draw_action`/`draw_fill` 的光栅分支会读 `s.clip` / `f.clip`（`lib.rs:113-119`、`lib.rs:149-173`），但 `to_svg` 的 `Action::Stroke` / `Action::Fill` 两个分支（`lib.rs:438-480`）**完全没有引用 `clip`**（`rg -n clip crates/buncraft-render/src/lib.rs` 只有 `113,117,149,162,188,189,193,194,656,666,673`）。SVG 与 PNG 不是同一幅画：`clipped_stroke_stays_inside_mask` 只测 PNG（`lib.rs:655-680`），没有 SVG 对应测试。
- 图层用 `<g opacity="{layer.opacity:.3}">`（`lib.rs:430-433`），没有混合模式属性。

### 3.4 抗锯齿 / 伽马 / 色彩管理 / 图层混合模式

| 能力 | 结论 | 证据 |
|---|---|---|
| 抗锯齿 | **部分**：tiny-skia 路径填充开了 `anti_alias`（`lib.rs:154,178,358,392,200`），手写圆用 1px 线性 `cov`（`lib.rs:337`）；**无超采样/MSAA/可配置滤波器/解析面积覆盖**；图层合成 `FilterQuality::Nearest`（`lib.rs:102`） | 同上 |
| 伽马校正 | **未找到**。`rg -n -i 'gamma|srgb|linear.?light' crates/`（6 crate）无匹配；`to_color` 直接 `from_rgba8`（`lib.rs:58-61`），`mix` 直接对 `u8` 插值（`stroke.rs:91-100`），`circle` 直接对字节做算术（`lib.rs:326-347`） | `research/physics_math_science/notes/verified_formulas.md:74-78` 也点名此三处 |
| 色彩管理 | **未找到**：`Rgba` 无色域/白点/ICC 字段（`stroke.rs:50-56`），`Canvas.background` 也是 `Rgba`（`document.rs:20`），`to_svg` 只写 hex（`lib.rs:418,464,476`），无 `color-profile`/ICC chunk | 同上 |
| 真实图层混合模式 | **未找到**：只有 `BlendMode::SourceOver`（`lib.rs:101`），clip 合成用 `PixmapPaint::default()`（`lib.rs:124`、`lib.rs:169`，默认同样是 SourceOver）；`Layer` 只有 `opacity`（`document.rs:33-39`），无 `blend_mode` 字段。Multiply / Screen / Overlay / 线性光混合一律**未找到** | `lib.rs:99-104`、`document.rs:33-39` |

---

## 4. 光照与阴影：`crates/buncraft-anime/src/lib.rs` 全是经验几何

### 4.1 `stage_shading`（`lib.rs:1513-1637`）

```rust
// crates/buncraft-anime/src/lib.rs:1519-1531
        let neck_shadow = self.air(58.0, C_SKIN_SHADOW, 0.34);
        for i in 0..5 {
            let t = i as f32 / 4.0;
            let x = cx + (t * 2.0 - 1.0) * 46.0;
            self.curve(
                &[Vec2::new(x, chin + 20.0)],
                neck_shadow.clone(),
                LAYER_SHADOW,
                Stage::Shading,
                "neck shadow",
                0.0,
            );
        }
```

其余同类：额头 7 个点 `air(44.0, Rgba::rgb(214,190,196), 0.30)`，位置 `x = cx + (2t−1)·half_w·0.74`、`y = brow_y − 26 + |2t−1|·10`（`lib.rs:1549-1562`）；腮红 `c = (cx ± half_w·0.52, eye_y + 62)`、`air(46.0, C_BLUSH, 0.16)`（`lib.rs:1566-1574`）；眼睑 `y = eye.center.y − eye_h·1.1`、`pencil(1.4, Rgba::rgb(206,186,196), 0.45)`（`lib.rs:1591-1598`）；头发内侧 `mix(Rgba::rgb(40,36,54), 0.45)` 后 `air(62.0, ..., 0.13)`（`lib.rs:1607-1616`）；刘海下 `air(52.0, Rgba::rgb(226,206,208), 0.10)`（`lib.rs:1626-1636`）。**所有位置都是相对 landmark 的固定像素偏移，颜色都是常量或固定比例混色，不透明度是写死的小数。**

### 4.2 `highlight_band`（`lib.rs:734-765`）

```rust
// crates/buncraft-anime/src/lib.rs:738-760
    fn highlight_band(&self, offset: f32, thickness: f32) -> Vec<Vec2> {
        let (cx, top, h, w) = (self.geo.cx, self.geo.top, self.geo.head_h, self.geo.half_w);
        let steps = 41;
        let mid_y = top + h * 0.19 + offset;
        let sag = h * 0.085;
        let mut upper = Vec::with_capacity(steps);
        let mut lower = Vec::with_capacity(steps);
        for i in 0..steps {
            let t = i as f32 / (steps - 1) as f32 * 2.0 - 1.0;
            let x = cx + t * w * 0.76;
            let y = mid_y + t * t * sag;
            // fades out at both ends, so the band dissolves into the side
            // locks instead of ending in a blunt cut
            let taper = (1.0 - t.abs()).powf(0.38);
            // teeth of uneven length: a regular sawtooth reads as a zipper,
            // an irregular one as torn light
            let jitter = ((i * 7 + 3) % 5) as f32 / 4.0;
            // fine, shallow serration: one tooth every few pixels is what the
            // edge of a real highlight looks like, deep bites look torn
            let tooth = 0.74 + 0.26 * jitter;
            let thick = h * 0.105 * taper * tooth * thickness;
            upper.push(Vec2::new(x, y));
            lower.push(Vec2::new(x, y + thick));
```

`41` 步、抛物线 `y = mid_y + t²·sag`、幂律 taper `(1−|t|)^0.38`、伪随机锯齿 `((i*7+3)%5)/4`（`lib.rs:740-758`），最后 `lower.reverse()` 拼成闭合多边形（`lib.rs:762-764`）。注释明确说这是「动漫头发的光带形状」（`lib.rs:734-737`）。调用处 `lib.rs:1481-1490`，并裁到头发轮廓 `hair_clip`（`lib.rs:1481`）。

### 4.3 `side_strands`（`lib.rs:699-732`）

```rust
// crates/buncraft-anime/src/lib.rs:719-730
        let mut out = Vec::new();
        for k in 0..4 {
            let frac = 0.62 + 0.11 * k as f32;
            let y0 = y_top0 + span * 0.03 * k as f32;
            let y1 = y0 + span * (0.72 + 0.08 * k as f32);
            out.push(vec![
                at(y0, frac),
                at(y0 + (y1 - y0) * 0.40, frac + 0.04),
                at(y0 + (y1 - y0) * 0.78, frac + 0.01),
                at(y1, frac - 0.03),
            ]);
        }
```

4 条静态曲线（`lib.rs:720`），横向位置由 `hair_half_width_at(y)·frac` 线性给出（`lib.rs:713-718`，`geometry.rs:191-216`）。没有悬链线、没有重力、没有弯曲刚度、没有约束求解。

### 4.4 iris shade / glow / catchlight（`lib.rs:1406-1450`、`lib.rs:1492-1508`）

```rust
// crates/buncraft-anime/src/lib.rs:1409-1432
            let shade = ellipse(
                Vec2::new(eye.iris_center.x, eye.iris_center.y - eye.iris_ry * 0.66),
                eye.iris_rx * 0.97,
                eye.iris_ry * 0.64,
                20,
            );
            self.fill_clipped(
                shade,
                self.p.eyes.color.mix(Rgba::rgb(26, 20, 38), 0.46),
                ...
            );
            let glow = ellipse(
                Vec2::new(eye.iris_center.x, eye.iris_center.y + eye.iris_ry * 0.58),
                eye.iris_rx * 0.74,
                eye.iris_ry * 0.32,
                18,
            );
            self.fill_clipped(
                glow,
                self.p.eyes.color.mix(Rgba::rgb(255, 255, 255), 0.52),
                ...
            );
```

注释直言是经验美术规则：「Anime irises are darker under the lid and lit from below」（`lib.rs:1406-1408`）。眼神光是固定白椭圆 `hi = (iris_center.x − side·0.42·iris_rx, iris_center.y − 0.38·iris_ry)`、半径 `8.5/9.5`（`lib.rs:1495-1502`），与光源方向无关。

### 4.5 缺失的物理量（逐函数对照）

| 函数 | 缺的法线 `N` | 缺的光源方向 `L` / 位置 | 缺的辐照度 `E` | 缺的 BRDF / 镜面项 | 缺的遮挡 / 可见性 `V` | 证据 |
|---|---|---|---|---|---|---|
| `stage_shading` | 未找到：无表面法线、无 `Vec3` | 未找到：无光源向量/光位/强度/光谱 | 未找到：无 `max(0,N·L)`、无 `1/r²` 衰减 | 未找到：无漫反射/镜面反射模型 | 未找到：阴影点只是 `chin+20`、`brow_y−26`、`eye_y+62` 等固定偏移，无射线/阴影图/遮挡查询 | `lib.rs:1513-1637`、`lib.rs:1519`、`lib.rs:1549-1553`、`lib.rs:1566`、`lib.rs:1591`、`lib.rs:1609`、`lib.rs:1627` |
| `highlight_band` | 未找到 | 未找到 | 未找到 | 未找到：无 `N·H`、无 Fresnel、无 GGX/粗糙度/能量守恒 | 未找到：仅靠 `hair_clip` 裁到轮廓（`lib.rs:1481-1489`），不是可见性计算 | `lib.rs:738-765`、`lib.rs:1470-1482` |
| `side_strands` | 未找到 | 不适用 | 未找到：无重力/张力/质量 | 不适用 | 未找到：无自碰撞/与脸的碰撞 | `lib.rs:699-732`、`geometry.rs:191-216` |
| iris shade / glow | 未找到：无角膜/虹膜法线 | 未找到：`0.66`/`0.58` 是固定比例（`lib.rs:1410`、`lib.rs:1424`） | 未找到 | 未找到：无折射、无角膜反射、无高光模型 | 未找到：上眼睑遮挡只用一个暗椭圆近似（`lib.rs:1409-1422`） | `lib.rs:1406-1450`、`lib.rs:1492-1508` |
| 全局 | 全仓 `rg -n -i 'brdf|irradiance|light.?dir|light.?source|occlusion|normal| shading normal'` 在 6 个 crate 中**未找到**（`normal` 仅见 `dataset.rs:78`、`rng.rs:40`） | 同上 | 同上 | 同上 | 同上 | `docs/limitations.md:12-15`、`README.md:43` |

---

## 5. 几何 / 解剖：`crates/buncraft-anime/src/geometry.rs`

文件头自述这是「新手漫画教程的比例规则」（`geometry.rs:1-6`：「eye line at ~57% of the head, nose at ~73%, mouth at ~85%」）。

### 5.1 `Geometry::new` 全部坐标公式与魔法系数（`geometry.rs:47-78`）

```rust
// crates/buncraft-anime/src/geometry.rs:47-77
    pub fn new(p: &CharacterProfile) -> Self {
        let cx = 600.0;
        let top = 322.0;
        let head_h = p.face.head_height;
        let chin = top + head_h;
        let half_w = p.face.head_width * 0.5;
        let eye_y = top + head_h * 0.575;
        let eye_dx = half_w * 0.495;
        let eye_w = 62.0 * p.face.eye_size * p.eyes.size;
        let eye_h = 34.0 * p.face.eye_size * p.eyes.size;
        let hair_bottom = chin + (p.hair.length - 0.8) * 760.0;
        Self {
            cx,
            top,
            chin,
            half_w,
            head_h,
            jaw: p.face.jaw_softness.clamp(0.0, 1.0),
            eye_y,
            eye_dx,
            eye_w,
            eye_h,
            brow_y: eye_y - 58.0,
            nose_y: top + head_h * 0.735,
            mouth_y: top + head_h * 0.845,
            neck_y: chin + 120.0,
            shoulder_y: chin + 168.0,
            shoulder_half: 252.0,
            bottom: 1600.0,
            hair_bottom,
        }
    }
```

| 行 | 公式 | 魔法系数 |
|---|---|---|
| `geometry.rs:48` | `cx = 600.0` | 600.0 |
| `geometry.rs:49` | `top = 322.0` | 322.0 |
| `geometry.rs:50-51` | `head_h = p.face.head_height`；`chin = top + head_h` | 默认值来自 `profile.rs:100-103`（372.0 / 470.0 / 0.55 / 1.0） |
| `geometry.rs:52` | `half_w = head_width · 0.5` | 0.5 |
| `geometry.rs:53` | `eye_y = top + head_h · 0.575` | 0.575 |
| `geometry.rs:54` | `eye_dx = half_w · 0.495` | 0.495 |
| `geometry.rs:55-56` | `eye_w = 62.0·…`；`eye_h = 34.0·…` | 62.0 / 34.0 |
| `geometry.rs:57` | `hair_bottom = chin + (length − 0.8)·760.0` | 0.8 / 760.0 |
| `geometry.rs:64` | `jaw = jaw_softness.clamp(0,1)` | 1.0 |
| `geometry.rs:69` | `brow_y = eye_y − 58.0` | 58.0 |
| `geometry.rs:70-71` | `nose_y = top + head_h·0.735`；`mouth_y = top + head_h·0.845` | 0.735 / 0.845 |
| `geometry.rs:72-75` | `neck_y = chin+120`；`shoulder_y = chin+168`；`shoulder_half = 252.0`；`bottom = 1600.0` | 120 / 168 / 252 / 1600 |

### 5.2 `face_polygon` / `face_right` / `face_outline`（`geometry.rs:80-121`）

- `face_right`（`geometry.rs:81-101`）：`jaw_x = w·(0.60 + 0.14·j)`（`86`）、`chin_w = w·(0.10 + 0.06·j)`（`87`）；11 个控制点的比例/偏移：`0/0`（`90`）、`0.46/0.012`（`91`）、`0.84/0.10`（`92`）、`0.99/0.28`（`93`）、`1.0/0.46`（`94`）、`0.975/0.60`（`95`）、`0.90/0.71`（`96`）、`jaw_x/0.855`（`97`）、`0.36/0.955`（`98`）、`chin_w/(chin−4.0)`（`99`）、`0/(chin)`（`100`）。
- `face_outline`（`geometry.rs:106-117`）：用 `2·cx − p.x` 手工镜像（`114`），跳过离中轴 `< 0.5` 的点（`111`）；即**强制左右对称**。
- `face_polygon`（`geometry.rs:119-121`）：`catmull_rom_closed(&self.face_outline(), 10)`（`120`）——只是对控制点做均匀 CR 折线化（`geom.rs:146-165`），不是真 Bézier 路径、无面积/布尔/曲率运算。
- 其余：`cheek_y = top + head_h·0.60`（`geometry.rs:123-125`）；`ear_center = (cx + side·half_w·0.985, eye_y + head_h·0.045)`（`127-132`）；`neck_polygon` 的 `nw = half_w·0.37`、`ny = chin + head_h·0.20`、倍数 `1.12 / 2.4 / 0.35` 与偏移 `20 / 8 / 6`（`134-147`）。

### 5.3 `hair_silhouette`（`geometry.rs:150-183`）

- `flare` 按发型写死：`ShortBob 1.06` / `MessyShort 1.02` / `TwinTails 1.04` / `Ponytail 1.05` / 其它 `1.20`（`geometry.rs:156-162`）。
- 右半 10 点（`geometry.rs:163-174`）：`-26/-20`（`164-165`）、`0.55/0.03`（`166`）、`flare/0.24`（`167`）、`(flare+0.02)/0.48`（`168`）、`flare/(chin−30)`（`169`）、`(flare−0.10)/(bottom·0.55 + (chin−30)·0.45)`（`170`）、`(flare−0.22)/bottom`（`171`）、`0.45/(bottom+26)`（`172`）、`0/(bottom+10)`（`173`）。
- 同样用 `2·cx − q.x` 镜像、跳过 `<0.5`（`geometry.rs:176-181`）。`hair_half_width_at`（`geometry.rs:191-216`）只是把右半点按 y 排序后做**线性插值**（`204-213`），并 `.max(1.0)`（`212`）——不是曲线求交。

### 5.4 `bangs_polygon`（`geometry.rs:218-242`）

```rust
// crates/buncraft-anime/src/geometry.rs:223-240
        let part = cx + p.hair.parting * 70.0;
        let mut pts = vec![
            Vec2::new(cx - w * 0.96, self.top + h * 0.20),
            Vec2::new(cx - w * 0.80, self.top + h * 0.06),
            Vec2::new(cx - w * 0.40, self.top - 6.0),
            Vec2::new(part, self.top - 14.0),
            Vec2::new(cx + w * 0.45, self.top - 4.0),
            Vec2::new(cx + w * 0.84, self.top + h * 0.08),
            Vec2::new(cx + w * 0.97, self.top + h * 0.21),
        ];
        // zig-zag fringe tips, right to left
        let tips = 7;
        for i in 0..tips {
            let t = i as f32 / (tips - 1) as f32;
            let x = cx + w * 0.95 - t * w * 1.9;
            let y = self.brow_y - 20.0 + ((i % 3) as f32) * 9.0;
            pts.push(Vec2::new(x, y));
        }
```

系数：`parting·70.0`（`223`）；`0.96/0.20, 0.80/0.06, 0.40/−6, −14, 0.45/−4, 0.84/0.08, 0.97/0.21`（`225-231`）；`tips = 7`（`234`）；`x` 跨度 `0.95 → −0.95` 乘 `w·1.9`（`237`）；`y = brow_y − 20 + (i%3)·9`（`238`）。锯齿由 `i%3` 产生，不是任何物理/几何模型。

### 5.5 `eye`（`geometry.rs:244-309`）

| 行 | 公式 / 系数 |
|---|---|
| `geometry.rs:245-248` | `ex = cx + side·eye_dx`，`ey = eye_y`，`ew/eh` 来自 §5.1 |
| `geometry.rs:254-263` | `lid_lift`：`Round 1.05` / `Almond 0.92` / `Sharp 0.86` / `Droopy 0.95`；`outer_drop`：`Sharp 0.10` / `Droopy −0.12` |
| `geometry.rs:266-277` | 表情：`Surprised eh·1.12, lid_lift·1.08`；`Smile eh·0.94`；`Shy eh·0.9` |
| `geometry.rs:279-280` | `inner = (ex − side·ew, ey + 4.0)`；`outer = (ex + side·ew, ey − 2.0 − outer_drop·eh)` |
| `geometry.rs:281-287` | 上睑 5 点：`0.58/0.80`、`0.10/1.02`、`0.66/0.72` 乘 `ew/eh·lid_lift` |
| `geometry.rs:288-294` | 下睑 5 点：`0.58/0.48`、`0/0.60`、`−0.60/0.44` 乘 `ew/eh` |
| `geometry.rs:295-296` | `polygon = upper + lower[1..len−2]`（拼接，非多边形裁剪/布尔） |
| `geometry.rs:302-305` | `center = (ex, ey)`；`iris_center = (ex + side·2.0, ey + eh·0.12)`；`iris_rx = ew·0.40`；`iris_ry = eh·1.05` |

补充：`body_outline_right`（`geometry.rs:312-326`）系数 `0.40/(chin+30)`、`0.42/(sh−100)`、`0.62/(sh−55)`、`half·0.62/(sh−18)`、`half·0.92/(sh+18)`、`half/(sh+78)`、`half·1.03/(sh+220)`、`half·1.02/bottom`；`outfit_polygon`（`geometry.rs:328-347`）系数 `1.06/1.03/0.90/0.45`、`78.0`、`sh+64`。`DrawingPlan`（`geometry.rs:350-376`）只记录 `face_width`、`face_height`、`eye_centers`、`chin`、`mouth_y`、`outline`，全是 2D landmark。

### 5.6 没有透视 / 3D / 姿态参数 / 3-4 视角

- **没有透视投影**：全文件只有 `Vec2`（`geometry.rs:8`），没有 `Vec3`、没有 `Matrix`/`Perspective`/`Camera`、没有 `z`、没有除以 `w` 的齐次坐标、没有 `Transform3`（`rg -n 'Vec3|Matrix|Perspective|camera|project' crates/buncraft-anime/` **未找到**）。画布固定 1200×1600（`anime/lib.rs:52`），坐标就是像素（`docs/drawing-model.md:5-6`）。
- **没有 3D**：所有形状都是 2D 折线/多边形；`ellipse`（`geom.rs:264-272`）是 2D 参数椭圆；`face_outline`（`geometry.rs:106-117`）、`hair_silhouette`（`geometry.rs:175-182`）都靠 `2·cx − x` 镜像生成另一半。
- **没有姿态参数**：`CharacterProfile` 字段只有 `name/seed/gender/skin/hair/eyes/outfit/expression/face/notes`（`profile.rs:109-122`），`HairSpec` 只有 `style/color/length/parting`（`profile.rs:58-66`），`FaceSpec` 只有 `head_width/head_height/jaw_softness/eye_size`（`profile.rs:85-95`）——**没有 yaw/pitch/roll、没有骨骼/关节、没有相机参数**。`Geometry::new(p)` 只吃 `&CharacterProfile`（`geometry.rs:47`）。
- **没有 3/4 视角**：`side` 恒为 `±1.0`（`anime/lib.rs:303`、`lib.rs:349`、`lib.rs:388`、`lib.rs:425-426`、`lib.rs:809`、`lib.rs:1128` 等），左右两眼永远等距且镜像。`README.md:43` "Pose variation, full body, hands | not implemented"；`docs/limitations.md:12-13` "Front-facing bust only. / Hands, full body, perspective."；`brain/src/lib.rs:177` "Pose variation is limited to the bust portrait camera used by the anime generator."；`docs/roadmap.md:13-14` 把「Add `Pose` to `CharacterProfile` (head yaw/pitch, shoulder tilt) ... Unlocks 3/4 and profile views」列为**下一步待办**。

---

## 6. 物理：全项目「无任何物理模拟」

**明确结论：在任务书指定的 6 个 crate 中，无任何物理模拟。**

反证检索（在 `crates/` 下，`rg -n -i`）：

```
\b(gravity|damping|collision|collide|integrator|integrate|velocity|acceleration|friction|
   spring|constraint|rigid|physics|simulation|impulse|torque|inertia|solver|force)\b
→ 未找到（仅 `crates/buncraft-science/Cargo.toml:9` 的包描述里出现 "physics"，
          `crates/buncraft-science/src/curve.rs:36` 注释出现 "integrate |C'(t)|"，
          二者属于范围外的并发新建 crate，见 §0 边界说明）
```

逐项核实：

| 物理量 / 机制 | 结论 | 证据 |
|---|---|---|
| 积分器（显式/隐式 Euler、Verlet、RK4、substep） | **未找到** | 全仓无 `dt`/`step` 物理循环；唯一「时间步」是浏览器播放器的 `dt = now − last`（`crates/buncraft-cli/src/player.html:224`、`player.html:227`），它只推进播放头 `t += dt·speed`，不更新任何状态变量 |
| 约束求解（PBD/XPBD、Lagrange 乘子、Jacobian） | **未找到** | `rg` 无匹配；`docs/roadmap.md:14-15` 只说要加 pose，未提约束 |
| 质量 / 重力 / 阻尼 | **未找到** | 唯一的 "mass" 是美术术语「hair mass」（`anime/lib.rs:257,269,699,821,1069,1098,1155,1292,1468,1517,1606,1625`），不是物理量；`Rng::gauss`（`core/src/rng.rs:41-45`）是 Box–Muller 正态随机数，用于手抖（`anime/lib.rs:115-116`），不是力学 |
| 碰撞 / 接触 / 摩擦 / 恢复系数 | **未找到** | `rg` 无匹配；`hair_half_width_at`（`geometry.rs:191-216`）只是防止头发画到脸上的 2D 宽度查表，不是碰撞检测 |
| 速度 / 加速度 / 力 / 冲量 / 扭矩 / 惯量 | **未找到** | `BrushKind::base_speed()`（`stroke.rs:28-37`）是「绘图速度」常量（px/s），仅用于算时间轴长度（`document.rs:381`），不是物体速度 |
| 唯一的运动学式子 | `duration = length / (base_speed × stage_factor × speed)` | `document.rs:377-383`；这是匀速 `t = s/v`，无质量、无力、无加速度、无状态演化 |
| 神经网络 ≠ 物理 | `SketchRnn::step`（`brain/src/model.rs:144-190`）是 GRU 前向传播；且**无训练权重** | `brain/src/model.rs:7-10` 明写 "No trained weights are bundled"；`README.md:42`；`docs/limitations.md:8-9` |

---

## 7. 时间线与动画：是「回放」，不是「仿真」

### 7.1 `Stage` 与静态流水线

`Stage` 是 13 个枚举值（`stroke.rs:180-196`），`label`（`stroke.rs:198-215`）、`index`（`stroke.rs:217-234`）只是查表。`Artist::run` 按固定顺序调用 12 个 stage 方法（`anime/lib.rs:64-82`），`Stage::Correction` 只由 `apply_corrections` 在评估超差时追加（`anime/lib.rs:1643-1671`、`brain/src/lib.rs:98-102`）。**没有关键帧、没有插值、没有状态机**；`docs/limitations.md:14-15` 明写 "MP4/GIF, keyframes and in-betweens do not [exist]"。

### 7.2 `start_ms` / `duration_ms` / `render(up_to_ms)`

```rust
// crates/buncraft-core/src/document.rs:146-167
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Timing {
    pub start_ms: f64,
    pub duration_ms: f64,
}

impl Timing {
    pub fn end_ms(&self) -> f64 {
        self.start_ms + self.duration_ms
    }

    /// 0..1 progress at time `t`.
    pub fn progress(&self, t: f64) -> f32 {
        if t <= self.start_ms {
            0.0
        } else if t >= self.end_ms() {
            1.0
        } else {
            ((t - self.start_ms) / self.duration_ms.max(0.001)) as f32
        }
    }
}
```

```rust
// crates/buncraft-core/src/document.rs:367-392
    pub fn schedule(&mut self, speed: f64) {
        let speed = speed.max(0.05);
        let mut t = 0.0f64;
        let mut last_stage: Option<Stage> = None;
        self.timings.clear();
        for action in &self.actions {
            if last_stage != Some(action.stage()) {
                t += 260.0;
                last_stage = Some(action.stage());
            }
            let dur = match action {
                Action::Fill(_) => 420.0,
                Action::Stroke(s) => {
                    let factor = stage_speed_factor(s.stage);
                    let px_per_s = (s.brush.kind.base_speed() as f64) * factor * speed;
                    ((s.length() as f64 / px_per_s) * 1000.0).clamp(45.0, 5000.0) + 35.0
                }
            };
            self.timings.push(Timing {
                start_ms: t,
                duration_ms: dur,
            });
            t += dur + 32.0;
        }
        self.duration_ms = t;
    }
```

- `schedule` 是**离线一次性排程**：时间轴顺序、互不重叠（换 stage 加 `260.0`ms，`document.rs:374`；每个 action 后加 `32.0`ms，`document.rs:389`），`Fill` 固定 `420.0`ms（`document.rs:378`），Stroke 用 `length/速度` 夹到 `[45,5000]+35`ms（`document.rs:382`）。`stage_speed_factor` 是一张 13 项常量表（`document.rs:452-468`），`base_speed` 是 5 项常量表（`stroke.rs:28-37`）。
- `render(up_to_ms)`（`render/src/lib.rs:64-107`）：`opts.up_to_ms`（`render/src/lib.rs:24`、`lib.rs:50-55`）→ `timing.progress(t)`（`render/src/lib.rs:85-89`）→ `progress <= 0.0` 跳过（`lib.rs:90-92`）→ `draw_action(..., progress, ...)`（`lib.rs:93`）。
- **stroke 显隐按「点序号」而非弧长**：`visible = (n−1)·progress`（`render/src/lib.rs:213`），`whole/frac`（`214-215`），`segment()` 在两点间按 `(k/steps)·frac` 线性插值（`render/src/lib.rs:242-249`）。所以「画到一半」= 画到**第 n/2 个采样点**，不等于画到一半弧长（§1.3、§1.4 已说明采样本非等弧长）。
- **fill 是透明度淡入，不是注释说的 wipe**：`draw_fill` 用 `to_color(f.color, f.opacity * progress)`（`render/src/lib.rs:153`、`lib.rs:177`），而 `lib.rs:139` 注释写 "Fills are revealed as a wipe so replay still looks like painting" —— 实现与注释不符。
- 浏览器播放器同源：`progressOf` 线性 `(now−s)/d`（`crates/buncraft-cli/src/player.html:141-147`）、`vis = (pts.length−1)*p`（`player.html:152`）、`fillStyle = rgba(a.color, a.opacity*p)`（`player.html:183`）、`t += dt*speed`（`player.html:227`）。API 侧播放器为同一文件副本（`crates/buncraft-api/src/player.html:263` 等）。
- CLI `--frames N`（`cli/src/main.rs:41-43`）只是对 N 个等距时刻调用 `render(doc, RenderOptions::at_time(t))` 存 PNG（`cli/src/main.rs:153-163`），没有帧间状态传递。

**结论**：`up_to_ms` 是一个**预录时间轴的查询参数**，`progress(t)` 是分段线性显隐系数。系统里不存在 `x(t+Δt) = ∫v dt`、`v(t+Δt) = v + (F/m)Δt` 之类的状态演化，也没有任何对象属性随时间被积分。因此它是**回放（replay）而非仿真（simulation）**；`docs/architecture.md:37-38` 也只说 "Timing is derived"，`docs/drawing-model.md:57-58` 称其 "frame-accurate"，但从 §1.3 的采样误差看，它连「按弧长精确」都做不到。

---

## 8. 差距对照表（16 行）

> 「对应上游仓库」列引用 `research/physics_math_science/repo_manifest.md:10-20` 的已克隆仓库；「验收标准」列均为可写成 `#[test]` 的量化判据。

| # | 能力 | 现状（证据） | 需要的真实科学 | 对应上游仓库 | 建议落在哪个 crate 的哪个文件 | 验收标准（可测量的测试） |
|---|---|---|---|---|---|---|
| 1 | 曲线弧长参数化 | `catmull_rom` 均匀 `t` 采样（`core/src/geom.rs:124-143`、`geom.rs:137`），`resample` 按折线弦长（`geom.rs:185-213`），椭圆按角度（`geom.rs:264-271`）；无 `s(t)`/`t(s)` | 自适应 Gauss–Legendre 求 `∫|C'(t)|dt` 与反函数 `inv_arclen`（`notes/verified_formulas.md:80-98`） | `linebender/kurbo`（`repo_manifest.md:12`）、`nical/lyon`（`:13`） | `crates/buncraft-core/src/geom.rs` 新增 `arc_length_param()`；`stroke.rs:306-308` 改调用 | `rx=100,ry=25` 椭圆 64 采样，相邻弧长 `max/min < 1.02`；与 kurbo `gauss_arclen` 相对误差 `< 0.5%` |
| 2 | 曲率 / 切线 / 法线 / 叉积 | 全**未找到**（§1.6）；仅 `perp`（`geom.rs:55-57`）无调用点，`dot` 仅 2 处（`geom.rs:305,309`） | 一阶/二阶导 `C'(t),C''(t)`、`κ=|x'y''−y'x''|/(x'²+y'²)^{3/2}`、3D 叉积与法线矩阵 | `dimforge/nalgebra`（`repo_manifest.md:10`）、`linebender/kurbo`（`:12`） | 新建 `crates/buncraft-core/src/curve.rs`；`geom.rs:18` 增加 re-export | 半径 `R` 圆上 100 点 `κ·R` 误差 `< 1e-3`；`tangent·normal < 1e-6`；绕向符号与叉积一致 |
| 3 | 路径布尔 / 描边轮廓 | **未找到**（`docs/limitations.md:17`、`research/research_gaps.md:42-43`）；`clip` 只是掩膜（`render/src/lib.rs:188-202`） | 并/交/差、非零/奇偶填充规则、tessellation 与 stroke outline | `nical/lyon`（`repo_manifest.md:13`）、`dimforge/parry`（`:18`） | `crates/buncraft-render/src/lib.rs` 的 `polygon_mask`/`draw_fill`（`188-202`,`135-186`）+ `anime/src/geometry.rs:119-121` | 两重叠多边形 union 面积 = `A+B−I`，误差 `< 0.1%`；发丝与脸轮廓求交后无越界像素 |
| 4 | 精确包围盒 / OBB | 只有采样点 AABB（`geom.rs:69-117`、`stroke.rs:282-288`） | 曲线控制点精确包围盒、OBB、凸包、包围球 | `dimforge/nalgebra`（`:10`）、`dimforge/parry`（`:18`） | `crates/buncraft-core/src/geom.rs` 的 `Bounds`（`69-117`） | 旋转 45° 矩形 OBB 面积 = 真值 ±0.1%；AABB 必须完全包含所有采样点（1000 随机点 0 漏） |
| 5 | 线性光颜色混合 | `mix` 对 `u8` 编码值插值（`stroke.rs:91-100`）；5 处调用（`anime/lib.rs:1417,1431,1477,1479,1607`） | sRGB EOTF 解码 → 线性光插值 → OETF 编码（`notes/verified_formulas.md:61-78`）、Oklab 保色相 | `Ogeon/palette`（`repo_manifest.md:15`）、`colour-science/colour`（`:19`） | `crates/buncraft-core/src/stroke.rs:91-100` 旁新增 `mix_linear()`（保留旧函数） | 黑→白 `t=0.5` 解码后线性亮度 = `0.5 ± 1e-3`；与 `palette::LinSrgb` 逐通道差 `≤ 1/255`；ΔE_ok 单调性测试 |
| 6 | 真实亮度 / 光度学 | `luminance` 是 Rec.601 luma on γ 字节（`stroke.rs:82-89`），唯一用于 `anime/lib.rs:1470` 阈值 `0.72`（`lib.rs:1476`） | 线性化后 Rec.709 `0.2126/0.7152/0.0722`、CIE XYZ / 照度、CIEDE2000 | `Ogeon/palette`（`:15`）、`colour-science/colour`（`:19`） | `crates/buncraft-core/src/stroke.rs:87-89` 旁新增 `luminance_linear()` | sRGB 中灰 `127` 解码后 `Y ≈ 0.2158 ± 1e-3`；1000 组随机色与 colour-science 参考值 ΔE2000 `< 0.5` |
| 7 | 伽马校正 / 色彩管理 | **未找到**（§3.4）；`to_color` 直接 `from_rgba8`（`render/src/lib.rs:58-61`），`circle` 对字节做算术（`lib.rs:343-347`）；SVG 只写 hex（`lib.rs:418,464,476`） | IEC 61966-2-1 sRGB EOTF/OETF、ICC profile、白点/色域标记 | `Ogeon/palette`（`:15`）、`colour-science/colour`（`:19`） | `crates/buncraft-render/src/lib.rs:58-61` 的 `to_color`；`lib.rs:408-491` 的 `to_svg` | 1000 个 `x∈[0,1]` 的 `encode(decode(x))` 往返误差 `< 1/255`；PNG 含 `sRGB` chunk、SVG 含 `color-interpolation` |
| 8 | 预乘 alpha / 图层混合模式 | 只有 `SourceOver`（`render/src/lib.rs:101`）+ `FilterQuality::Nearest`（`lib.rs:102`）；`Layer` 无 `blend_mode`（`document.rs:33-39`）；`color.a` 在 `circle()` 被忽略（`lib.rs:326`）而 `to_color` 会乘（`lib.rs:59`）→ `r=26` 处不连续（`lib.rs:312-315`） | 线性光预乘 SourceOver、Multiply/Screen/Overlay、能量守恒合成 | `Ogeon/palette`（`:15`）、`tiny-skia`（`Cargo.toml:26`） | `crates/buncraft-render/src/lib.rs:99-104`（图层）、`lib.rs:308-350`（圆）、`core/src/document.rs:33-39`（`Layer` 加字段） | 100 组随机 `Multiply/Screen/Overlay` 与参考值差 `≤ 1/255`；`a=128` 时 `r=25` 与 `r=27` 两路径像素差 `≤ 1/255` |
| 9 | 抗锯齿 | 1px 线性 `cov`（`render/src/lib.rs:337`）+ 路径 `anti_alias=true`（`lib.rs:154,178,358,392`）；无 MSAA/超采样 | 解析面积覆盖、4×/8× 超采样、滤波重建 | `nical/lyon`（`:13`）、`linebender/kurbo`（`:12`） | `crates/buncraft-render/src/lib.rs:308-350` 的 `circle` | `r=3.5` 圆盘覆盖像素总和与 `πr²` 相对误差 `< 1%`；4×SSAA 金图逐像素差 `< 1/255` |
| 10 | SVG 保真（clip/渐变/压力） | clip 完全丢失（`lib.rs:438-480` 无 `clip`）、压力/taper/jitter 丢失（`lib.rs:461-467`）、`fill` 丢 `color.a`（`lib.rs:476` + `stroke.rs:67-69`） | 真 `BezPath`、`clipPath`、渐变对象、笔压轮廓 | `linebender/kurbo`（`:12`）、`nical/lyon`（`:13`） | `crates/buncraft-render/src/lib.rs:408-533` 的 `to_svg`/`polygon_d`/`stroke_path_d` | 含 N 个 clip 的文档导出 SVG 恰有 N 个 `clipPath`；`with_alpha(0.55)` 的 fill 导出 `fill-opacity` 含 `0.55`；PNG↔SVG 渲染 RMS `< 2/255` |
| 11 | 光照：法线 / 光源 / 辐照度 | 全**未找到**（§4.5）；阴影是固定像素偏移（`anime/lib.rs:1519-1562`,`1566`,`1591`,`1609`,`1627`） | 表面法线 `N`、光向 `L`、`E = max(0,N·L)·I/r²`、环境光 | `dimforge/nalgebra`（`:10`）、`dimforge/parry`（`:18`） | 新建 `crates/buncraft-anime/src/shading.rs`，由 `lib.rs:1513` 的 `stage_shading` 调用 | Lambert 球渲染亮度与解析 `cosθ` 曲线逐点误差 `< 2%`；改变光向 180° 后亮暗区互换（相关系数 `< −0.9`） |
| 12 | BRDF / 镜面高光 | `highlight_band` 是抛物线+幂律+锯齿（`anime/lib.rs:738-765`），高光色由 `luminance>0.72` 决定（`lib.rs:1476-1482`） | Fresnel / GGX / `N·H`、粗糙度、能量守恒 `∫f_r ≤ 1` | `dimforge/nalgebra`（`:10`）、`dimforge/parry`（`:18`） | `crates/buncraft-anime/src/lib.rs:738-765` 新增 `specular_band()` | 光源方位每转 15°，高光峰位置单调移动且单调性 100% 成立；离散 `∫f_r dω ≤ 1+1e-3` |
| 13 | 遮挡 / 投射阴影 | 只有 `hair_clip` 轮廓裁剪（`anime/lib.rs:1481-1489`），无射线/阴影图 | 可见性 `V`、射线求交、软阴影/半影 | `dimforge/parry`（`:18`）、`dimforge/rapier`（`:17`） | `crates/buncraft-anime/src/lib.rs:1513-1637` | 已知遮挡物投影边界与渲染阴影边界偏差 `< 1px`；遮挡物移除后阴影像素数归零 |
| 14 | 3D / 透视 / 3-4 视角 / 姿态 | 全 2D、严格镜像（`geometry.rs:106-117`,`175-182`），`CharacterProfile` 无 pose 字段（`profile.rs:109-122`），`README.md:43`、`docs/limitations.md:12-13` | 齐次坐标、旋转/透视矩阵、相机、yaw/pitch/roll | `dimforge/nalgebra`（`:10`）、`sarah-quinones/faer-rs`（`:11`）、`rust-ndarray/ndarray`（`:14`） | `crates/buncraft-core/src/profile.rs:109-122` 加 `#[serde(default)] Pose`；`crates/buncraft-anime/src/geometry.rs:47-78` 加 `project()` | `yaw=0` 时几何与旧版逐点差 `< 1e-4`；`yaw=30°` 时眼距比 = `cos30° ± 1%`；投影后近大远小比例误差 `< 1%` |
| 15 | 头发 / 布料动力学 | `side_strands` 4 条静态曲线（`anime/lib.rs:719-730`），`hair_half_width_at` 只是 y 线性查表（`geometry.rs:191-216`） | 弹性杆 / XPBD 距离与弯曲约束、质量、重力、阻尼、子步（`notes/verified_formulas.md:9-38`） | `avianphysics/avian`（`:16`）、`dimforge/rapier`（`:17`）、`PositionBasedDynamics`（`:20`） | 新建 `crates/buncraft-anime/src/dynamics.rs`，由 `lib.rs:703` / `lib.rs:1159-1167` 选用 | 单摆 `g=9.81, L=1` 的周期与 `2π√(L/g)` 误差 `< 2%`；10 s 内总能量漂移 `< 1%`；`Δt` 减半结果收敛（误差比 `< 0.3`） |
| 16 | 积分器 / 约束 / 碰撞 / 时间线物理化 | **无任何物理模拟**（§6）；`Timing::progress` 只是线性显隐（`document.rs:158-166`），`render` 按点序号显隐（`render/src/lib.rs:213`） | 半隐式 Euler / Verlet / XPBD 子步、接触求解、固定步长确定性 | `avianphysics/avian`（`:16`）、`dimforge/rapier`（`:17`）、`dimforge/parry`（`:18`） | 新建 `crates/buncraft-core/src/dynamics.rs` + `document.rs:146-167` 的 `Timing` 增加可选 `state` | 自由落体 `x(t)=½gt²` 在 `t=1s` 误差 `< 0.1%`；恢复系数 `e` 由反弹高度反算误差 `< 2%`；固定 seed 跑 600 步 ×2 次结果 bit-identical；`cargo test --workspace` 原有 49 测试零回归 |

---

## 9. 最小可行真科学改造：4 步顺序

> 原则：**每一步都是一次独立提交**，只「加」不「删」；保留现有公开 API 与默认输出（同一 `(prompt, seed, speed, correct)` 的 `document.to_json()` 必须逐字节不变，`buncraft-anime/src/lib.rs:1703-1708` 的 `is_deterministic` 已有此断言）；每步结束都执行 `cargo test --workspace` 且必须全绿（基线 49 通过，§0）。**不做一次性大重构。**
> 依赖按 `research/physics_math_science/repo_manifest.md:10-20` 的已克隆版本逐个加入 `[workspace.dependencies]`（`Cargo.toml:19-32`），一次只加一个。

### 第 1 步：把颜色运算搬进线性光（引入 `palette`）

- **落点**：`crates/buncraft-core/Cargo.toml:9-12` 加 `palette`；`crates/buncraft-core/src/stroke.rs` **新增** `mix_linear()` / `luminance_linear()` / `to_lin_srgb()`，**不删** `mix`（`91-100`）/`luminance`（`87-89`）/`with_alpha`（`71-76`）。
- **依据**：`stroke.rs:91-100` 现在对 γ 编码 `u8` 插值；真公式在 `research/physics_math_science/notes/verified_formulas.md:61-78`（sRGB EOTF/OETF）。
- **新增可测测试**（`stroke.rs:398-438` 的 `mod tests` 内）：① `srgb_decode(0.5) ≈ 0.21404114 ± 1e-6`；② 黑→白 `t=0.5` 的线性亮度 `=0.5 ± 1e-3`（旧 `mix` 给 ≈0.214，测试要显式记录这个差值）；③ 1000 组 `encode(decode(x))` 往返 `≤ 1/255`；④ `mix_linear` 与 `palette::LinSrgb` 逐通道 `≤ 1/255`。
- **独立验收**：`cargo test -p buncraft-core` 与 `cargo test --workspace` 全绿；`stroke.rs:402-408`、`stroke.rs:411-414`、`stroke.rs:433-437` 旧测试不改动即通过；`README.md:163-172` 声称的确定性不受影响。

### 第 2 步：渲染端线性光合成 + 修掉 alpha 不一致（仍用 `palette`，不动几何）

- **落点**：`crates/buncraft-render/Cargo.toml:9-12` 加 `palette`；改 `crates/buncraft-render/src/lib.rs:58-61` 的 `to_color`（解码→合成→编码边界），并在 `lib.rs:308-350` 的 `circle()` 里把 `cr/cg/cb`（`lib.rs:326`）改成同样经过线性化/或至少乘上 `color.a/255`，消除 `r=26`（`lib.rs:312-315`）分界处的跳变；`soft_circle` 的 3 个停靠点（`lib.rs:382-384`）改在线性空间插值。
- **新增可测测试**（`lib.rs:594-680` 的 `mod tests` 内）：① 构造 `brush.color.a=128` 的两种半径（`r=25` 与 `r=27`）圆，中心像素差 `≤ 1/255`；② 线性光 SourceOver 与 gamma 空间 SourceOver 在 `a=0.5` 时结果不同的断言（记录差值，证明改造生效）；③ `to_svg` 对 `with_alpha` 的 fill 导出含正确 alpha（`lib.rs:473-478` 顺带修）。
- **独立验收**：`render/src/lib.rs:614-680` 的 5 个原测试（`svg_keeps_single_point_airbrush_dabs`、`renders_ink_pixels`、`replay_starts_blank`、`svg_contains_path`、`clipped_stroke_stays_inside_mask`）全部保持通过；`cargo test --workspace` 全绿。

### 第 3 步：真几何——等弧长 + 曲率/切线/法线（引入 `kurbo`，随后 `lyon`）

- **落点**：`crates/buncraft-core/Cargo.toml` 加 `kurbo`；在 `crates/buncraft-core/src/geom.rs` **新增** `arc_length_param()`、`inv_arc_length()`、`curvature()`、`tangent()`、`normal()`（新增 API，不改 `catmull_rom`/`resample`/`ellipse`/`mirror_x` 的签名，`geom.rs:124-291` 原样保留）；`crates/buncraft-core/src/lib.rs:18` 增加 re-export。随后在同一步内加 `lyon` 到 `crates/buncraft-render/Cargo.toml:9-12`，为 `polygon_mask`（`render/src/lib.rs:188-202`）补一个 `path_boolean_union()` 的可选实现，默认仍走旧掩膜。
- **依据**：`notes/verified_formulas.md:80-98`（kurbo 自适应 Gauss–Legendre 弧长与 `inv_arclen`）。
- **新增可测测试**：① `rx=100, ry=25` 椭圆 64 点重采样后相邻弧长 `max/min < 1.02`（对照 `geom.rs:264-271` 现在 ≈4×）；② 半径 `R=50` 圆上 100 点 `κ·R` 误差 `< 1e-3`；③ `t·n < 1e-6`（正交）；④ union 面积误差 `< 0.1%`。
- **独立验收**：`geom.rs:321-353` 的 4 个原测试 + `stroke.rs:416-430` 的 `stroke_length_matches_polyline`（要求 `100±1`）全部通过；`cargo test --workspace` 全绿。

### 第 4 步：最小物理内核——一个可验证的积分器（引入 `avian` 或 `rapier`，feature 隔离）

- **落点**：**新增** `crates/buncraft-core/src/dynamics.rs`（或 `crates/buncraft-anime/src/dynamics.rs`），以 `#[cfg(feature = "physics")]` 隔离，默认 **off**；只实现「质点 + 重力 + 线性阻尼 + 半隐式 Euler + 固定子步」与一个距离约束（对照 `notes/verified_formulas.md:9-38` 的 XPBD compliance 公式），**不接入** `Artist::run`（`anime/lib.rs:64-82`），因此现有出图逐字节不变。依赖 `avian`（`repo_manifest.md:16`）或 `rapier`（`:17`）只放在该 feature 下。
- **新增可测测试**：① 自由落体 `g=9.81` 在 `t=1 s` 时 `x = 4.905 m ± 0.1%`；② 单摆 `L=1` 周期与 `2π√(L/g)` 误差 `< 2%`；③ `Δt=16.67ms` 与 `8.33ms` 结果收敛（误差比 `< 0.3`）；④ 固定 seed 跑 600 步两次 bit-identical；⑤ `feature=off` 时 `cargo test --workspace` 与基线 49 测试完全一致。
- **独立验收**：`cargo test --workspace`（默认 feature）与 `cargo test --workspace --features physics` **都**全绿；`brain/tests/end_to_end.rs:65-70` 的 `same_seed_same_art`、`:8-62` 的 `draws_renders_and_reloads` 不回归。

**这 4 步之后才谈得上**把 `nalgebra` 的姿态/透视（`§8 #14`）、`rapier`/`parry` 的遮挡与布料（`§8 #13,15`）接进 `stage_shading`（`anime/lib.rs:1513`）与 `geometry.rs:47-78`；任何一步都不要求同时改 6 个 crate，也不要求替换 `Rgba`/`Stage`/`Document` 的数据模型。

---

## 附录 A：本次使用的主要反证检索（可复现）

| 检索 | 结果 |
|---|---|
| `rg -n -i '\b(gravity\|damping\|collision\|integrator\|velocity\|acceleration\|force\|constraint\|rigid\|physics\|simulation\|impulse\|torque\|inertia\|solver)\b' crates/` | 6 个 crate 内**未找到**（§6） |
| `rg -n -i 'curvature\|tangent\|srgb\|gamma\|linear.?light\|brdf\|irradiance\|occlusion' crates/`（限 6 crate） | **未找到**（§1.6、§3.4、§4.5） |
| `rg -n 'mix\(' crates/buncraft-anime/` | 5 处：`lib.rs:1417,1431,1477,1479,1607`（§2.2） |
| `rg -n 'Rgba::rgb\|Rgba::new' crates/buncraft-anime/` | 20 处，全表列于 §2.2 |
| `rg -n '\.perp\(\|\.rot\(' crates/`（限 6 crate） | **未找到调用点**（§1.1） |
| `rg -n 'clip' crates/buncraft-render/src/lib.rs` | 仅 `113,117,149,162,188,189,193,194,656,666,673`；`to_svg`（`408-491`）无 clip（§3.3） |
| `cargo test --workspace` | 11 组 `test result: ok`，非空合计 49 passed / 0 failed（§0） |

## 附录 B：一句话总结

BunCraft 的「数学」是**均匀参数 Catmull-Rom + 折线弦长重采样 + 角度采样椭圆 + 竖直镜像**（`core/src/geom.rs`），「色彩」是**γ 编码 u8 上的线性插值与 Rec.601 luma**（`core/src/stroke.rs`），「光照」是**固定像素偏移的化妆式阴影**（`anime/src/lib.rs:1513-1637`），「动画」是**预录时间轴的线性显隐**（`core/src/document.rs:146-167`、`render/src/lib.rs:85-93`），「物理」是**零**（§6）。全流程确定性、结构清晰、测试完整，但所有「看起来像物理/光学」的效果都是经验美术规则；引入 `nalgebra`/`kurbo`/`lyon`/`palette`/`avian`/`rapier` 时应按 §9 的 4 步逐个替换，并以 §8 的量化测试作为验收闸门。
