# LIVE_STEEL_EXECUTION.md

## ДЕФЕКТ

### Верификация исходного ответа

- Предыдущий диагноз `particle -> seat` подтверждён исходником.
- `src/lib.rs` держит `seats: Vec<f32>` как главный носитель формы.
- `src/lib.rs` вызывает `seat_at()` внутри каждого `sim_step()`.
- `src/lib.rs` назначает частице конкретный `seat` по индексу `i`.
- `src/lib.rs` ведёт частицу к `sx, sy` прямым steering-force.
- `src/lib.rs` фиксирует посадку через `set[i]`.
- `src/lib.rs` использует `birth` как псевдо-фронт чтения.
- `template.html` генерирует `sampleSeats(text)` из пикселей glyph canvas.
- `template.html` режет glyph pixels до `TARGET=2800` discrete seats.
- `template.html` сортирует seats по `x`, создавая UI-порядок, а не физику.
- `template.html` рисует через `globalCompositeOperation='lighter'`.
- `template.html` выводит бело-синий свечущий металл, а не тёмное тело стали.
- `OUT_STRIDE=6` недостаточен для живого материала.
- Текущий output несёт только `x, y, angle, set, wake, edge`.
- Текущий output не несёт `length, radius, field, chain, density, glint, occlusion`.
- Proof frames показывают читаемое слово.
- Proof frames не показывают настоящую причинность магнитного поля.
- Proof frames показывают даши, припаркованные в glyph-mask.
- Proof frames дают ощущение цифровой расчески.
- Proof frames дают glow-typography, а не стальную стружку.
- Сборка в этой среде не исполнена из-за отсутствующего `cargo`.
- Архитектура проверена по исходникам и кадрам.
- Предыдущий план верен по направлению.
- Предыдущий план недостаточен как исполнительный документ.

### Главный дефект старой модели

- Частица знает адрес буквы.
- Живая стружка адресов не знает.
- Частица должна знать только поле, соседей, инерцию, трение, магнитный момент.
- Буква должна быть энергетическим ландшафтом.
- Слово должно возникать как след магнитизации.
- Форма не должна включаться пиксельной телепортацией.

### Вторичный дефект старой модели

- `SDF only` притягивает к контуру, а не гарантирует плотный stroke fill.
- `SDF tangent only` даёт слишком чистую математику.
- `seat removal only` ломает читаемость.
- Нужна density-field коррекция без индивидуальных targets.
- Нужна dipole-chain физика без explicit seat ownership.
- Нужен renderer тела, а не renderer свечения.

### Что запрещено

- Запрещено оставлять `seat_at()` в основном force loop.
- Запрещено оставлять `birth` как источник появления буквы.
- Запрещено рисовать весь металл через `lighter`.
- Запрещено делать все filings одинаковой длины.
- Запрещено делать все filings одинаковой толщины.
- Запрещено выравнивать filings только по glyph sample angle.
- Запрещено превращать field в скрытый список targets.
- Запрещено дорисовывать текст canvas glyph поверх particles.
- Запрещено спавнить particles внутри букв.
- Запрещено удалять particles из heap.
- Запрещено принимать кадр, если он выглядит как шрифт из белых палочек.

---

## ИДЕАЛ

### Нужный percept

- Зритель видит тёмную кучку металлических filings.
- Кучка реагирует до контакта со словом.
- Filings слышат невидимый магнитный рельеф.
- Filings вытягиваются в цепочки.
- Цепочки дрожат, ломаются, снова сцепляются.
- Слово собирается не как UI, а как намагниченная материя.
- Буквы плотные, но края рваные.
- Внутри strokes есть масса, не только контур.
- Вокруг букв есть слабые stray filings.
- Свет блестит на отдельных гранях, а не заливает весь объект.
- Никакой синтетической белой вывески.
- Никакой cyan sci-fi пыли.
- Никакого ощущения `particles lerp into mask`.

### Физическая иллюзия

- Форма возникает из трёх полей.
- Первое поле удерживает material inside glyph mass.
- Второе поле ориентирует filings вдоль stroke tangent.
- Третье поле распределяет density без адресов.
- Wand активирует область, а не назначает particles.
- Dipoles создают цепочки, а не renderer изображает цепочки.
- Renderer показывает массу, occlusion, scratches, glints.

### Визуальная гарантия через gates

- Гарантия не является обещанием вкуса без просмотра кадра.
- Гарантия здесь означает запрет выхода без прохождения измеримых gates.
- Любая сборка, похожая на glow text, считается failed build.
- Любая сборка с прямым particle-seat mapping считается failed build.
- Любая сборка без field-before-form считается failed build.
- Любая сборка без видимых chains считается failed build.
- Любая сборка без тёмного metal body считается failed build.

---

## МОДЕЛЬ

### Новый source of truth

```text
glyph alpha
-> signed distance field
-> stroke tangent tensor
-> target density field
-> wand activation memory
-> particle dipoles
-> neighbor chain forces
-> dark steel renderer
```

### Контракт данных

- JS больше не передаёт seats.
- JS передаёт alpha-grid glyph field.
- Rust строит SDF, potential, tangent, target density.
- Rust симулирует particles через поля и соседей.
- JS только рисует output particles.
- Canvas glyph никогда не рисуется в итоговый кадр.

### Полевые массивы Rust

```rust
const MAX_PHRASES: usize = 4;
const FIELD_MAX_W: usize = 320;
const FIELD_MAX_H: usize = 180;
const FIELD_MAX: usize = FIELD_MAX_W * FIELD_MAX_H;
const FIELD_META_STRIDE: usize = 8;
const OUT_STRIDE: usize = 11;
```

- `field_alpha[phrase][cell]` хранит anti-aliased glyph occupancy.
- `field_sdf[phrase][cell]` хранит signed distance in screen pixels.
- `field_potential[phrase][cell]` хранит blurred alpha attraction.
- `field_tx[phrase][cell]` хранит stroke tangent x.
- `field_ty[phrase][cell]` хранит stroke tangent y.
- `field_target_rho[phrase][cell]` хранит desired active density.
- `field_bounds[phrase]` хранит glyph bounds.
- `activation[cell]` хранит magnetic memory текущей phrase.
- `rho[cell]` хранит текущую particle density.
- `rho_err[cell]` хранит target-minus-current density.
- `rho_force_x[cell]` хранит gradient density correction.
- `rho_force_y[cell]` хранит gradient density correction.

### Particle state

```rust
x, y
vx, vy
ang, omega
len, rad
mass
moment
rough
shine
phase_noise
polarity
heat
field_lock
chain
active
```

- `len` варьируется в диапазоне `4.2..11.5 px at DPR=1`.
- `rad` варьируется в диапазоне `0.45..1.35 px at DPR=1`.
- `mass` коррелирует с `len * rad`.
- `moment` коррелирует с `len`, но имеет roughness.
- `rough` управляет грязью и микрошумом.
- `shine` управляет редкими specular glints.
- `polarity` используется только для endpoint dipole math.
- `chain` не задаётся renderer-ом напрямую.
- `chain` является следствием neighbor interactions.

### Output state

```rust
out[o + 0] = x
out[o + 1] = y
out[o + 2] = angle
out[o + 3] = len
out[o + 4] = rad
out[o + 5] = active
out[o + 6] = field_strength
out[o + 7] = chain_strength
out[o + 8] = density_pressure
out[o + 9] = glint
out[o + 10] = depth
```

- `set` удаляется из visual contract.
- `wake` заменяется на `active`.
- `edge` заменяется на `field_strength + chain_strength + glint`.
- Renderer больше не знает phase typography.

### Field sampling

- Все field samples bilinear.
- Все screen-to-grid conversions проходят через bounds-aware mapping.
- Field coordinates не зависят от particle index.
- Field samples clamp to grid border.
- Distance values хранятся в screen pixels.
- Forces не должны зависеть от DPR напрямую.

### SDF semantics

- `sdf > 0` означает outside glyph.
- `sdf = 0` означает glyph boundary.
- `sdf < 0` означает inside glyph.
- `abs(sdf) < 36` означает magnetic influence band.
- `sdf > 96` означает no letter pull.
- `sdf < -2` означает captured inside stroke mass.

### Potential semantics

- `potential = gaussian_blur(alpha, sigma=2.2 cells)`.
- `potential` тянет к glyph mass.
- `potential` не задаёт индивидуальные positions.
- `potential` спасает читаемость без seats.
- `potential` работает слабее, чем chains near final state.

### Target density semantics

- `target_rho = normalize(alpha^0.72) * N_active`.
- `target_rho` распределяет массу по strokes.
- `target_rho` не создаёт адресов.
- `target_rho` живёт на grid.
- `rho` строится каждый frame из particle deposit.
- `rho_err = activation * target_rho - blur(rho)`.
- `rho_force = grad(blur(max(rho_err, 0)))`.
- `rho_force` тянет particles в пустоты буквы.
- `rho_force` ограничивается `MAX_DENSITY_FORCE`.
- `rho_force` выключается в unactivated cells.

### Tangent field semantics

- `grad_sdf = central_diff(sdf)`.
- `normal = normalize(grad_sdf)`.
- `t_sdf = perp(normal)`.
- `structure_tensor` строится из smoothed alpha gradient.
- `t_tensor` берётся как eigenvector with lower eigenvalue.
- `tangent = normalize(lerp(t_sdf, t_tensor, tensor_confidence))`.
- `tensor_confidence` защищает medial axis от flip noise.
- `tangent` разрешает `t` и `-t` как один physical axis.
- Torque использует axis alignment, не arrow alignment.

### Wand semantics

- Wand это activation source, не target source.
- Wand движется по glyph bounds слева направо.
- Wand имеет elliptical magnetic lobe.
- Wand оставляет residual magnetization.
- Activation растёт быстро.
- Activation затухает медленно.
- Activation никогда не телепортирует particles.

### Wand formula

```text
wand_x = lerp(min_x - 0.08 * span_x, max_x + 0.10 * span_x, front)
wand_y = center_y + sin(time * 1.37) * 0.035 * screen_h
rx = 0.115 * screen_w
ry = 0.190 * screen_h
q = ((cell_x - wand_x) / rx)^2 + ((cell_y - wand_y) / ry)^2
pulse = exp(-q * 1.85)
letter_gate = exp(-abs(sdf) / 54)
new_activation = pulse * letter_gate
activation = max(activation * 0.992, new_activation)
```

- `front` is phase scalar.
- `front` activates fields.
- `front` never indexes particles.
- `birth` is deleted.

### Force stack

```text
F_total = F_heap_idle
        + F_magnetic_capture
        + F_density_void
        + F_tangent_flow
        + F_dipole_neighbors
        + F_core_repulsion
        + F_boundary
        + F_friction
```

- `F_heap_idle` exists before activation.
- `F_magnetic_capture` pulls toward glyph mass.
- `F_density_void` fills strokes without seats.
- `F_tangent_flow` makes filings slide along strokes.
- `F_dipole_neighbors` creates chains.
- `F_core_repulsion` prevents clumps.
- `F_boundary` keeps particles on canvas.
- `F_friction` kills synthetic ease-in-out.

### Magnetic capture

```text
outside = smoothstep(96, 2, sdf)
inside = smoothstep(-30, -2, sdf)
band = exp(-abs(sdf) / 42)
F_capture = grad(potential) * 0.72 * activation
F_contain = -normal * max(sdf, 0) * 0.010 * activation * outside
F_inside_soft = normal * clamp((-sdf - 18) / 42, 0, 1) * 0.035 * activation
```

- `F_capture` gets particles to the glyph.
- `F_contain` prevents outside halos from becoming the word.
- `F_inside_soft` prevents center-line overcompression.
- Constants are starting points, not decoration knobs.

### Tangent flow

```text
m = vec2(cos(ang), sin(ang))
t = sample_tangent(x, y)
axis_dot = abs(dot(m, t))
torque_field = sign(cross(m, t)) * (1 - axis_dot)^2 * K_TORQUE * activation
F_flow = t * signed_noise_axis(i, time) * 0.12 * activation * band
```

- Torque aligns rods to stroke axis.
- Flow adds tiny living creep along strokes.
- Flow must be weaker than capture.
- Flow must be stronger than idle noise inside activated glyph.

### Dipole endpoints

```text
m = vec2(cos(ang), sin(ang))
head = p + m * len * 0.5
tail = p - m * len * 0.5
```

- Endpoint forces are calculated in neighbor pass.
- Opposite poles attract.
- Same poles repel.
- Center force moves the particle.
- Endpoint torque rotates the particle.
- Force is softened and clipped.

### Dipole pair rules

```text
for endpoint a in [-1, +1]
for endpoint b in [-1, +1]
polarity = a * b
r = endpoint_b - endpoint_a
d2 = dot(r, r) + SOFTEN
f_mag = -polarity * K_DIPOLE / d2
f_mag *= smoothstep(CHAIN_R, CHAIN_R * 0.35, sqrt(d2))
f = normalize(r) * clamp(f_mag, -F_PAIR_MAX, F_PAIR_MAX)
```

- `polarity = -1` attracts.
- `polarity = +1` repels.
- Endpoint force contributes to center force.
- Endpoint force contributes to angular torque.
- Pair force is skipped when both particles inactive and far from wand.

### Anti-clump rules

```text
core = rad_i + rad_j + 1.15
if d < core:
  push = normalize(p_i - p_j) * (core - d) * K_CORE
```

- Core repulsion always wins over dipole attraction.
- Side repulsion wins over head-tail attraction when lateral distance is too small.
- Angular damping rises with local density.
- Max velocity is hard-clamped.
- Max omega is hard-clamped.

### Chain strength metric

```text
parallel = abs(dot(m_i, m_j))
axial = abs(dot(normalize(p_j - p_i), m_i))
near = smoothstep(CHAIN_R, 3.0, d)
chain_pair = parallel * axial * near
```

- `chain[i]` is EMA of strongest neighbor chain_pair.
- `chain[i]` drives renderer glints lightly.
- `chain[i]` never creates geometry by itself.

### Integrator

- Use semi-implicit Euler.
- Use dt clamp already present.
- Use `sd = dt / 16.67` only as legacy compatibility.
- Prefer physical scale constants in px/frame.
- Velocity damping depends on material state.
- Idle particles have higher friction.
- Active particles have lower friction.
- Dense particles have higher angular damping.

### Renderer model

- Steel is mostly dark.
- Brightness is rare.
- White only appears as tiny specular cuts.
- Body pass uses `source-over`.
- Shadow pass uses `source-over`.
- Glint pass may use `lighter`, but only for tiny segments.
- No full-line glow.
- No blue body color.
- No global additive typography.

### Renderer passes

```text
1. background persistence
2. contact shadow behind active chains
3. dark steel body rods
4. oxidized edge scratches
5. thin silver glints
6. sparse dust motes outside glyph
7. debug HUD
```

- Pass 1 uses slow fade, not glow trail.
- Pass 2 gives mass.
- Pass 3 carries the material.
- Pass 4 breaks synthetic uniformity.
- Pass 5 sells metal.
- Pass 6 sells physical residue.
- Pass 7 is disabled in final capture.

### Body color range

```text
r = 38..92
g = 43..104
b = 49..116
alpha = 0.34..0.76
```

- Body never reaches pure white.
- Body never reaches cyan.
- Active area is denser, not simply brighter.
- Density darkens before it brightens.

### Glint rule

```text
light = normalize(vec2(-0.35, -0.94))
axis = vec2(cos(angle), sin(angle))
spec_axis = pow(abs(dot(axis, perp(light))), 10)
spec_motion = smoothstep(0.1, 2.8, speed)
spec = spec_axis * (0.35 + 0.65 * shine) * (0.45 + 0.55 * chain)
```

- Glint length is `0.22..0.48` of body length.
- Glint alpha is clipped below `0.11`.
- Glint pass draws only when `spec > 0.38`.
- Glint position jitters per particle by stable hash.

### Texture rule

- Each rod gets stable asymmetry.
- Each rod gets 1 or 2 micro-notches.
- Long rods bend visually by drawing two nearly aligned segments.
- Dirt is stable per particle.
- Dirt must not shimmer frame-to-frame.
- Noise lives in material, not in final typography mask.

---

## РЕАЛИЗАЦИЯ

### Commit 0 — Baseline lock

- Copy ZIP into repo root.
- Commit current `mf_life_wasm` unchanged.
- Save proof frames as baseline artifacts.
- Record current frame time from HUD.
- Mark current effect as `baseline_seat_glow`.
- Do not tune old physics further.

### Commit 1 — Delete seat authority without breaking host

- Keep `seats_ptr()` temporarily for compatibility only.
- Add `field_alpha_ptr()`.
- Add `field_meta_ptr()`.
- Add `sim_rebuild_field(phrase: u32)`.
- Add `debug_field_ptr(kind: u32, phrase: u32)`.
- Add Rust arrays for fields.
- Leave old draw running until field debug works.
- No physics change in this commit.

### Commit 1 patch contract

```rust
#[no_mangle]
pub extern "C" fn field_alpha_ptr() -> *mut f32;

#[no_mangle]
pub extern "C" fn field_meta_ptr() -> *mut f32;

#[no_mangle]
pub extern "C" fn sim_rebuild_field(phrase: u32);

#[no_mangle]
pub extern "C" fn debug_field_ptr(kind: u32, phrase: u32) -> *const f32;
```

- `field_meta[0] = field_w`.
- `field_meta[1] = field_h`.
- `field_meta[2] = min_x`.
- `field_meta[3] = max_x`.
- `field_meta[4] = min_y`.
- `field_meta[5] = max_y`.
- `field_meta[6] = screen_w`.
- `field_meta[7] = screen_h`.

### Commit 2 — JS alpha field instead of seats

- Replace `sampleSeats(text)` with `sampleGlyphField(text)`.
- Render text to offscreen canvas as before.
- Downsample alpha into field grid.
- Compute bounds from alpha threshold.
- Write alpha into WASM `field_alpha_ptr()`.
- Write meta into WASM `field_meta_ptr()`.
- Call `wasm.sim_rebuild_field(pi)` after each phrase write.
- Keep `PHRASES` unchanged.
- Keep `TARGET=2800` unchanged.

### Commit 2 alpha rules

```text
FIELD_W = clamp(round(W / 5), 192, 320)
FIELD_H = clamp(round(H / 5), 108, 180)
alpha_cell = mean(canvas_alpha over covered pixels) / 255
alpha_cell = pow(alpha_cell, 0.85)
inside = alpha_cell > 0.16
```

- Use averaged alpha, not single pixel sample.
- Keep antialiasing.
- Do not threshold away soft edge.
- Bounds expand by `64 px` in screen space.

### Commit 3 — Field builder

- Build signed distance field from alpha.
- Use exact EDT if time allows.
- Use 8-neighbor chamfer only as fallback.
- Build outside distance to inside cells.
- Build inside distance to outside cells.
- Set `sdf = outside - inside`.
- Convert cell distance to screen px.
- Clamp SDF to `[-128, 192]`.

### Commit 3 EDT requirement

- Exact EDT is preferred because it prevents anisotropic grid diamonds.
- Chamfer fallback must use weights `1.0` and `1.4142`.
- Chamfer fallback must run forward and backward passes twice.
- SDF artifacts visible in debug are build-blocking.

### Commit 4 — Tangent and potential

- Build `potential = blur(alpha, sigma=2.2 cells)`.
- Build `grad_potential` by central diff.
- Build `grad_sdf` by central diff.
- Build `t_sdf = perp(normalize(grad_sdf))`.
- Build structure tensor from blurred alpha gradients.
- Blend `t_sdf` and `t_tensor`.
- Store final `tx, ty`.

### Commit 4 tensor rule

```text
Jxx = blur(gx * gx)
Jxy = blur(gx * gy)
Jyy = blur(gy * gy)
tensor_tangent = eigenvector_min(J)
confidence = clamp((Jxx + Jyy) * 12, 0, 1)
tangent = axis_blend(t_sdf, tensor_tangent, confidence)
```

- `axis_blend` must flip tangent sign before interpolation when dot is negative.
- Tangent debug must show continuous strokes.
- Tangent debug must not boil at medial axes.

### Commit 5 — Density field

- Add per-frame particle density grid.
- Clear `rho` each `sim_step`.
- Deposit every particle into `rho` with bilinear weights.
- Blur `rho` one pass horizontal and vertical.
- Build active target density from `target_rho * activation`.
- Build `rho_err = active_target - rho`.
- Clamp negative `rho_err` to zero for attraction.
- Build `rho_force` from gradient of blurred positive error.
- Sample `rho_force` in particle loop.

### Commit 5 normalization

```text
sum_alpha = sum(alpha^0.72)
target_rho[cell] = alpha[cell]^0.72 * N / max(sum_alpha, 1)
```

- Target density uses count conservation.
- No particle receives an individual destination.
- Empty glyph cells exert no density pull.
- Activated empty spaces inside letters stay empty because alpha is low.

### Commit 6 — Replace seat steering

- Remove `seat_at()` from `sim_step()`.
- Remove `birth` from force logic.
- Remove direct `sx, sy` steering.
- Remove arrival logic from `set[i]`.
- Replace `set/wake/edge` with `active/field/chain`.
- Use field sample at particle position.
- Use capture force, density force, tangent torque.
- Keep heap idle only before activation.

### Commit 6 particle loop skeleton

```rust
sample = sample_field(active_phrase, x[i], y[i]);
active[i] += (sample.activation - active[i]) * (0.18 * sd).min(1.0);
force += sample.grad_potential * K_CAPTURE * active[i];
force += sample.rho_force * K_DENSITY * active[i];
force += sample.contain_force * active[i];
torque += axis_torque(ang[i], sample.tangent) * K_FIELD_TORQUE * active[i];
force += sample.tangent * flow_noise(i, phase_s) * K_TANGENT_FLOW * active[i];
```

- `K_CAPTURE = 0.70` initial.
- `K_DENSITY = 0.34` initial.
- `K_FIELD_TORQUE = 0.23` initial.
- `K_TANGENT_FLOW = 0.10` initial.
- Force clipping happens after all forces.

### Commit 7 — Wand activation grid

- Add `activation: Vec<f32>` in `Sim` for current phrase.
- Update activation once per frame before particle loop.
- Use phrase bounds from field meta.
- Front scalar comes from phase time.
- Activation memory decays, not resets every frame.
- Activation resets on `sim_reset()` and `sim_to()`.
- Morph blends old activation out and new activation in.

### Commit 7 phase behavior

```text
Assemble: wand sweeps current phrase from left to right.
Dwell: activation remains high, particles settle and breathe.
Morph: old activation decays, next phrase wand starts at left.
```

- Morph must not reintroduce seats.
- Morph must not teleport density target.
- Morph can apply weak de-magnetization noise to old phrase.

### Commit 8 — Particle material initialization

- Add `omega`, `len`, `rad`, `mass`, `moment`, `rough`, `shine`, `chain`, `active` vectors.
- Initialize in `sim_init()`.
- Reset positions only in `sim_reset()`.
- Do not reset stable material traits on reset.
- Make distribution skewed, not uniform.

### Commit 8 distributions

```text
len = 4.2 + pow(rand, 0.42) * 7.3
rad = 0.45 + pow(rand, 1.8) * 0.90
mass = 0.65 + len * rad * 0.095
moment = len * (0.72 + rand * 0.46)
rough = rand
shine = pow(rand, 2.4)
omega = (rand - 0.5) * 0.02
```

- Long rods are rarer but visually important.
- High shine is rare.
- Rough rods should dominate.

### Commit 9 — Spatial hash upgrade

- Current hash can stay.
- Cell size becomes `max(CHAIN_R, 16)`.
- Neighbor scan stays 3x3 cells.
- Pair pass computes core, dipole, side, alignment.
- Pair force is accumulated per particle in temporary arrays.
- Pair torque is accumulated per particle in temporary arrays.
- Do not mutate neighbor velocities inside nested loop.
- Apply accumulated forces after pair pass.

### Commit 9 constants

```text
CHAIN_R = 18.0
CORE_PAD = 1.15
K_CORE = 0.42
K_DIPOLE = 8.5
K_SIDE = 0.18
K_ALIGN = 0.055
F_PAIR_MAX = 0.72
TORQUE_PAIR_MAX = 0.065
```

- These constants are first-pass anchors.
- Tuning must use gates, not taste panic.

### Commit 10 — Dipole chain pass

- Build particle hash.
- For each pair within `CHAIN_R`, compute endpoint forces.
- Add short-range core repulsion.
- Add side repulsion when lateral distance is low.
- Add axis alignment torque.
- Add chain metric.
- Clip pair force and pair torque.
- Write `chain[i] = ema(chain[i], chain_metric, 0.18)`.

### Commit 10 side repulsion

```text
axis = normalize(m_i + signed_flip(m_j, m_i))
rij = p_j - p_i
lateral = rij - axis * dot(rij, axis)
side_d = length(lateral)
if side_d < rad_i + rad_j + 2.2:
  push sideways
```

- Side repulsion prevents steel fur from becoming white mud.
- Head-tail attraction is allowed only when axial relation is strong.
- Parallel packed rods must form lanes, not blobs.

### Commit 11 — Integrator and friction

- Integrate velocity after all force accumulation.
- Integrate omega after all torque accumulation.
- Clamp force magnitude.
- Clamp velocity magnitude.
- Clamp omega.
- Damping depends on active state.
- Active rods slide more than idle rods.
- Dense rods damp angular jitter.

### Commit 11 formulas

```text
v += force / mass * sd
omega += torque / mass * sd
v *= lerp(0.875, 0.935, active)
omega *= lerp(0.78, 0.91, active) * (1 - 0.12 * density_pressure)
x += v * sd
y += v * sd
ang += omega * sd
```

- Clamp `|v| <= 7.0 px/frame`.
- Clamp `|omega| <= 0.24 rad/frame`.
- Clamp final positions to canvas with damped bounce, not hard sticky clamp.

### Commit 12 — Renderer contract update

- Set JS `OUT_STRIDE=11`.
- Read `len, rad, active, field, chain, pressure, glint, depth`.
- Remove foreground split based on `set/wake/edge`.
- Sort is optional.
- If sorting is too slow, draw in three buckets by `depth`.
- Body pass first.
- Glint pass last.
- `lighter` only in glint pass.

### Commit 12 draw skeleton

```js
x.globalCompositeOperation = 'source-over';
x.fillStyle = 'rgba(5,7,11,0.31)';
x.fillRect(0,0,W,H);

drawContactShadows(out);
drawSteelBodies(out);
drawScratches(out);
drawSpecularGlints(out);
```

- Background persistence is darker than current.
- Fill alpha is lower to preserve trails subtly.
- No blue HUD in final capture mode.

### Commit 12 body draw

```js
const ca = Math.cos(a), sa = Math.sin(a);
const L = len * DPR;
const R = rad * DPR;
const dark = 38 + rough * 22 + active * 18 + pressure * 10;
const alpha = 0.32 + active * 0.20 + chain * 0.16;
x.strokeStyle = `rgba(${dark},${dark+6},${dark+13},${alpha})`;
x.lineWidth = Math.max(0.65, R * (1.1 + pressure * 0.35));
x.beginPath();
x.moveTo(px - ca*L*.5, py - sa*L*.5);
x.lineTo(px + ca*L*.5, py + sa*L*.5);
x.stroke();
```

- Replace `rough` with stable value derived from index if not output.
- Do not allow body channel above 128.
- Brightness comes from density and glint, not body wash.

### Commit 12 glint draw

```js
if(spec > 0.38){
  x.globalCompositeOperation = 'lighter';
  x.strokeStyle = `rgba(210,220,226,${Math.min(0.42, spec*0.32)})`;
  x.lineWidth = Math.max(0.45, R * 0.42);
  draw shorter centered segment;
}
```

- Glint segment is short.
- Glint count is sparse.
- Glint must not reveal a perfect font mask alone.

### Commit 13 — Debug modes

- Add `window.__debugField = 0`.
- `1` shows alpha.
- `2` shows SDF isolines.
- `3` shows tangent strokes.
- `4` shows activation memory.
- `5` shows target density.
- `6` shows current density.
- `7` shows density error.
- `8` shows chain strength.

### Commit 13 capture tools

```js
window.__pause = () => { running = false; };
window.__play = () => { running = true; last = performance.now(); };
window.__to = idx => { wasm.sim_to(idx|0); };
window.__steps = n => { for(let k=0;k<(n|0);k++){ wasm.sim_step(16.67); draw(); } };
window.__perf = () => ({ N:TARGET, phase, phrase, perf, avgField, avgChain, avgActive });
window.__shot = () => cv.toDataURL('image/png');
```

- Existing helpers stay.
- Perf now exposes field and chain metrics.
- Debug must not ship enabled.

### Commit 14 — Acceptance gates

- Add visual and numeric gates.
- Do not merge until all gates pass.
- Do not tune renderer to hide failed physics.
- Do not tune physics to compensate for glow.

### Gate A — No target ownership

```text
grep -R "seat_at" src/lib.rs -> not used by sim_step
grep -R "birth" src/lib.rs -> not used by force logic
grep -R "set\[" src/lib.rs -> no visual arrival state
```

- Passing means no particle has a private glyph address.
- Failure means synthetic animation still exists.

### Gate B — Material conservation

```text
N_out == TARGET every frame
no spawn during phrase
no delete during phrase
max clamp count < 0.5% per 300 frames
```

- Passing means same material becomes word.
- Failure means magic dust trick.

### Gate C — Field-before-form

```text
At frame 20..45:
particles ahead of visible word must rotate toward tangent field
avg active outside glyph band > idle baseline
word is not fully readable yet
```

- Passing means filings hear the letter before becoming the letter.
- Failure means old mask reveal.

### Gate D — Chain emergence

```text
avgChain inside active glyph >= 0.34
p90 chain length visual >= 3 rods
side clump blobs < 6% of active material
```

- Passing means steel filings, not confetti.
- Failure means particle dust.

### Gate E — Dense strokes

```text
glyph stroke coverage >= 78%
hole preservation >= 90%
edge raggedness visible but bounded
no contour-only collapse
```

- Passing means readable metal mass.
- Failure means outline swarm or soup.

### Gate F — No glow typography

```text
body pass max RGB <= 128
full-line additive draw count == 0
glint additive draw count <= 28% of particles
background blue channel not dominant
```

- Passing means steel.
- Failure means synthetic neon.

### Gate G — Temporal life

```text
settled word still micro-moves
chain break/rejoin visible at low amplitude
no boiling at medial axes
no frame-to-frame random shimmer
```

- Passing means alive.
- Failure means dead mask or noisy screensaver.

### Gate H — Performance

```text
TARGET=2800 interactive
frame time target <= 8 ms on baseline desktop browser
field rebuild only on resize/phrase load
neighbor pass pair count bounded by spatial hash
```

- Passing means prototype remains usable.
- Failure means beautiful corpse.

### Commit 15 — Tuning order

- Tune field first.
- Tune density second.
- Tune dipoles third.
- Tune renderer fourth.
- Never tune renderer before physics gates pass.
- Never tune color before body/glint pass separation.
- Never increase particle count before chains work.

### Tuning ladder

```text
If word unreadable: increase K_DENSITY by 0.04.
If contour-only: reduce F_contain, increase target density blur.
If blobs appear: increase K_CORE and K_SIDE before reducing K_DIPOLE.
If chains invisible: increase K_DIPOLE, then K_ALIGN, not glint.
If shimmer appears: lower per-frame random, increase stable roughness.
If synthetic mask appears: lower K_DENSITY, raise chain/tangent contribution.
If glow appears: lower glint count, darken body, remove additive body paths.
```

### Kill switches

- If `seat_at()` returns to force loop, stop.
- If `lighter` wraps body pass, stop.
- If particles can be matched to glyph pixels by stable index, stop.
- If word is readable before wand reaches it, stop.
- If all rods share length, stop.
- If debug tangent looks cleaner than final steel, add material roughness, not field noise.

### Minimal implementation order that must not change

```text
1. Field memory
2. Alpha host write
3. SDF/potential/tangent debug
4. Density field
5. Seat steering removal
6. Wand activation
7. Particle material traits
8. Dipole chains
9. Renderer body/glint split
10. Gates and tuning
```

- Reordering renderer before physics creates fake beauty.
- Reordering dipoles before field creates clumped dust.
- Reordering density before SDF debug creates invisible bugs.

---

## VERDICT

- Предыдущий ответ подтверждён в главном: `seat-targeting` надо вырезать.
- Предыдущий ответ неполон: одного SDF недостаточно для плотной живой стали.
- Лучший путь: `alpha field + SDF + tangent tensor + density field + wand activation + dipole chains + dark steel renderer`.
- Это не cosmetic refactor.
- Это смена онтологии симуляции.
- Слово больше не является набором парковочных мест.
- Слово становится магнитным ландшафтом.
- Частица больше не является пикселем текста.
- Частица становится стальной стружкой с моментом, массой, трением, соседями и блеском.
- Renderer больше не продаёт glow.
- Renderer продаёт тёмное тело, occlusion и редкие серебряные ножевые glints.
- Проход gates обязателен.
- Без gates результата нет.
- С gates synthetic поделка не проходит ревью.
- Исполнять ровно в указанном порядке.
- Первый merge считается успешным только после `Gate A..H = PASS`.
