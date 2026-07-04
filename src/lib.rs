const MAX_PHRASES: usize = 4;
const MAX_PARTICLES: usize = 4096;
const LETTER_SLOT_BUDGET: usize = 3200;
const FIELD_W: usize = 224;
const FIELD_H: usize = 126;
const FIELD_MAX: usize = FIELD_W * FIELD_H;
const FIELD_META_STRIDE: usize = 8;
const OUT_STRIDE: usize = 14;
const PERF_STRIDE: usize = 16;
const HASH_W: usize = 80;
const HASH_H: usize = 45;
const HASH_MAX: usize = HASH_W * HASH_H;
const DENSITY_CADENCE: u32 = 6;
const PAIR_CADENCE: u32 = 72;
const PAIR_FORCE_DECAY: f32 = 0.78;

const TAU: f32 = core::f32::consts::PI * 2.0;
const EPS: f32 = 0.0001;
const INF: f32 = 1_000_000.0;

const ASSEMBLE: f32 = 3.4;
const DWELL: f32 = 2.4;
const MORPH: f32 = 3.3;

const CHAIN_NEAR: f32 = 7.5;
const CHAIN_FAR: f32 = 18.5;
const PAIR_CENTER_REACH: f32 = CHAIN_FAR;
const ROD_LEN_MIN: f32 = 2.2;
const ROD_LEN_SPAN: f32 = 3.4;
const MAX_ROD_LEN: f32 = ROD_LEN_MIN + ROD_LEN_SPAN + 0.1;
const BIRTH_SLOT_CLEARANCE: f32 = 64.0;
const TYPESET_RAIL_JITTER_RAD: f32 = 0.155;
const PAIR_BROAD_REACH: f32 = CHAIN_FAR + MAX_ROD_LEN;
const SOFTEN: f32 = 14.0;
const K_DIPOLE: f32 = 0.50;
const F_PAIR_MAX: f32 = 0.045;
const T_PAIR_MAX: f32 = 0.090;
const K_CORE: f32 = 0.072;
const K_SIDE: f32 = 0.050;
const SIDE_PAD: f32 = 1.35;
const RHO_DEPOSIT_SCALE: f32 = 0.022;
const PRESSURE_OVERFILL: f32 = 1.30;
const K_RHO_VOID: f32 = 18.0;
const K_RHO_PRESSURE: f32 = 7.0;

const STATE_CHAOS: f32 = 0.0;
const STATE_AGITATED: f32 = 1.0;
const STATE_SEARCH: f32 = 2.0;
const STATE_CLAIMED: f32 = 3.0;
const STATE_CAPTURED: f32 = 4.0;
const STATE_ALIGNING: f32 = 5.0;
const STATE_LOCKED: f32 = 6.0;
const STATE_SETTLED: f32 = 7.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Assemble,
    Dwell,
    Morph,
}

#[derive(Clone, Copy)]
struct FieldSample {
    sdf: f32,
    potential: f32,
    nx: f32,
    ny: f32,
    tx: f32,
    ty: f32,
    target_rho: f32,
    spine: f32,
    activation: f32,
    rho_force_x: f32,
    rho_force_y: f32,
    pressure: f32,
}

struct Sim {
    n: usize,
    phrase_count: usize,
    width: f32,
    height: f32,
    seed0: u32,
    rng: u32,
    cur: usize,
    nxt: usize,
    phase: Phase,
    phase_s: f32,
    total_s: f32,
    tick: u32,
    wand_front: f32,
    wand_y: f32,

    field_alpha: Vec<f32>,
    field_sdf: Vec<f32>,
    field_potential: Vec<f32>,
    field_nx: Vec<f32>,
    field_ny: Vec<f32>,
    field_tx: Vec<f32>,
    field_ty: Vec<f32>,
    field_target_rho: Vec<f32>,
    field_spine: Vec<f32>,
    field_anisotropy: Vec<f32>,
    field_meta: Vec<f32>,
    slot_count: [usize; MAX_PHRASES],
    slot_fx: Vec<f32>,
    slot_fy: Vec<f32>,
    slot_tx: Vec<f32>,
    slot_ty: Vec<f32>,
    slot_priority: Vec<f32>,
    debug_field: Vec<f32>,

    activation: Vec<f32>,
    rho: Vec<f32>,
    desired: Vec<f32>,
    void: Vec<f32>,
    pressure: Vec<f32>,
    rho_force_x: Vec<f32>,
    rho_force_y: Vec<f32>,

    scratch_a: Vec<f32>,
    scratch_b: Vec<f32>,
    scratch_c: Vec<f32>,
    scratch_d: Vec<f32>,
    dist_inside: Vec<f32>,
    dist_outside: Vec<f32>,

    out: Vec<f32>,

    x: Vec<f32>,
    y: Vec<f32>,
    vx: Vec<f32>,
    vy: Vec<f32>,
    ang: Vec<f32>,
    omega: Vec<f32>,
    axis_x: Vec<f32>,
    axis_y: Vec<f32>,
    len: Vec<f32>,
    rad: Vec<f32>,
    mass: Vec<f32>,
    moment: Vec<f32>,
    rough: Vec<f32>,
    shine: Vec<f32>,
    phase_noise: Vec<f32>,
    polarity: Vec<f32>,
    heat: Vec<f32>,
    field_lock: Vec<f32>,
    chain: Vec<f32>,
    active: Vec<f32>,
    depth: Vec<f32>,
    speed: Vec<f32>,
    render_field: Vec<f32>,
    particle_pressure: Vec<f32>,
    heap_x: Vec<f32>,
    heap_y: Vec<f32>,
    capture_s: Vec<f32>,
    trace_len: Vec<f32>,
    rotation_trace: Vec<f32>,
    particle_state: Vec<f32>,

    force_x: Vec<f32>,
    force_y: Vec<f32>,
    torque: Vec<f32>,
    head: Vec<i32>,
    next: Vec<i32>,

    pair_count: u32,
    pair_saturation_count: u32,
    clamp_count: u32,
    glint_count: u32,
}

static mut SIM: Option<Sim> = None;
static mut PERF: [f32; PERF_STRIDE] = [0.0; PERF_STRIDE];

#[inline]
fn clamp01(v: f32) -> f32 {
    v.clamp(0.0, 1.0)
}

#[inline]
fn smooth_up(lo: f32, hi: f32, v: f32) -> f32 {
    debug_assert!(hi > lo);
    let t = ((v - lo) / (hi - lo).max(EPS)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

#[inline]
fn smooth_down(lo: f32, hi: f32, v: f32) -> f32 {
    1.0 - smooth_up(lo, hi, v)
}

#[inline]
fn rng_next(s: &mut u32) -> f32 {
    *s ^= *s << 13;
    *s ^= *s >> 17;
    *s ^= *s << 5;
    *s as f32 / u32::MAX as f32
}

#[inline]
fn hash_u32(mut h: u32) -> f32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^= h >> 16;
    h as f32 / u32::MAX as f32
}

#[inline]
fn typeset_slot_tangent(seed: u32) -> (f32, f32) {
    let angle = (hash_u32(seed) * 2.0 - 1.0) * TYPESET_RAIL_JITTER_RAD;
    (angle.cos(), angle.sin())
}

#[inline]
fn angle_delta(to: f32, from: f32) -> f32 {
    let mut d = (to - from + core::f32::consts::PI) % TAU - core::f32::consts::PI;
    if d < -core::f32::consts::PI {
        d += TAU;
    }
    d
}

#[inline]
fn axis_delta(target: f32, current: f32) -> f32 {
    let mut d = angle_delta(target, current);
    if d > core::f32::consts::FRAC_PI_2 {
        d -= core::f32::consts::PI;
    }
    if d < -core::f32::consts::FRAC_PI_2 {
        d += core::f32::consts::PI;
    }
    d
}

#[inline]
fn cross(ax: f32, ay: f32, bx: f32, by: f32) -> f32 {
    ax * by - ay * bx
}

#[inline]
fn normalize(x: f32, y: f32) -> (f32, f32) {
    let l = (x * x + y * y).sqrt();
    if l > EPS {
        (x / l, y / l)
    } else {
        (1.0, 0.0)
    }
}

#[inline]
fn axis_blend(ax: f32, ay: f32, bx: f32, by: f32, w: f32) -> (f32, f32) {
    let dot = ax * bx + ay * by;
    let sign = if dot < 0.0 { -1.0 } else { 1.0 };
    normalize(
        ax * (1.0 - w) + bx * sign * w,
        ay * (1.0 - w) + by * sign * w,
    )
}

#[inline]
fn primary_slot_flow_weight(
    anisotropy: f32,
    spine: f32,
    stroke_room: f32,
    edge_rail: f32,
    tangent_y_abs: f32,
) -> f32 {
    let support = (stroke_room * 0.72 + spine * 0.16).max(edge_rail * 0.86);
    let field_conf = (0.22 + anisotropy * 1.02 + spine * 0.24 + edge_rail * 0.18).clamp(0.0, 1.0);
    let vertical_floor =
        smooth_up(0.26, 0.74, tangent_y_abs) * (0.58 + edge_rail * 0.18 + stroke_room * 0.12);
    (support * field_conf).max(vertical_floor).clamp(0.0, 0.94)
}

#[derive(Clone, Copy)]
struct RailSegment {
    ax: f32,
    ay: f32,
    bx: f32,
    by: f32,
    width: f32,
}

#[derive(Clone, Copy)]
struct RailSample {
    px: f32,
    py: f32,
    tx: f32,
    ty: f32,
    nx: f32,
    ny: f32,
    signed: f32,
    dist: f32,
    width: f32,
}

const PRIMARY_RAILS: &[RailSegment] = &[
    RailSegment {
        ax: 30.0,
        ay: 56.0,
        bx: 30.0,
        by: 67.0,
        width: 5.2,
    },
    RailSegment {
        ax: 42.0,
        ay: 56.0,
        bx: 42.0,
        by: 67.0,
        width: 5.2,
    },
    RailSegment {
        ax: 30.0,
        ay: 67.0,
        bx: 34.0,
        by: 70.0,
        width: 5.6,
    },
    RailSegment {
        ax: 34.0,
        ay: 70.0,
        bx: 39.0,
        by: 70.0,
        width: 5.6,
    },
    RailSegment {
        ax: 39.0,
        ay: 70.0,
        bx: 42.0,
        by: 67.0,
        width: 5.6,
    },
    RailSegment {
        ax: 46.0,
        ay: 70.0,
        bx: 46.0,
        by: 56.0,
        width: 5.2,
    },
    RailSegment {
        ax: 46.0,
        ay: 57.0,
        bx: 51.0,
        by: 54.5,
        width: 5.4,
    },
    RailSegment {
        ax: 51.0,
        ay: 54.5,
        bx: 58.0,
        by: 58.0,
        width: 5.4,
    },
    RailSegment {
        ax: 59.0,
        ay: 58.0,
        bx: 59.0,
        by: 70.0,
        width: 5.2,
    },
    RailSegment {
        ax: 63.0,
        ay: 56.0,
        bx: 63.0,
        by: 67.0,
        width: 5.2,
    },
    RailSegment {
        ax: 76.0,
        ay: 56.0,
        bx: 76.0,
        by: 67.0,
        width: 5.2,
    },
    RailSegment {
        ax: 63.0,
        ay: 67.0,
        bx: 67.0,
        by: 70.0,
        width: 5.6,
    },
    RailSegment {
        ax: 67.0,
        ay: 70.0,
        bx: 72.0,
        by: 70.0,
        width: 5.6,
    },
    RailSegment {
        ax: 72.0,
        ay: 70.0,
        bx: 76.0,
        by: 67.0,
        width: 5.6,
    },
    RailSegment {
        ax: 90.0,
        ay: 56.0,
        bx: 81.0,
        by: 55.0,
        width: 5.0,
    },
    RailSegment {
        ax: 81.0,
        ay: 55.0,
        bx: 76.0,
        by: 60.0,
        width: 5.0,
    },
    RailSegment {
        ax: 76.0,
        ay: 60.0,
        bx: 87.0,
        by: 64.0,
        width: 5.0,
    },
    RailSegment {
        ax: 87.0,
        ay: 64.0,
        bx: 92.0,
        by: 67.0,
        width: 5.0,
    },
    RailSegment {
        ax: 92.0,
        ay: 67.0,
        bx: 87.0,
        by: 71.0,
        width: 5.0,
    },
    RailSegment {
        ax: 87.0,
        ay: 71.0,
        bx: 77.0,
        by: 70.0,
        width: 5.0,
    },
    RailSegment {
        ax: 96.0,
        ay: 56.0,
        bx: 96.0,
        by: 67.0,
        width: 5.2,
    },
    RailSegment {
        ax: 109.0,
        ay: 56.0,
        bx: 109.0,
        by: 67.0,
        width: 5.2,
    },
    RailSegment {
        ax: 96.0,
        ay: 67.0,
        bx: 100.0,
        by: 70.0,
        width: 5.6,
    },
    RailSegment {
        ax: 100.0,
        ay: 70.0,
        bx: 105.0,
        by: 70.0,
        width: 5.6,
    },
    RailSegment {
        ax: 105.0,
        ay: 70.0,
        bx: 109.0,
        by: 67.0,
        width: 5.6,
    },
    RailSegment {
        ax: 114.0,
        ay: 51.0,
        bx: 114.0,
        by: 71.0,
        width: 4.4,
    },
    RailSegment {
        ax: 128.0,
        ay: 51.0,
        bx: 128.0,
        by: 71.0,
        width: 4.4,
    },
    RailSegment {
        ax: 128.0,
        ay: 51.0,
        bx: 128.0,
        by: 71.0,
        width: 4.4,
    },
    RailSegment {
        ax: 147.0,
        ay: 70.0,
        bx: 147.0,
        by: 56.0,
        width: 5.0,
    },
    RailSegment {
        ax: 147.0,
        ay: 57.0,
        bx: 153.0,
        by: 54.5,
        width: 5.2,
    },
    RailSegment {
        ax: 153.0,
        ay: 54.5,
        bx: 158.0,
        by: 58.0,
        width: 5.2,
    },
    RailSegment {
        ax: 158.0,
        ay: 58.0,
        bx: 158.0,
        by: 70.0,
        width: 5.0,
    },
    RailSegment {
        ax: 158.0,
        ay: 57.0,
        bx: 164.0,
        by: 54.5,
        width: 5.2,
    },
    RailSegment {
        ax: 164.0,
        ay: 54.5,
        bx: 169.0,
        by: 58.0,
        width: 5.2,
    },
    RailSegment {
        ax: 169.0,
        ay: 58.0,
        bx: 169.0,
        by: 70.0,
        width: 5.0,
    },
    RailSegment {
        ax: 183.0,
        ay: 57.0,
        bx: 174.0,
        by: 55.0,
        width: 5.3,
    },
    RailSegment {
        ax: 174.0,
        ay: 55.0,
        bx: 167.0,
        by: 62.0,
        width: 5.3,
    },
    RailSegment {
        ax: 167.0,
        ay: 62.0,
        bx: 167.0,
        by: 70.0,
        width: 5.3,
    },
    RailSegment {
        ax: 167.0,
        ay: 70.0,
        bx: 174.0,
        by: 75.0,
        width: 5.3,
    },
    RailSegment {
        ax: 174.0,
        ay: 75.0,
        bx: 184.0,
        by: 72.0,
        width: 5.3,
    },
    RailSegment {
        ax: 185.0,
        ay: 55.0,
        bx: 185.0,
        by: 76.0,
        width: 5.1,
    },
    RailSegment {
        ax: 186.0,
        ay: 56.0,
        bx: 196.0,
        by: 56.0,
        width: 5.0,
    },
    RailSegment {
        ax: 196.0,
        ay: 56.0,
        bx: 200.0,
        by: 64.0,
        width: 5.0,
    },
    RailSegment {
        ax: 200.0,
        ay: 64.0,
        bx: 197.0,
        by: 71.0,
        width: 5.0,
    },
    RailSegment {
        ax: 197.0,
        ay: 71.0,
        bx: 186.0,
        by: 70.0,
        width: 5.0,
    },
];

fn nearest_primary_rail(fx: f32, fy: f32) -> RailSample {
    let mut best = RailSample {
        px: fx,
        py: fy,
        tx: 1.0,
        ty: 0.0,
        nx: 0.0,
        ny: 1.0,
        signed: 0.0,
        dist: INF,
        width: 5.0,
    };

    for rail in PRIMARY_RAILS {
        let sx = rail.bx - rail.ax;
        let sy = rail.by - rail.ay;
        let len2 = sx * sx + sy * sy;
        if len2 <= EPS {
            continue;
        }
        let t = (((fx - rail.ax) * sx + (fy - rail.ay) * sy) / len2).clamp(0.0, 1.0);
        let px = rail.ax + sx * t;
        let py = rail.ay + sy * t;
        let dx = fx - px;
        let dy = fy - py;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist < best.dist {
            let (tx, ty) = normalize(sx, sy);
            let nx = -ty;
            let ny = tx;
            best = RailSample {
                px,
                py,
                tx,
                ty,
                nx,
                ny,
                signed: dx * nx + dy * ny,
                dist,
                width: rail.width,
            };
        }
    }
    best
}

#[inline]
fn endpoint_force_scalar(q_i: f32, q_j: f32, d2: f32, pair_gate: f32) -> f32 {
    let magnetic = -q_i * q_j;
    (K_DIPOLE * magnetic / d2.max(SOFTEN)).clamp(-F_PAIR_MAX, F_PAIR_MAX) * pair_gate
}

#[inline]
fn phrase_writes_wand(phase: Phase, phrase: usize, cur: usize, nxt: usize) -> bool {
    match phase {
        Phase::Assemble | Phase::Dwell => phrase == cur,
        Phase::Morph => phrase == nxt,
    }
}

#[inline]
fn endpoint_broad_reach(len_i: f32, len_j: f32) -> f32 {
    CHAIN_FAR + (len_i + len_j) * 0.5
}

#[inline]
fn phrase_offset(phrase: usize) -> usize {
    phrase * FIELD_MAX
}

#[inline]
fn field_index(x: usize, y: usize) -> usize {
    y * FIELD_W + x
}

#[inline]
fn meta_offset(phrase: usize) -> usize {
    phrase * FIELD_META_STRIDE
}

#[inline]
fn screen_x(fx: usize, width: f32) -> f32 {
    (fx as f32 + 0.5) * width / FIELD_W as f32
}

#[inline]
fn screen_y(fy: usize, height: f32) -> f32 {
    (fy as f32 + 0.5) * height / FIELD_H as f32
}

fn deposit_debug_point(field: &mut [f32], width: f32, height: f32, px: f32, py: f32, value: f32) {
    let fx = (px / width.max(1.0) * (FIELD_W - 1) as f32).clamp(0.0, (FIELD_W - 1) as f32);
    let fy = (py / height.max(1.0) * (FIELD_H - 1) as f32).clamp(0.0, (FIELD_H - 1) as f32);
    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let x1 = (x0 + 1).min(FIELD_W - 1);
    let y1 = (y0 + 1).min(FIELD_H - 1);
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;
    let taps = [
        (field_index(x0, y0), (1.0 - tx) * (1.0 - ty)),
        (field_index(x1, y0), tx * (1.0 - ty)),
        (field_index(x0, y1), (1.0 - tx) * ty),
        (field_index(x1, y1), tx * ty),
    ];
    for (idx, weight) in taps {
        field[idx] = field[idx].max(value * weight);
    }
}

#[inline]
#[allow(static_mut_refs)]
unsafe fn sim() -> &'static mut Sim {
    SIM.as_mut()
        .expect("sim_init must run before exported sim calls")
}

#[inline]
fn heap_point(width: f32, height: f32, rng: &mut u32) -> (f32, f32) {
    let hx = width * 0.165;
    let hy = height * 0.675;
    let r = width * 0.140;
    let gx = rng_next(rng) + rng_next(rng) + rng_next(rng) - 1.5;
    let gy = rng_next(rng) + rng_next(rng) + rng_next(rng) - 1.5;
    (hx + gx * r, hy + gy * r * 0.58)
}

fn reset_density_fields(s: &mut Sim) {
    s.rho.fill(0.0);
    s.desired.fill(0.0);
    s.void.fill(0.0);
    s.pressure.fill(0.0);
    s.rho_force_x.fill(0.0);
    s.rho_force_y.fill(0.0);
}

fn reset_particles_to_heap(s: &mut Sim) {
    let slot_phrase = s.cur.min(MAX_PHRASES - 1);
    let slot_count = s.slot_count[slot_phrase].min(s.n);
    for i in 0..s.n {
        let (mut x, mut y) = heap_point(s.width, s.height, &mut s.rng);
        if i < slot_count {
            let si = slot_phrase * MAX_PARTICLES + i;
            let sx = s.slot_fx[si] * s.width / FIELD_W as f32;
            let sy = s.slot_fy[si] * s.height / FIELD_H as f32;
            let dx = x - sx;
            let dy = y - sy;
            let dist = (dx * dx + dy * dy).sqrt();
            if dist < BIRTH_SLOT_CLEARANCE {
                let (ux, uy) = if dist > EPS {
                    (dx / dist, dy / dist)
                } else {
                    let a = hash_u32((i as u32).wrapping_mul(747_796_405) ^ s.seed0) * TAU;
                    (a.cos(), a.sin())
                };
                x = sx + ux * BIRTH_SLOT_CLEARANCE;
                y = sy + uy * BIRTH_SLOT_CLEARANCE;
            }
        }
        s.heap_x[i] = x;
        s.heap_y[i] = y;
        s.x[i] = x;
        s.y[i] = y;
        s.vx[i] = (rng_next(&mut s.rng) - 0.5) * 0.12;
        s.vy[i] = (rng_next(&mut s.rng) - 0.5) * 0.12;
        s.ang[i] = rng_next(&mut s.rng) * TAU;
        s.axis_x[i] = s.ang[i].cos();
        s.axis_y[i] = s.ang[i].sin();
        s.omega[i] = 0.0;
        s.active[i] = 0.0;
        s.field_lock[i] = 0.0;
        s.chain[i] = 0.0;
        s.heat[i] = 0.0;
        s.speed[i] = 0.0;
        s.render_field[i] = 0.0;
        s.particle_pressure[i] = 0.0;
        s.capture_s[i] = -1.0;
        s.trace_len[i] = 0.0;
        s.rotation_trace[i] = 0.0;
        s.particle_state[i] = STATE_CHAOS;
    }
}

fn reset_for_seed(s: &mut Sim, seed: u32) {
    s.seed0 = if seed == 0 { 2246 } else { seed };
    s.rng = s.seed0;
    s.cur = 0;
    s.nxt = if s.phrase_count > 1 { 1 } else { 0 };
    s.phase = Phase::Assemble;
    s.phase_s = 0.0;
    s.total_s = 0.0;
    s.tick = 0;
    s.wand_front = 0.0;
    s.wand_y = s.height * 0.5;
    s.activation.fill(0.0);
    reset_density_fields(s);
    reset_particles_to_heap(s);
}

fn relax_distance(dist: &mut [f32], x: usize, y: usize, nx: usize, ny: usize, cost: f32) {
    let i = field_index(x, y);
    let n = field_index(nx, ny);
    let candidate = dist[n] + cost;
    if candidate < dist[i] {
        dist[i] = candidate;
    }
}

fn chamfer_distance(dist: &mut [f32]) {
    let diag = core::f32::consts::SQRT_2;
    for y in 0..FIELD_H {
        for x in 0..FIELD_W {
            if x > 0 {
                relax_distance(dist, x, y, x - 1, y, 1.0);
            }
            if y > 0 {
                relax_distance(dist, x, y, x, y - 1, 1.0);
            }
            if x > 0 && y > 0 {
                relax_distance(dist, x, y, x - 1, y - 1, diag);
            }
            if x + 1 < FIELD_W && y > 0 {
                relax_distance(dist, x, y, x + 1, y - 1, diag);
            }
        }
    }
    for y in (0..FIELD_H).rev() {
        for x in (0..FIELD_W).rev() {
            if x + 1 < FIELD_W {
                relax_distance(dist, x, y, x + 1, y, 1.0);
            }
            if y + 1 < FIELD_H {
                relax_distance(dist, x, y, x, y + 1, 1.0);
            }
            if x + 1 < FIELD_W && y + 1 < FIELD_H {
                relax_distance(dist, x, y, x + 1, y + 1, diag);
            }
            if x > 0 && y + 1 < FIELD_H {
                relax_distance(dist, x, y, x - 1, y + 1, diag);
            }
        }
    }
}

fn blur_2d(input: &[f32], output: &mut [f32], radius: isize) {
    let sigma = (radius as f32 * 0.48).max(0.75);
    for y in 0..FIELD_H {
        for x in 0..FIELD_W {
            let mut sum = 0.0;
            let mut weight = 0.0;
            for ky in -radius..=radius {
                for kx in -radius..=radius {
                    let sx = (x as isize + kx).clamp(0, FIELD_W as isize - 1) as usize;
                    let sy = (y as isize + ky).clamp(0, FIELD_H as isize - 1) as usize;
                    let r2 = (kx * kx + ky * ky) as f32;
                    let w = (-r2 / (2.0 * sigma * sigma)).exp();
                    sum += input[field_index(sx, sy)] * w;
                    weight += w;
                }
            }
            output[field_index(x, y)] = sum / weight.max(EPS);
        }
    }
}

fn blur_box(input: &[f32], temp: &mut [f32], output: &mut [f32], radius: isize) {
    let width = 2.0 * radius as f32 + 1.0;
    for y in 0..FIELD_H {
        for x in 0..FIELD_W {
            let mut sum = 0.0;
            for kx in -radius..=radius {
                let sx = (x as isize + kx).clamp(0, FIELD_W as isize - 1) as usize;
                sum += input[field_index(sx, y)];
            }
            temp[field_index(x, y)] = sum / width;
        }
    }
    for y in 0..FIELD_H {
        for x in 0..FIELD_W {
            let mut sum = 0.0;
            for ky in -radius..=radius {
                let sy = (y as isize + ky).clamp(0, FIELD_H as isize - 1) as usize;
                sum += temp[field_index(x, sy)];
            }
            output[field_index(x, y)] = sum / width;
        }
    }
}

fn rebuild_field(s: &mut Sim, phrase: usize) {
    let phrase = phrase.min(MAX_PHRASES - 1);
    let base = phrase_offset(phrase);
    let mut min_x = FIELD_W;
    let mut min_y = FIELD_H;
    let mut max_x = 0usize;
    let mut max_y = 0usize;
    let mut mass = 0.0;

    s.dist_inside.fill(0.0);
    s.dist_outside.fill(0.0);

    for y in 0..FIELD_H {
        for x in 0..FIELD_W {
            let i = field_index(x, y);
            let a = clamp01(s.field_alpha[base + i]);
            s.field_alpha[base + i] = a;
            let inside = a > 0.12;
            s.dist_inside[i] = if inside { 0.0 } else { INF };
            s.dist_outside[i] = if inside { INF } else { 0.0 };
            s.scratch_a[i] = a;
            if inside {
                min_x = min_x.min(x);
                max_x = max_x.max(x);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
                mass += a;
            }
        }
    }

    chamfer_distance(&mut s.dist_inside);
    chamfer_distance(&mut s.dist_outside);
    blur_2d(&s.scratch_a, &mut s.scratch_b, 2);

    let cell = 0.5 * (s.width / FIELD_W as f32 + s.height / FIELD_H as f32);
    let mut target_sum = 0.0;
    for i in 0..FIELD_MAX {
        let signed = (s.dist_inside[i] - s.dist_outside[i]) * cell;
        let alpha_blur = s.scratch_b[i];
        let alpha_gate = smooth_up(0.08, 0.45, alpha_blur);
        let depth = (-signed).max(0.0);
        let outside_near = (-signed.max(0.0) / 88.0).exp();
        let interior = smooth_up(-28.0, -2.0, -signed);
        let edge = (-signed.abs() / 28.0).exp();
        let edge_rail = (-signed.abs() / 16.0).exp() * alpha_gate;
        let core_rail = smooth_up(5.0, 18.0, depth) * smooth_down(36.0, 82.0, depth);
        let spine = clamp01(core_rail * 0.82 + edge_rail * 0.30);
        let target_weight = s.field_alpha[base + i].powf(0.62) * (0.55 + spine * 2.10);
        s.field_sdf[base + i] = signed;
        s.field_potential[base + i] = clamp01(
            alpha_blur * 0.62 + interior * 0.30 + outside_near * 0.17 + edge * 0.10 + spine * 0.18,
        );
        s.field_spine[base + i] = spine;
        s.scratch_c[i] = target_weight;
        target_sum += target_weight;
    }

    for i in 0..FIELD_MAX {
        s.field_target_rho[base + i] = if target_sum > EPS {
            s.scratch_c[i] / target_sum
        } else {
            0.0
        };
    }

    for y in 0..FIELD_H {
        for x in 0..FIELD_W {
            let l = s.scratch_b[field_index(x.saturating_sub(1), y)];
            let r = s.scratch_b[field_index((x + 1).min(FIELD_W - 1), y)];
            let t = s.scratch_b[field_index(x, y.saturating_sub(1))];
            let b = s.scratch_b[field_index(x, (y + 1).min(FIELD_H - 1))];
            let gx = (r - l) * 0.5;
            let gy = (b - t) * 0.5;
            let i = field_index(x, y);
            s.scratch_c[i] = gx * gx;
            s.scratch_d[i] = gx * gy;
            s.rho[i] = gy * gy;
        }
    }

    blur_2d(&s.scratch_c, &mut s.void, 6);
    blur_2d(&s.scratch_d, &mut s.pressure, 6);
    blur_2d(&s.rho, &mut s.desired, 6);

    for y in 0..FIELD_H {
        for x in 0..FIELD_W {
            let i = field_index(x, y);
            let sdf_l = s.field_sdf[base + field_index(x.saturating_sub(1), y)];
            let sdf_r = s.field_sdf[base + field_index((x + 1).min(FIELD_W - 1), y)];
            let sdf_t = s.field_sdf[base + field_index(x, y.saturating_sub(1))];
            let sdf_b = s.field_sdf[base + field_index(x, (y + 1).min(FIELD_H - 1))];
            let (nx, ny) = normalize(sdf_r - sdf_l, sdf_b - sdf_t);
            let t_sdf_x = -ny;
            let t_sdf_y = nx;

            let jxx = s.void[i];
            let jxy = s.pressure[i];
            let jyy = s.desired[i];
            let theta = 0.5 * (2.0 * jxy).atan2(jxx - jyy);
            let major_x = theta.cos();
            let major_y = theta.sin();
            let t_tensor_x = -major_y;
            let t_tensor_y = major_x;
            let trace = jxx + jyy;
            let delta = ((jxx - jyy) * (jxx - jyy) * 0.25 + jxy * jxy).sqrt();
            let anisotropy = (2.0 * delta / (trace + EPS)).clamp(0.0, 1.0);

            let alpha_blur = s.scratch_b[i];
            let inside = smooth_up(0.10, 0.55, alpha_blur);
            let edge_band = (-s.field_sdf[base + i].abs() / 32.0).exp();
            let tensor_conf = (anisotropy * 5.5).clamp(0.0, 1.0) * inside;
            let sdf_conf = (edge_band * (1.0 - inside)).clamp(0.0, 1.0);
            let blend_w = tensor_conf / (tensor_conf + sdf_conf + EPS);
            let (tx, ty) = axis_blend(t_sdf_x, t_sdf_y, t_tensor_x, t_tensor_y, blend_w);

            s.field_tx[base + i] = tx;
            s.field_ty[base + i] = ty;
            s.field_nx[base + i] = nx;
            s.field_ny[base + i] = ny;
            s.field_anisotropy[base + i] = anisotropy;
        }
    }

    if mass <= 0.0 {
        min_x = FIELD_W / 2;
        max_x = FIELD_W / 2;
        min_y = FIELD_H / 2;
        max_y = FIELD_H / 2;
    }
    let mo = meta_offset(phrase);
    s.field_meta[mo] = FIELD_W as f32;
    s.field_meta[mo + 1] = FIELD_H as f32;
    s.field_meta[mo + 2] = min_x as f32 * s.width / FIELD_W as f32;
    s.field_meta[mo + 3] = (max_x + 1) as f32 * s.width / FIELD_W as f32;
    s.field_meta[mo + 4] = min_y as f32 * s.height / FIELD_H as f32;
    s.field_meta[mo + 5] = (max_y + 1) as f32 * s.height / FIELD_H as f32;
    s.field_meta[mo + 6] = mass;
    s.field_meta[mo + 7] = if mass > 0.0 { 1.0 } else { 0.0 };

    reset_density_fields(s);
}

fn rebuild_slots(s: &mut Sim, phrase: usize) {
    let phrase = phrase.min(MAX_PHRASES - 1);
    let base = phrase_offset(phrase);
    let slot_base = phrase * MAX_PARTICLES;
    let mut candidates = Vec::new();

    for i in 0..FIELD_MAX {
        if s.field_alpha[base + i] > 0.035 {
            candidates.push(i);
        }
    }

    let candidate_count = candidates.len();
    let desired =
        s.n.min(LETTER_SLOT_BUDGET)
            .min(candidate_count.saturating_mul(2));
    s.slot_count[phrase] = desired;
    if desired == 0 {
        return;
    }

    for written in 0..desired {
        let duplicate_count = desired.saturating_sub(candidate_count).max(1);
        let first_pass = written < candidate_count;
        let ci = if first_pass {
            written
        } else {
            (((written - candidate_count) * candidate_count) / duplicate_count)
                .min(candidate_count - 1)
        };
        let i = candidates[ci];
        let x = i % FIELD_W;
        let y = i / FIELD_W;
        let oi = base + i;
        let si = slot_base + written;
        let jitter = (written + phrase * 17) as u32;
        let base_tx = s.field_tx[oi];
        let base_ty = s.field_ty[oi];
        let base_nx = s.field_nx[oi];
        let base_ny = s.field_ny[oi];
        let stroke_room = smooth_up(7.0, 18.0, (-s.field_sdf[oi]).max(0.0));
        let edge_rail = (-s.field_sdf[oi].abs() / 17.0).exp() * s.field_alpha[oi];
        let mut flow_base_tx = base_tx;
        let mut flow_base_ty = base_ty;
        let mut rail_dist = INF;
        if phrase == 0 {
            let fx = x as f32 + 0.5;
            let fy = y as f32 + 0.5;
            let rail = nearest_primary_rail(fx, fy);
            rail_dist = rail.dist;
            let rail_core = smooth_down(0.8, rail.width + 2.4, rail.dist);
            let rail_snap = rail_core * (0.035 + edge_rail * 0.025 + stroke_room * 0.025);
            let rail_orient = smooth_down(2.8, rail.width + 7.0, rail.dist)
                .max(rail_core * 0.68)
                .clamp(0.0, 0.96);
            let along = (hash_u32(jitter.wrapping_mul(977)) - 0.5)
                * (0.18 + s.field_anisotropy[oi] * 0.10 + stroke_room * 0.12);
            let across = (hash_u32(jitter.wrapping_mul(571)) - 0.5)
                * (0.10 + (1.0 - s.field_spine[oi]).clamp(0.0, 1.0) * 0.10);
            let side = rail.signed.clamp(-rail.width * 0.55, rail.width * 0.55);
            let glyph_fx = fx + base_tx * along + base_nx * across;
            let glyph_fy = fy + base_ty * along + base_ny * across;
            let rail_fx =
                rail.px + rail.tx * along * 0.72 + rail.nx * (side * 0.72 + across * 0.28);
            let rail_fy =
                rail.py + rail.ty * along * 0.72 + rail.ny * (side * 0.72 + across * 0.28);
            s.slot_fx[si] = (glyph_fx * (1.0 - rail_snap) + rail_fx * rail_snap)
                .clamp(0.5, FIELD_W as f32 - 0.5);
            s.slot_fy[si] = (glyph_fy * (1.0 - rail_snap) + rail_fy * rail_snap)
                .clamp(0.5, FIELD_H as f32 - 0.5);
            let (rtx, rty) = axis_blend(base_tx, base_ty, rail.tx, rail.ty, rail_orient);
            flow_base_tx = rtx;
            flow_base_ty = rty;
        } else {
            s.slot_fx[si] = x as f32 + 0.5 + (hash_u32(jitter.wrapping_mul(977)) - 0.5) * 0.42;
            s.slot_fy[si] = y as f32 + 0.5 + (hash_u32(jitter.wrapping_mul(571)) - 0.5) * 0.42;
        }
        let (jtx, jty) = typeset_slot_tangent(jitter.wrapping_mul(1103515245));
        let flow_w = if phrase == 0 {
            primary_slot_flow_weight(
                s.field_anisotropy[oi],
                s.field_spine[oi],
                stroke_room,
                edge_rail,
                flow_base_ty.abs(),
            )
            .max(smooth_down(2.8, 11.0, rail_dist) * 0.74)
        } else {
            0.0
        };
        let (flow_tx, flow_ty) = axis_blend(1.0, 0.0, flow_base_tx, flow_base_ty, flow_w);
        let (tx, ty) = normalize(flow_tx * jtx - flow_ty * jty, flow_tx * jty + flow_ty * jtx);
        s.slot_tx[si] = tx;
        s.slot_ty[si] = ty;
        let alpha_priority = s.field_alpha[oi].max(0.24);
        let copy_priority = if !first_pass { 0.50 } else { 1.0 };
        s.slot_priority[si] = if phrase == 0 {
            let rail_priority = smooth_down(2.0, 8.5, rail_dist);
            ((alpha_priority
                * (0.74
                    + s.field_spine[oi] * 0.66
                    + s.field_anisotropy[oi] * 0.16
                    + edge_rail * 0.18
                    + rail_priority * 0.22))
                .clamp(0.0, 1.0))
                * copy_priority
        } else {
            (s.field_alpha[oi] * (0.70 + s.field_spine[oi] * 0.60)).clamp(0.0, 1.0) * copy_priority
        };
    }

    s.slot_count[phrase] = desired;
}

fn sample_phrase_array(array: &[f32], phrase: usize, fx: f32, fy: f32) -> f32 {
    let x0 = fx.floor().clamp(0.0, (FIELD_W - 1) as f32) as usize;
    let y0 = fy.floor().clamp(0.0, (FIELD_H - 1) as f32) as usize;
    let x1 = (x0 + 1).min(FIELD_W - 1);
    let y1 = (y0 + 1).min(FIELD_H - 1);
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;
    let base = phrase_offset(phrase);
    let a = array[base + field_index(x0, y0)];
    let b = array[base + field_index(x1, y0)];
    let c = array[base + field_index(x0, y1)];
    let d = array[base + field_index(x1, y1)];
    let ab = a + (b - a) * tx;
    let cd = c + (d - c) * tx;
    ab + (cd - ab) * ty
}

fn sample_runtime_array(array: &[f32], fx: f32, fy: f32) -> f32 {
    let x0 = fx.floor().clamp(0.0, (FIELD_W - 1) as f32) as usize;
    let y0 = fy.floor().clamp(0.0, (FIELD_H - 1) as f32) as usize;
    let x1 = (x0 + 1).min(FIELD_W - 1);
    let y1 = (y0 + 1).min(FIELD_H - 1);
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;
    let a = array[field_index(x0, y0)];
    let b = array[field_index(x1, y0)];
    let c = array[field_index(x0, y1)];
    let d = array[field_index(x1, y1)];
    let ab = a + (b - a) * tx;
    let cd = c + (d - c) * tx;
    ab + (cd - ab) * ty
}

fn sample_phrase_field(s: &Sim, phrase: usize, px: f32, py: f32) -> FieldSample {
    let fx = (px / s.width.max(1.0) * (FIELD_W - 1) as f32).clamp(0.0, (FIELD_W - 1) as f32);
    let fy = (py / s.height.max(1.0) * (FIELD_H - 1) as f32).clamp(0.0, (FIELD_H - 1) as f32);
    let tx = sample_phrase_array(&s.field_tx, phrase, fx, fy);
    let ty = sample_phrase_array(&s.field_ty, phrase, fx, fy);
    let (tx, ty) = normalize(tx, ty);
    let nx = sample_phrase_array(&s.field_nx, phrase, fx, fy);
    let ny = sample_phrase_array(&s.field_ny, phrase, fx, fy);
    let (nx, ny) = normalize(nx, ny);
    FieldSample {
        sdf: sample_phrase_array(&s.field_sdf, phrase, fx, fy),
        potential: sample_phrase_array(&s.field_potential, phrase, fx, fy),
        nx,
        ny,
        tx,
        ty,
        target_rho: sample_phrase_array(&s.field_target_rho, phrase, fx, fy),
        spine: sample_phrase_array(&s.field_spine, phrase, fx, fy),
        activation: sample_phrase_array(&s.activation, phrase, fx, fy),
        rho_force_x: sample_runtime_array(&s.rho_force_x, fx, fy),
        rho_force_y: sample_runtime_array(&s.rho_force_y, fx, fy),
        pressure: sample_runtime_array(&s.pressure, fx, fy),
    }
}

fn blend_samples_axis_aware(a: FieldSample, b: FieldSample, t: f32) -> FieldSample {
    let (tx, ty) = axis_blend(a.tx, a.ty, b.tx, b.ty, t);
    let (nx, ny) = normalize(a.nx + (b.nx - a.nx) * t, a.ny + (b.ny - a.ny) * t);
    FieldSample {
        sdf: a.sdf + (b.sdf - a.sdf) * t,
        potential: a.potential + (b.potential - a.potential) * t,
        nx,
        ny,
        tx,
        ty,
        target_rho: a.target_rho + (b.target_rho - a.target_rho) * t,
        spine: a.spine + (b.spine - a.spine) * t,
        activation: a.activation + (b.activation - a.activation) * t,
        rho_force_x: a.rho_force_x + (b.rho_force_x - a.rho_force_x) * t,
        rho_force_y: a.rho_force_y + (b.rho_force_y - a.rho_force_y) * t,
        pressure: a.pressure + (b.pressure - a.pressure) * t,
    }
}

fn sample_field(s: &Sim, px: f32, py: f32) -> FieldSample {
    match s.phase {
        Phase::Morph => {
            let t = smooth_up(0.0, 1.0, s.phase_s / MORPH);
            let cur = sample_phrase_field(s, s.cur, px, py);
            let nxt = sample_phrase_field(s, s.nxt, px, py);
            blend_samples_axis_aware(cur, nxt, t)
        }
        _ => sample_phrase_field(s, s.cur, px, py),
    }
}

fn update_one_activation(
    s: &mut Sim,
    phrase: usize,
    feed_enabled: bool,
    sustain_enabled: bool,
    write_wand: bool,
    sd: f32,
) {
    let mo = meta_offset(phrase);
    let min_x = s.field_meta[mo + 2];
    let max_x = s.field_meta[mo + 3];
    let min_y = s.field_meta[mo + 4];
    let max_y = s.field_meta[mo + 5];
    let span_x = (max_x - min_x).max(s.width * 0.08);
    let span_y = (max_y - min_y).max(s.height * 0.08);
    let progress = match s.phase {
        Phase::Assemble => smooth_up(0.02, 0.62, s.phase_s / ASSEMBLE),
        Phase::Dwell => 1.0,
        Phase::Morph => smooth_up(0.02, 0.96, s.phase_s / MORPH),
    };
    let wand_x = min_x - span_x * 0.18 + span_x * 1.36 * progress;
    let wand_y = (min_y + max_y) * 0.5 + (s.total_s * 2.1).sin() * span_y * 0.045;
    let rx = (span_x * 0.18).clamp(42.0, 130.0);
    let ry = (span_y * 0.75).clamp(48.0, 118.0);
    let base = phrase_offset(phrase);
    let decay = 0.994_f32.powf(sd.max(0.1));

    if write_wand {
        s.wand_front = wand_x;
        s.wand_y = wand_y;
    }

    for y in 0..FIELD_H {
        let sy = screen_y(y, s.height);
        for x in 0..FIELD_W {
            let sx = screen_x(x, s.width);
            let i = field_index(x, y);
            let oi = base + i;
            let old = s.activation[oi];
            let dx = (sx - wand_x) / rx;
            let dy = (sy - wand_y) / ry;
            let wand_lobe = (-(dx * dx) - (dy * dy)).exp();
            let sdf = s.field_sdf[oi];
            let potential = s.field_potential[oi];
            let target_rho = s.field_target_rho[oi];
            let letter_gate = potential * smooth_down(0.0, 96.0, sdf.max(0.0));
            let feed = if feed_enabled {
                wand_lobe * letter_gate * (0.16 + target_rho * 1.35) * sd
            } else {
                0.0
            };
            let sustain = if sustain_enabled {
                target_rho * 520.0 * 0.014 * sd
            } else {
                0.0
            };
            s.activation[oi] = (old * decay + feed + sustain).clamp(0.0, 1.0);
        }
    }
}

fn update_activation(s: &mut Sim, sd: f32) {
    match s.phase {
        Phase::Assemble => {
            update_one_activation(s, s.cur, true, false, true, sd);
            for p in 0..s.phrase_count {
                if p != s.cur {
                    update_one_activation(
                        s,
                        p,
                        false,
                        false,
                        phrase_writes_wand(s.phase, p, s.cur, s.nxt),
                        sd,
                    );
                }
            }
        }
        Phase::Dwell => {
            update_one_activation(s, s.cur, false, true, true, sd);
            for p in 0..s.phrase_count {
                if p != s.cur {
                    update_one_activation(
                        s,
                        p,
                        false,
                        false,
                        phrase_writes_wand(s.phase, p, s.cur, s.nxt),
                        sd,
                    );
                }
            }
        }
        Phase::Morph => {
            update_one_activation(
                s,
                s.cur,
                false,
                false,
                phrase_writes_wand(s.phase, s.cur, s.cur, s.nxt),
                sd,
            );
            update_one_activation(
                s,
                s.nxt,
                true,
                false,
                phrase_writes_wand(s.phase, s.nxt, s.cur, s.nxt),
                sd,
            );
        }
    }
}

fn deposit_density(s: &mut Sim, px: f32, py: f32, amount: f32) {
    let fx = (px / s.width.max(1.0) * (FIELD_W - 1) as f32).clamp(0.0, (FIELD_W - 1) as f32);
    let fy = (py / s.height.max(1.0) * (FIELD_H - 1) as f32).clamp(0.0, (FIELD_H - 1) as f32);
    let x0 = fx.floor() as usize;
    let y0 = fy.floor() as usize;
    let x1 = (x0 + 1).min(FIELD_W - 1);
    let y1 = (y0 + 1).min(FIELD_H - 1);
    let tx = fx - x0 as f32;
    let ty = fy - y0 as f32;
    s.rho[field_index(x0, y0)] += (1.0 - tx) * (1.0 - ty) * amount;
    s.rho[field_index(x1, y0)] += tx * (1.0 - ty) * amount;
    s.rho[field_index(x0, y1)] += (1.0 - tx) * ty * amount;
    s.rho[field_index(x1, y1)] += tx * ty * amount;
}

fn build_density(s: &mut Sim) {
    s.rho.fill(0.0);
    s.desired.fill(0.0);
    s.void.fill(0.0);
    s.pressure.fill(0.0);
    let mut active_area = 0.0;

    for i in 0..s.n {
        let axis_x = s.ang[i].cos();
        let axis_y = s.ang[i].sin();
        let rod_area = s.len[i] * (s.rad[i] * 2.0) + core::f32::consts::PI * s.rad[i] * s.rad[i];
        let amount = s.active[i] * rod_area * RHO_DEPOSIT_SCALE;
        active_area += amount;
        if amount > EPS {
            for k in 0..5 {
                let u = k as f32 / 4.0 - 0.5;
                let px = s.x[i] + axis_x * s.len[i] * u;
                let py = s.y[i] + axis_y * s.len[i] * u;
                deposit_density(s, px, py, amount / 5.0);
            }
        }
    }

    blur_box(&s.rho, &mut s.scratch_a, &mut s.scratch_c, 1);
    s.rho.copy_from_slice(&s.scratch_c);

    let mut active_target_sum = 0.0;
    for i in 0..FIELD_MAX {
        let mut weight = 0.0;
        for p in 0..s.phrase_count {
            let base = phrase_offset(p);
            weight += s.field_target_rho[base + i] * s.activation[base + i].powf(0.85);
        }
        s.scratch_b[i] = weight;
        active_target_sum += weight;
    }
    active_target_sum = active_target_sum.max(EPS);

    for i in 0..FIELD_MAX {
        let desired = s.scratch_b[i] / active_target_sum * active_area;
        s.desired[i] = desired;
        s.void[i] = (desired - s.rho[i]).max(0.0);
        s.pressure[i] = (s.rho[i] - desired * PRESSURE_OVERFILL).max(0.0);
    }

    blur_box(&s.void, &mut s.scratch_a, &mut s.scratch_c, 2);
    blur_box(&s.pressure, &mut s.scratch_a, &mut s.scratch_d, 2);
    s.void.copy_from_slice(&s.scratch_c);
    s.pressure.copy_from_slice(&s.scratch_d);

    for y in 0..FIELD_H {
        for x in 0..FIELD_W {
            let l_v = s.void[field_index(x.saturating_sub(1), y)];
            let r_v = s.void[field_index((x + 1).min(FIELD_W - 1), y)];
            let t_v = s.void[field_index(x, y.saturating_sub(1))];
            let b_v = s.void[field_index(x, (y + 1).min(FIELD_H - 1))];
            let l_p = s.pressure[field_index(x.saturating_sub(1), y)];
            let r_p = s.pressure[field_index((x + 1).min(FIELD_W - 1), y)];
            let t_p = s.pressure[field_index(x, y.saturating_sub(1))];
            let b_p = s.pressure[field_index(x, (y + 1).min(FIELD_H - 1))];
            let i = field_index(x, y);
            s.rho_force_x[i] = (r_v - l_v) * 0.5 * K_RHO_VOID - (r_p - l_p) * 0.5 * K_RHO_PRESSURE;
            s.rho_force_y[i] = (b_v - t_v) * 0.5 * K_RHO_VOID - (b_p - t_p) * 0.5 * K_RHO_PRESSURE;
        }
    }
}

fn hash_cell(s: &Sim, px: f32, py: f32) -> usize {
    let hx = (px / s.width.max(1.0) * HASH_W as f32)
        .floor()
        .clamp(0.0, (HASH_W - 1) as f32) as usize;
    let hy = (py / s.height.max(1.0) * HASH_H as f32)
        .floor()
        .clamp(0.0, (HASH_H - 1) as f32) as usize;
    hy * HASH_W + hx
}

fn build_hash(s: &mut Sim) {
    s.head.fill(-1);
    for i in 0..s.n {
        let cell = hash_cell(s, s.x[i], s.y[i]);
        s.next[i] = s.head[cell];
        s.head[cell] = i as i32;
    }
}

fn apply_pair_force(s: &mut Sim, i: usize, j: usize, fx: f32, fy: f32) {
    s.force_x[i] += fx;
    s.force_y[i] += fy;
    s.force_x[j] -= fx;
    s.force_y[j] -= fy;
}

fn neighbor_forces(s: &mut Sim) {
    s.force_x.fill(0.0);
    s.force_y.fill(0.0);
    s.torque.fill(0.0);
    s.pair_count = 0;
    s.pair_saturation_count = 0;
    for i in 0..s.n {
        s.chain[i] *= 0.90;
        s.axis_x[i] = s.ang[i].cos();
        s.axis_y[i] = s.ang[i].sin();
    }

    let cell_w = s.width / HASH_W as f32;
    let cell_h = s.height / HASH_H as f32;
    let rx = (PAIR_BROAD_REACH / cell_w.max(1.0)).ceil() as isize;
    let ry = (PAIR_BROAD_REACH / cell_h.max(1.0)).ceil() as isize;
    let pair_slot_phrase = match s.phase {
        Phase::Morph if s.phase_s / MORPH > 0.35 => s.nxt,
        _ => s.cur,
    };
    let slot_count = s.slot_count[pair_slot_phrase];

    for i in 0..s.n {
        let pair_interest_i = s.active[i].max(s.field_lock[i]).max(s.chain[i]);
        let primary_i = i < slot_count;
        let hx = (s.x[i] / s.width.max(1.0) * HASH_W as f32)
            .floor()
            .clamp(0.0, (HASH_W - 1) as f32) as isize;
        let hy = (s.y[i] / s.height.max(1.0) * HASH_H as f32)
            .floor()
            .clamp(0.0, (HASH_H - 1) as f32) as isize;
        for cy in (hy - ry)..=(hy + ry) {
            if cy < 0 || cy >= HASH_H as isize {
                continue;
            }
            for cx in (hx - rx)..=(hx + rx) {
                if cx < 0 || cx >= HASH_W as isize {
                    continue;
                }
                let mut j = s.head[cy as usize * HASH_W + cx as usize];
                while j >= 0 {
                    let jn = j as usize;
                    if !primary_i && jn >= slot_count {
                        j = s.next[jn];
                        continue;
                    }
                    let pair_interest_j = s.active[jn].max(s.field_lock[jn]).max(s.chain[jn]);
                    if jn > i && pair_interest_i.max(pair_interest_j) >= 0.06 {
                        pair_interaction(s, i, jn, slot_count);
                    }
                    j = s.next[jn];
                }
            }
        }
    }
}

fn pair_interaction(s: &mut Sim, i: usize, j: usize, slot_count: usize) -> bool {
    let rij_x = s.x[j] - s.x[i];
    let rij_y = s.y[j] - s.y[i];
    let center_d2 = rij_x * rij_x + rij_y * rij_y;
    if center_d2 < EPS {
        return false;
    }
    let primary_pair = i < slot_count && j < slot_count;
    let center_reach = if primary_pair {
        PAIR_CENTER_REACH
    } else {
        CHAIN_NEAR + (s.rad[i] + s.rad[j]) * 1.8
    };
    let center_possible = center_d2 <= center_reach * center_reach;
    let endpoint_possible = primary_pair && {
        let endpoint_reach = endpoint_broad_reach(s.len[i], s.len[j]);
        center_d2 <= endpoint_reach * endpoint_reach
    };
    if !center_possible && !endpoint_possible {
        return false;
    }
    let pair_gate = (s.active[i] * s.active[j]).sqrt();
    let core_limit = s.rad[i] + s.rad[j] + 1.10;
    let core_limit2 = core_limit * core_limit;
    if pair_gate < 0.01 && center_d2 > core_limit2 {
        return false;
    }

    let center_d = center_d2.sqrt();
    let center_ux = rij_x / center_d;
    let center_uy = rij_y / center_d;
    let axis_i_x = s.axis_x[i];
    let axis_i_y = s.axis_y[i];
    let axis_j_x = s.axis_x[j];
    let axis_j_y = s.axis_y[j];
    let axis_dot = axis_i_x * axis_j_x + axis_i_y * axis_j_y;
    let sign_j = if axis_dot < 0.0 { -1.0 } else { 1.0 };
    let axis_j_signed_x = axis_j_x * sign_j;
    let axis_j_signed_y = axis_j_y * sign_j;

    s.pair_count = s.pair_count.saturating_add(1);
    if center_possible && center_d < core_limit {
        let push = (core_limit - center_d) * K_CORE * (0.25 + pair_gate);
        apply_pair_force(s, i, j, -center_ux * push, -center_uy * push);
    }

    let parallel_base = (axis_i_x * axis_j_signed_x + axis_i_y * axis_j_signed_y).max(0.0);
    let parallel2 = parallel_base * parallel_base;
    let parallel = parallel2 * parallel2;
    let (lane_x, lane_y) = normalize(axis_i_x + axis_j_signed_x, axis_i_y + axis_j_signed_y);
    let along = rij_x * lane_x + rij_y * lane_y;
    let lat_x = rij_x - lane_x * along;
    let lat_y = rij_y - lane_y * along;
    let lat_len = (lat_x * lat_x + lat_y * lat_y).sqrt();
    let side_limit = s.rad[i] + s.rad[j] + SIDE_PAD;
    let overlap_gate = smooth_down(0.0, (s.len[i] + s.len[j]) * 0.45, along.abs());
    let side_gate = parallel * overlap_gate * pair_gate;
    if center_possible && lat_len < side_limit && side_gate > 0.001 {
        let (nx, ny) = if lat_len > EPS {
            (lat_x / lat_len, lat_y / lat_len)
        } else {
            (-lane_y, lane_x)
        };
        let push = (side_limit - lat_len) * K_SIDE * side_gate;
        apply_pair_force(s, i, j, -nx * push, -ny * push);
    }

    let axial_i = (center_ux * axis_i_x + center_uy * axis_i_y).abs();
    let axial_j = (center_ux * axis_j_signed_x + center_uy * axis_j_signed_y).abs();
    let axial_base = axial_i * axial_j;
    let axial_gate = axial_base * axial_base.sqrt();
    let mut chain_pair: f32 = 0.0;
    let sides = [-1.0, 1.0];
    let chain_far2 = CHAIN_FAR * CHAIN_FAR;

    if endpoint_possible {
        for side_i in sides {
            let endpoint_i_x = s.x[i] + axis_i_x * side_i * s.len[i] * 0.5;
            let endpoint_i_y = s.y[i] + axis_i_y * side_i * s.len[i] * 0.5;
            let q_i = side_i * s.polarity[i];
            for side_j in sides {
                let endpoint_j_x = s.x[j] + axis_j_x * side_j * s.len[j] * 0.5;
                let endpoint_j_y = s.y[j] + axis_j_y * side_j * s.len[j] * 0.5;
                let q_j = side_j * s.polarity[j];
                let r_x = endpoint_j_x - endpoint_i_x;
                let r_y = endpoint_j_y - endpoint_i_y;
                let r2_raw = r_x * r_x + r_y * r_y;
                if r2_raw > chain_far2 {
                    continue;
                }
                let d_raw = r2_raw.sqrt();
                let d2_force = r2_raw + SOFTEN;
                let (dir_x, dir_y) = normalize(r_x, r_y);
                let range_gate = smooth_down(CHAIN_NEAR, CHAIN_FAR, d_raw);
                let endpoint_gate = pair_gate * range_gate * (0.35 + 0.65 * axial_gate);
                let raw = K_DIPOLE * (-q_i * q_j) / d2_force.max(SOFTEN);
                let f_mag = endpoint_force_scalar(q_i, q_j, d2_force, endpoint_gate);
                if raw.abs() > F_PAIR_MAX {
                    s.pair_saturation_count = s.pair_saturation_count.saturating_add(1);
                }
                let f_x = dir_x * f_mag;
                let f_y = dir_y * f_mag;
                apply_pair_force(s, i, j, f_x, f_y);
                let torque_i = cross(endpoint_i_x - s.x[i], endpoint_i_y - s.y[i], f_x, f_y)
                    .clamp(-T_PAIR_MAX, T_PAIR_MAX);
                let torque_j = cross(endpoint_j_x - s.x[j], endpoint_j_y - s.y[j], -f_x, -f_y)
                    .clamp(-T_PAIR_MAX, T_PAIR_MAX);
                if torque_i.abs() >= T_PAIR_MAX || torque_j.abs() >= T_PAIR_MAX {
                    s.pair_saturation_count = s.pair_saturation_count.saturating_add(1);
                }
                s.torque[i] += torque_i;
                s.torque[j] += torque_j;
                if q_i * q_j < 0.0 {
                    chain_pair = chain_pair.max(endpoint_gate * axial_gate);
                }
            }
        }
    }

    if chain_pair > 0.0 {
        s.chain[i] = s.chain[i].max(chain_pair);
        s.chain[j] = s.chain[j].max(chain_pair);
    }
    true
}

fn decay_pair_memory(s: &mut Sim) {
    for i in 0..s.n {
        s.force_x[i] *= PAIR_FORCE_DECAY;
        s.force_y[i] *= PAIR_FORCE_DECAY;
        s.torque[i] *= PAIR_FORCE_DECAY;
    }
}

fn integrate_particles(s: &mut Sim, sd: f32) {
    for i in 0..s.n {
        let old_x = s.x[i];
        let old_y = s.y[i];
        let sample = sample_field(s, s.x[i], s.y[i]);
        let catchment = (-sample.sdf.max(0.0) / 310.0).exp();
        let activated_potential = sample.potential * sample.activation;
        let spine_field = sample.spine * sample.activation;
        let local_field =
            clamp01(sample.activation * 0.84 + activated_potential * 0.34 + spine_field * 0.30);
        let field_strength = clamp01(local_field + catchment * 0.045);
        let slot_phrase = match s.phase {
            Phase::Morph if s.phase_s / MORPH > 0.35 => s.nxt,
            _ => s.cur,
        };
        let mut slot_gate: f32 = 0.0;
        let mut slot_priority: f32 = 0.0;
        let mut slot_dx = 0.0;
        let mut slot_dy = 0.0;
        let mut slot_tx = sample.tx;
        let mut slot_ty = sample.ty;
        let mut slot_capture = 0.0;
        let mut halo_render = 0.0;
        let primary_slot = i < s.slot_count[slot_phrase];
        if primary_slot {
            let si = slot_phrase * MAX_PARTICLES + i;
            let sx = s.slot_fx[si] * s.width / FIELD_W as f32;
            let sy = s.slot_fy[si] * s.height / FIELD_H as f32;
            slot_dx = sx - s.x[i];
            slot_dy = sy - s.y[i];
            slot_tx = s.slot_tx[si];
            slot_ty = s.slot_ty[si];
            slot_priority = s.slot_priority[si];
            let front = s.wand_front - sx + s.width * 0.140;
            let front_gate = match s.phase {
                Phase::Assemble => smooth_up(-s.width * 0.16, s.width * 0.090, front),
                Phase::Dwell => 1.0,
                Phase::Morph => smooth_up(0.08, 0.56, s.phase_s / MORPH),
            };
            let phrase_gate = match s.phase {
                Phase::Assemble => {
                    let t = s.phase_s / ASSEMBLE;
                    let rightward_compensation = 0.86 + (sx / s.width).clamp(0.0, 1.0) * 0.30;
                    (smooth_up(0.12, 0.38, t)
                        * (0.50 + slot_priority * 0.56)
                        * rightward_compensation)
                        .min(1.0)
                }
                Phase::Dwell => 1.0,
                Phase::Morph => smooth_up(0.08, 0.56, s.phase_s / MORPH),
            };
            let dist = (slot_dx * slot_dx + slot_dy * slot_dy).sqrt();
            slot_gate = front_gate.max(phrase_gate) * (0.62 + slot_priority * 0.38);
            slot_capture = slot_gate * smooth_down(5.0, 42.0, dist);
        }

        let wand_dx = s.wand_front - s.x[i];
        let wand_dy = s.wand_y - s.y[i];
        let wand_d = (wand_dx * wand_dx + wand_dy * wand_dy).sqrt().max(EPS);
        let wand_progress = match s.phase {
            Phase::Assemble => smooth_up(0.02, 0.62, s.phase_s / ASSEMBLE),
            Phase::Morph => smooth_up(0.02, 0.96, s.phase_s / MORPH),
            Phase::Dwell => 0.0,
        };
        let wand_reach = smooth_down(0.0, s.width * 0.82, wand_d);
        let desired_active = clamp01(
            local_field * 1.50
                + sample.target_rho * 6200.0 * sample.activation * 0.57
                + wand_reach * wand_progress * 0.33
                + slot_gate * (0.42 + slot_priority * 0.44),
        );
        s.active[i] += (desired_active - s.active[i]) * (0.055 * sd).min(0.31);
        s.field_lock[i] += (field_strength - s.field_lock[i]) * (0.040 * sd).min(0.24);

        let nx = sample.nx;
        let ny = sample.ny;
        let outside_signed = (sample.sdf.max(0.0) / 92.0).clamp(0.0, 1.35);
        let inside_fill = smooth_up(-58.0, -5.0, sample.sdf)
            * sample.activation
            * (0.13 + sample.pressure * 0.10 + sample.spine * 0.11);
        let capture = (0.095 + s.active[i] * 0.62 + local_field * 0.90) * catchment;
        let letter_fill_pull = if primary_slot { inside_fill } else { 0.0 };
        let mut fx = s.force_x[i] - nx * (outside_signed * capture + letter_fill_pull);
        let mut fy = s.force_y[i] - ny * (outside_signed * capture + letter_fill_pull);

        fx += sample.rho_force_x * (1.65 + field_strength * 2.05 + sample.spine * 0.95);
        fy += sample.rho_force_y * (1.65 + field_strength * 2.05 + sample.spine * 0.95);
        if !primary_slot {
            let near_glyph = smooth_down(24.0, 92.0, sample.sdf);
            let inside_glyph = smooth_up(2.0, 56.0, -sample.sdf);
            let secondary_clear_boost = if s.phase == Phase::Dwell { 3.20 } else { 1.0 };
            let halo_pressure = (0.085 + sample.activation * 0.210 + field_strength * 0.140)
                * near_glyph
                * secondary_clear_boost
                + inside_glyph * (0.130 + sample.pressure * 0.085) * secondary_clear_boost;
            fx += nx * halo_pressure;
            fy += ny * halo_pressure;

            let halo_band =
                smooth_up(16.0, 58.0, sample.sdf) * smooth_down(72.0, 170.0, sample.sdf);
            let halo_orbit = halo_band * sample.activation * (0.018 + 0.024 * s.rough[i]);
            fx += sample.tx * s.polarity[i] * halo_orbit;
            fy += sample.ty * s.polarity[i] * halo_orbit;
            halo_render = halo_band
                * (0.118 + sample.activation * 0.064 + sample.pressure * 0.026)
                * (0.86 + s.rough[i] * 0.18);
        }
        let assemble_settle = if s.phase == Phase::Assemble {
            smooth_up(0.40, 0.68, s.phase_s / ASSEMBLE)
        } else {
            0.0
        };
        let assemble_motion_damping = if s.phase == Phase::Assemble {
            1.05 + assemble_settle * 0.50
        } else {
            1.0
        };
        if slot_gate > 0.0 {
            let dwell_settle_boost = if s.phase == Phase::Dwell {
                1.70
            } else {
                1.0 + assemble_settle * 0.55
            };
            let slot_spring = (0.030 + slot_priority * 0.038) * slot_gate * dwell_settle_boost;
            let slot_damping = (0.070 + slot_gate * 0.135) * assemble_motion_damping;
            fx += slot_dx * slot_spring - s.vx[i] * slot_damping;
            fy += slot_dy * slot_spring - s.vy[i] * slot_damping;
            s.chain[i] = s.chain[i].max(slot_capture * 0.92);
            s.field_lock[i] = s.field_lock[i].max(slot_gate * 0.86);
        }

        let edge_band = (-sample.sdf.abs() / 42.0).exp();
        let crawl = (s.total_s * 2.7 + s.phase_noise[i]).sin() * 0.014 * edge_band;
        fx += sample.tx * s.polarity[i] * crawl * s.active[i] * (0.3 + sample.potential);
        fy += sample.ty * s.polarity[i] * crawl * s.active[i] * (0.3 + sample.potential);

        let wand_pull = wand_reach * wand_progress * (0.085 + 0.235 * (1.0 - s.active[i] * 0.45));
        fx += wand_dx / wand_d * wand_pull;
        fy += wand_dy / wand_d * wand_pull;

        let front_lag = (s.wand_front - s.x[i]).max(0.0);
        let conveyor = smooth_up(24.0, s.width * 0.55, front_lag)
            * wand_progress
            * (0.050 + 0.150 * (1.0 - s.field_lock[i] * 0.35));
        fx += conveyor;
        fy += (s.wand_y - s.y[i]) / wand_d * conveyor * 1.82;

        let idle = 1.0 - s.active[i];
        let heap_gain = match s.phase {
            Phase::Assemble | Phase::Morph => 0.0008,
            Phase::Dwell => 0.0014,
        };
        let heap_pull = idle * idle * heap_gain;
        fx += (s.heap_x[i] - s.x[i]) * heap_pull;
        fy += (s.heap_y[i] - s.y[i]) * heap_pull;
        fx += (s.phase_noise[i] * 1.7 + s.total_s).sin() * 0.005 * idle;
        fy += (s.phase_noise[i] * 1.3 + s.total_s * 0.8).cos() * 0.004 * idle;
        if slot_gate > 0.0 {
            let slot_authority = (slot_gate * (0.38 + slot_priority * 0.58)).min(0.88);
            fx *= 1.0 - slot_authority;
            fy *= 1.0 - slot_authority;
            let dwell_settle_boost = if s.phase == Phase::Dwell {
                1.70
            } else {
                1.0 + assemble_settle * 0.55
            };
            let settle_spring = (0.024 + slot_priority * 0.032) * slot_gate * dwell_settle_boost;
            let settle_damping = (0.044 + slot_gate * 0.092) * assemble_motion_damping;
            fx += slot_dx * settle_spring - s.vx[i] * settle_damping;
            fy += slot_dy * settle_spring - s.vy[i] * settle_damping;
        }

        let force_mag = (fx * fx + fy * fy).sqrt();
        let speed_before = (s.vx[i] * s.vx[i] + s.vy[i] * s.vy[i]).sqrt();
        let pin_threshold = 0.026 + s.rough[i] * 0.040 + sample.pressure * 0.035;
        let stuck = s.active[i] > 0.35 && speed_before < 0.34 && force_mag < pin_threshold;

        if stuck {
            s.vx[i] *= 0.38;
            s.vy[i] *= 0.38;
            s.omega[i] *= 0.46;
        } else {
            let inv_mass = 1.0 / s.mass[i].max(0.1);
            s.vx[i] += fx * inv_mass * sd;
            s.vy[i] += fy * inv_mass * sd;
        }

        let dwell_drag = match s.phase {
            Phase::Dwell => s.active[i] * 0.040,
            Phase::Assemble => s.active[i] * slot_gate * 0.035 * assemble_settle,
            Phase::Morph => 0.0,
        };
        let drag = 0.932 - s.active[i] * 0.060 - s.rough[i] * 0.018 - dwell_drag;
        s.vx[i] *= drag.clamp(0.76, 0.960);
        s.vy[i] *= drag.clamp(0.76, 0.960);
        if s.phase == Phase::Dwell && slot_gate > 0.0 {
            let shimmer = (0.010 + s.rough[i] * 0.010 + slot_priority * 0.006) * slot_gate;
            s.vx[i] += (s.total_s * 8.7 + s.phase_noise[i]).sin() * shimmer;
            s.vy[i] += (s.total_s * 7.9 + s.phase_noise[i] * 1.3).cos() * shimmer * 0.72;
        }
        let speed = (s.vx[i] * s.vx[i] + s.vy[i] * s.vy[i]).sqrt();
        let max_speed = 2.25
            + s.active[i] * 0.78
            + field_strength * 0.82
            + wand_reach * wand_progress * 2.35
            + slot_gate * (2.30 + slot_priority * 2.92);
        if speed > max_speed {
            let scale = max_speed / speed;
            s.vx[i] *= scale;
            s.vy[i] *= scale;
            s.clamp_count = s.clamp_count.saturating_add(1);
        }
        s.x[i] += s.vx[i] * sd;
        s.y[i] += s.vy[i] * sd;

        if s.x[i] < -32.0 {
            s.x[i] = -32.0;
            s.vx[i] = s.vx[i].abs() * 0.35;
            s.clamp_count = s.clamp_count.saturating_add(1);
        } else if s.x[i] > s.width + 32.0 {
            s.x[i] = s.width + 32.0;
            s.vx[i] = -s.vx[i].abs() * 0.35;
            s.clamp_count = s.clamp_count.saturating_add(1);
        }
        if s.y[i] < -32.0 {
            s.y[i] = -32.0;
            s.vy[i] = s.vy[i].abs() * 0.35;
            s.clamp_count = s.clamp_count.saturating_add(1);
        } else if s.y[i] > s.height + 32.0 {
            s.y[i] = s.height + 32.0;
            s.vy[i] = -s.vy[i].abs() * 0.35;
            s.clamp_count = s.clamp_count.saturating_add(1);
        }
        let moved_x = s.x[i] - old_x;
        let moved_y = s.y[i] - old_y;
        s.trace_len[i] += (moved_x * moved_x + moved_y * moved_y).sqrt();

        let target_axis = slot_ty.atan2(slot_tx);
        let field_align = clamp01(
            edge_band * 0.30 + sample.activation * 0.54 + sample.spine * 0.44 + slot_gate * 0.62,
        );
        let axis_error = axis_delta(target_axis, s.ang[i]);
        let slot_bite = slot_gate * (0.012 + slot_priority * 0.018);
        let align_torque =
            axis_error * (0.024 + field_strength * 0.070 + slot_bite) * s.active[i] * field_align;
        let velocity_torque = if speed > 0.16 {
            axis_delta(s.vy[i].atan2(s.vx[i]), s.ang[i]) * 0.008 * idle
        } else {
            0.0
        };
        let magnetic_precession = (s.total_s * 8.9 + s.phase_noise[i] * 1.7).sin()
            * slot_gate
            * s.active[i]
            * (1.0 - s.chain[i] * 0.46).max(0.12)
            * (0.020 + slot_priority * 0.021);
        let moment = s.moment[i].max(0.2);
        if !stuck {
            s.omega[i] +=
                (align_torque + velocity_torque + magnetic_precession + s.torque[i]) * sd / moment;
        } else {
            s.omega[i] += (align_torque * 0.35 + magnetic_precession * 0.55) * sd / moment;
        }
        s.omega[i] *= 0.90 - s.active[i] * 0.04;
        let omega_limit = 0.24 + slot_gate * (0.006 + slot_priority * 0.012);
        if s.omega[i].abs() > omega_limit {
            s.omega[i] = s.omega[i].signum() * omega_limit;
            s.clamp_count = s.clamp_count.saturating_add(1);
        }
        let delta_ang = s.omega[i] * sd;
        s.ang[i] += delta_ang;
        s.rotation_trace[i] += delta_ang.abs();
        s.speed[i] = (s.vx[i] * s.vx[i] + s.vy[i] * s.vy[i]).sqrt();
        if primary_slot {
            let si = slot_phrase * MAX_PARTICLES + i;
            let sx = s.slot_fx[si] * s.width / FIELD_W as f32;
            let sy = s.slot_fy[si] * s.height / FIELD_H as f32;
            let dx = sx - s.x[i];
            let dy = sy - s.y[i];
            let slot_dist_now = (dx * dx + dy * dy).sqrt();
            let slot_axis_now = s.slot_ty[si].atan2(s.slot_tx[si]);
            let slot_angle_now = axis_delta(slot_axis_now, s.ang[i]).abs();
            let captured_now = slot_gate > 0.0 && slot_dist_now <= 14.0 && s.active[i] > 0.25;
            if captured_now && s.capture_s[i] < 0.0 {
                s.capture_s[i] = s.total_s;
            }
            s.particle_state[i] = if slot_gate <= 0.001 {
                if s.active[i] < 0.05 {
                    STATE_CHAOS
                } else {
                    STATE_AGITATED
                }
            } else if slot_dist_now > 42.0 {
                STATE_CLAIMED
            } else if slot_dist_now > 18.0 {
                STATE_CAPTURED
            } else if slot_angle_now > 0.70 {
                STATE_ALIGNING
            } else if s.speed[i] < 0.42 {
                STATE_SETTLED
            } else {
                STATE_LOCKED
            };
        } else {
            s.particle_state[i] = if s.active[i] < 0.05 {
                STATE_CHAOS
            } else {
                STATE_SEARCH
            };
        }
        s.render_field[i] = local_field.max(slot_gate * 0.90).max(halo_render);
        s.particle_pressure[i] = sample.pressure;
        s.heat[i] += (field_strength - s.heat[i]) * 0.032 * sd;
    }
}

fn publish_output(s: &mut Sim) {
    s.glint_count = 0;
    let mut avg_active = 0.0;
    let mut avg_chain = 0.0;
    let mut avg_speed = 0.0;
    let mut avg_pressure = 0.0;
    let mut chain_hist = [0usize; 10];

    for i in 0..s.n {
        let field_strength = s.render_field[i];
        let pressure = s.particle_pressure[i];
        let glint_seed = clamp01(s.shine[i] * (0.35 + field_strength * 0.30 + s.chain[i] * 0.35));
        if glint_seed > 0.42 {
            s.glint_count = s.glint_count.saturating_add(1);
        }
        avg_active += s.active[i];
        avg_chain += s.chain[i];
        avg_speed += s.speed[i];
        avg_pressure += pressure;
        let bin = (s.chain[i] * 9.99).clamp(0.0, 9.0) as usize;
        chain_hist[bin] += 1;

        let o = i * OUT_STRIDE;
        let slot_phrase = match s.phase {
            Phase::Morph if s.phase_s / MORPH > 0.35 => s.nxt,
            _ => s.cur,
        };
        let structural_priority = if i < s.slot_count[slot_phrase] {
            s.slot_priority[slot_phrase * MAX_PARTICLES + i]
        } else {
            0.0
        };
        s.out[o] = s.x[i];
        s.out[o + 1] = s.y[i];
        s.out[o + 2] = s.ang[i];
        s.out[o + 3] = s.len[i];
        s.out[o + 4] = s.rad[i];
        s.out[o + 5] = s.active[i];
        s.out[o + 6] = field_strength;
        s.out[o + 7] = s.chain[i];
        s.out[o + 8] = pressure;
        s.out[o + 9] = glint_seed;
        s.out[o + 10] = s.depth[i];
        s.out[o + 11] = s.rough[i];
        s.out[o + 12] = s.speed[i];
        s.out[o + 13] = structural_priority;
    }

    let inv_n = 1.0 / s.n.max(1) as f32;
    avg_active *= inv_n;
    avg_chain *= inv_n;
    avg_speed *= inv_n;
    avg_pressure *= inv_n;

    let mut cumulative = 0usize;
    let p90_mark = (s.n as f32 * 0.90) as usize;
    let mut p90_chain = 0.0;
    for (idx, count) in chain_hist.iter().enumerate() {
        cumulative += count;
        if cumulative >= p90_mark {
            p90_chain = idx as f32 / 9.0;
            break;
        }
    }

    unsafe {
        PERF[0] = s.phase_s;
        PERF[1] = match s.phase {
            Phase::Assemble => 0.0,
            Phase::Dwell => 1.0,
            Phase::Morph => 2.0,
        };
        PERF[2] = match s.phase {
            Phase::Morph => s.nxt as f32,
            _ => s.cur as f32,
        };
        PERF[3] = s.n as f32;
        PERF[4] = avg_active;
        PERF[5] = avg_chain;
        PERF[6] = p90_chain;
        PERF[7] = avg_speed;
        PERF[8] = s.pair_count as f32;
        PERF[9] = s.clamp_count as f32;
        PERF[10] = s.glint_count as f32;
        PERF[11] = avg_pressure;
        PERF[12] = s.wand_front;
        PERF[13] = s.width;
        PERF[14] = s.height;
        PERF[15] = s.pair_saturation_count as f32;
    }
}

#[no_mangle]
pub extern "C" fn sim_init(n: u32, seed: u32) -> *const f32 {
    let n = (n as usize).min(MAX_PARTICLES);
    let seed0 = if seed == 0 { 2246 } else { seed };
    let mut rng = seed0;
    let mut s = Sim {
        n,
        phrase_count: MAX_PHRASES,
        width: 1280.0,
        height: 720.0,
        seed0,
        rng,
        cur: 0,
        nxt: 1,
        phase: Phase::Assemble,
        phase_s: 0.0,
        total_s: 0.0,
        tick: 0,
        wand_front: 0.0,
        wand_y: 360.0,

        field_alpha: vec![0.0; MAX_PHRASES * FIELD_MAX],
        field_sdf: vec![0.0; MAX_PHRASES * FIELD_MAX],
        field_potential: vec![0.0; MAX_PHRASES * FIELD_MAX],
        field_nx: vec![1.0; MAX_PHRASES * FIELD_MAX],
        field_ny: vec![0.0; MAX_PHRASES * FIELD_MAX],
        field_tx: vec![1.0; MAX_PHRASES * FIELD_MAX],
        field_ty: vec![0.0; MAX_PHRASES * FIELD_MAX],
        field_target_rho: vec![0.0; MAX_PHRASES * FIELD_MAX],
        field_spine: vec![0.0; MAX_PHRASES * FIELD_MAX],
        field_anisotropy: vec![0.0; MAX_PHRASES * FIELD_MAX],
        field_meta: vec![0.0; MAX_PHRASES * FIELD_META_STRIDE],
        slot_count: [0; MAX_PHRASES],
        slot_fx: vec![0.0; MAX_PHRASES * MAX_PARTICLES],
        slot_fy: vec![0.0; MAX_PHRASES * MAX_PARTICLES],
        slot_tx: vec![1.0; MAX_PHRASES * MAX_PARTICLES],
        slot_ty: vec![0.0; MAX_PHRASES * MAX_PARTICLES],
        slot_priority: vec![0.0; MAX_PHRASES * MAX_PARTICLES],
        debug_field: vec![0.0; FIELD_MAX],

        activation: vec![0.0; MAX_PHRASES * FIELD_MAX],
        rho: vec![0.0; FIELD_MAX],
        desired: vec![0.0; FIELD_MAX],
        void: vec![0.0; FIELD_MAX],
        pressure: vec![0.0; FIELD_MAX],
        rho_force_x: vec![0.0; FIELD_MAX],
        rho_force_y: vec![0.0; FIELD_MAX],

        scratch_a: vec![0.0; FIELD_MAX],
        scratch_b: vec![0.0; FIELD_MAX],
        scratch_c: vec![0.0; FIELD_MAX],
        scratch_d: vec![0.0; FIELD_MAX],
        dist_inside: vec![0.0; FIELD_MAX],
        dist_outside: vec![0.0; FIELD_MAX],

        out: vec![0.0; n * OUT_STRIDE],

        x: vec![0.0; n],
        y: vec![0.0; n],
        vx: vec![0.0; n],
        vy: vec![0.0; n],
        ang: vec![0.0; n],
        omega: vec![0.0; n],
        axis_x: vec![1.0; n],
        axis_y: vec![0.0; n],
        len: vec![0.0; n],
        rad: vec![0.0; n],
        mass: vec![0.0; n],
        moment: vec![0.0; n],
        rough: vec![0.0; n],
        shine: vec![0.0; n],
        phase_noise: vec![0.0; n],
        polarity: vec![0.0; n],
        heat: vec![0.0; n],
        field_lock: vec![0.0; n],
        chain: vec![0.0; n],
        active: vec![0.0; n],
        depth: vec![0.0; n],
        speed: vec![0.0; n],
        render_field: vec![0.0; n],
        particle_pressure: vec![0.0; n],
        heap_x: vec![0.0; n],
        heap_y: vec![0.0; n],
        capture_s: vec![-1.0; n],
        trace_len: vec![0.0; n],
        rotation_trace: vec![0.0; n],
        particle_state: vec![STATE_CHAOS; n],

        force_x: vec![0.0; n],
        force_y: vec![0.0; n],
        torque: vec![0.0; n],
        head: vec![-1; HASH_MAX],
        next: vec![-1; n],

        pair_count: 0,
        pair_saturation_count: 0,
        clamp_count: 0,
        glint_count: 0,
    };

    for i in 0..n {
        let length = ROD_LEN_MIN + rng_next(&mut rng).powf(0.55) * ROD_LEN_SPAN;
        let radius = 0.46 + rng_next(&mut rng).powf(1.6) * 0.54;
        s.len[i] = length;
        s.rad[i] = radius;
        s.mass[i] = 0.68 + length * radius * 0.120;
        s.moment[i] = length * (0.88 + rng_next(&mut rng) * 0.56);
        s.rough[i] = rng_next(&mut rng);
        s.shine[i] = rng_next(&mut rng).powf(1.45);
        s.phase_noise[i] = rng_next(&mut rng) * TAU;
        s.polarity[i] = if rng_next(&mut rng) < 0.5 { -1.0 } else { 1.0 };
        s.depth[i] = rng_next(&mut rng);
        s.ang[i] = rng_next(&mut rng) * TAU;
    }
    s.rng = rng;
    reset_particles_to_heap(&mut s);

    unsafe {
        SIM = Some(s);
        sim().out.as_ptr()
    }
}

#[no_mangle]
pub extern "C" fn field_alpha_ptr() -> *mut f32 {
    unsafe { sim().field_alpha.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn field_meta_ptr() -> *mut f32 {
    unsafe { sim().field_meta.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn field_sdf_ptr() -> *mut f32 {
    unsafe { sim().field_sdf.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn field_potential_ptr() -> *mut f32 {
    unsafe { sim().field_potential.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn field_nx_ptr() -> *mut f32 {
    unsafe { sim().field_nx.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn field_ny_ptr() -> *mut f32 {
    unsafe { sim().field_ny.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn field_tx_ptr() -> *mut f32 {
    unsafe { sim().field_tx.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn field_ty_ptr() -> *mut f32 {
    unsafe { sim().field_ty.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn field_target_rho_ptr() -> *mut f32 {
    unsafe { sim().field_target_rho.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn field_spine_ptr() -> *mut f32 {
    unsafe { sim().field_spine.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn field_anisotropy_ptr() -> *mut f32 {
    unsafe { sim().field_anisotropy.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn activation_ptr() -> *mut f32 {
    unsafe { sim().activation.as_mut_ptr() }
}

#[no_mangle]
pub extern "C" fn out_ptr() -> *const f32 {
    unsafe { sim().out.as_ptr() }
}

#[no_mangle]
#[allow(static_mut_refs)]
pub extern "C" fn perf_ptr() -> *const f32 {
    unsafe { PERF.as_ptr() }
}

#[no_mangle]
pub extern "C" fn slot_count(phrase: u32) -> u32 {
    let s = unsafe { sim() };
    let phrase = (phrase as usize).min(MAX_PHRASES - 1);
    s.slot_count[phrase] as u32
}

#[no_mangle]
pub extern "C" fn slot_stride() -> u32 {
    MAX_PARTICLES as u32
}

#[no_mangle]
pub extern "C" fn slot_fx_ptr() -> *const f32 {
    unsafe { sim().slot_fx.as_ptr() }
}

#[no_mangle]
pub extern "C" fn slot_fy_ptr() -> *const f32 {
    unsafe { sim().slot_fy.as_ptr() }
}

#[no_mangle]
pub extern "C" fn slot_tx_ptr() -> *const f32 {
    unsafe { sim().slot_tx.as_ptr() }
}

#[no_mangle]
pub extern "C" fn slot_ty_ptr() -> *const f32 {
    unsafe { sim().slot_ty.as_ptr() }
}

#[no_mangle]
pub extern "C" fn slot_priority_ptr() -> *const f32 {
    unsafe { sim().slot_priority.as_ptr() }
}

#[no_mangle]
pub extern "C" fn heap_x_ptr() -> *const f32 {
    unsafe { sim().heap_x.as_ptr() }
}

#[no_mangle]
pub extern "C" fn heap_y_ptr() -> *const f32 {
    unsafe { sim().heap_y.as_ptr() }
}

#[no_mangle]
pub extern "C" fn capture_s_ptr() -> *const f32 {
    unsafe { sim().capture_s.as_ptr() }
}

#[no_mangle]
pub extern "C" fn trace_len_ptr() -> *const f32 {
    unsafe { sim().trace_len.as_ptr() }
}

#[no_mangle]
pub extern "C" fn rotation_trace_ptr() -> *const f32 {
    unsafe { sim().rotation_trace.as_ptr() }
}

#[no_mangle]
pub extern "C" fn particle_state_ptr() -> *const f32 {
    unsafe { sim().particle_state.as_ptr() }
}

#[no_mangle]
pub extern "C" fn field_w() -> u32 {
    FIELD_W as u32
}

#[no_mangle]
pub extern "C" fn field_h() -> u32 {
    FIELD_H as u32
}

#[no_mangle]
pub extern "C" fn out_stride() -> u32 {
    OUT_STRIDE as u32
}

#[no_mangle]
pub extern "C" fn perf_stride() -> u32 {
    PERF_STRIDE as u32
}

#[no_mangle]
pub extern "C" fn sim_config(phrase_count: u32, width: f32, height: f32) {
    let s = unsafe { sim() };
    s.phrase_count = (phrase_count as usize).clamp(1, MAX_PHRASES);
    s.width = width.max(1.0);
    s.height = height.max(1.0);
}

#[no_mangle]
pub extern "C" fn sim_rebuild_field(phrase: u32) {
    let s = unsafe { sim() };
    let phrase = (phrase as usize).min(MAX_PHRASES - 1);
    rebuild_field(s, phrase);
    rebuild_slots(s, phrase);
}

#[no_mangle]
pub extern "C" fn sim_reset() {
    let s = unsafe { sim() };
    reset_for_seed(s, s.seed0);
}

#[no_mangle]
pub extern "C" fn sim_reset_seed(seed: u32) {
    let s = unsafe { sim() };
    reset_for_seed(s, seed);
}

#[no_mangle]
pub extern "C" fn sim_to(idx: u32) {
    let s = unsafe { sim() };
    s.cur = (idx as usize) % s.phrase_count.max(1);
    s.nxt = if s.phrase_count > 1 {
        (s.cur + 1) % s.phrase_count
    } else {
        s.cur
    };
    s.phase = Phase::Assemble;
    s.phase_s = 0.0;
    s.total_s = 0.0;
    s.tick = 0;
    s.activation.fill(0.0);
    reset_density_fields(s);
    reset_particles_to_heap(s);
}

#[no_mangle]
pub extern "C" fn debug_field_ptr(kind: u32, phrase: u32) -> *const f32 {
    let s = unsafe { sim() };
    let phrase = (phrase as usize).min(MAX_PHRASES - 1);
    let base = phrase_offset(phrase);

    if kind == 11 {
        s.debug_field.fill(0.0);
        for i in 0..s.n {
            let value = s.chain[i].max(s.active[i] * 0.12);
            if value > 0.01 {
                deposit_debug_point(&mut s.debug_field, s.width, s.height, s.x[i], s.y[i], value);
            }
        }
        return s.debug_field.as_ptr();
    }

    for i in 0..FIELD_MAX {
        s.debug_field[i] = match kind {
            0 => s.field_alpha[base + i],
            1 => clamp01(0.5 - s.field_sdf[base + i] / 240.0),
            2 => s.field_potential[base + i],
            3 => s.field_target_rho[base + i] * 7000.0,
            4 => s.activation[base + i],
            5 => s.pressure[i] * 24.0,
            6 => s.field_tx[base + i] * 0.5 + 0.5,
            7 => s.field_anisotropy[base + i],
            8 => s.field_ty[base + i] * 0.5 + 0.5,
            9 => s.rho[i] * 0.018,
            10 => s.void[i] * 0.040,
            _ => 0.0,
        };
    }
    s.debug_field.as_ptr()
}

#[no_mangle]
pub extern "C" fn sim_step(dt_ms: f32) {
    let s = unsafe { sim() };
    let dt = dt_ms.clamp(0.0, 34.0);
    let sd = dt / 16.67;
    s.phase_s += dt * 0.001;
    s.total_s += dt * 0.001;

    if s.phase == Phase::Assemble && s.phase_s > ASSEMBLE {
        s.phase = Phase::Dwell;
        s.phase_s = 0.0;
    } else if s.phase == Phase::Dwell && s.phase_s > DWELL {
        s.nxt = if s.phrase_count > 1 {
            (s.cur + 1) % s.phrase_count
        } else {
            s.cur
        };
        s.phase = Phase::Morph;
        s.phase_s = 0.0;
    } else if s.phase == Phase::Morph && s.phase_s > MORPH {
        s.cur = s.nxt;
        s.nxt = if s.phrase_count > 1 {
            (s.cur + 1) % s.phrase_count
        } else {
            s.cur
        };
        s.phase = Phase::Dwell;
        s.phase_s = 0.0;
    }

    s.clamp_count = 0;
    update_activation(s, sd);
    if s.tick % DENSITY_CADENCE == 0 {
        build_density(s);
    }
    if s.tick % PAIR_CADENCE == 0 {
        build_hash(s);
        neighbor_forces(s);
    } else {
        decay_pair_memory(s);
    }
    s.tick = s.tick.wrapping_add(1);
    integrate_particles(s, sd);
    publish_output(s);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_falloffs_are_directional() {
        assert!(smooth_down(7.5, 28.0, 8.0) > 0.99);
        assert!(smooth_down(7.5, 28.0, 16.0) > 0.55);
        assert!(smooth_down(7.5, 28.0, 31.0) < 0.01);
        assert!(smooth_up(-58.0, -5.0, -60.0) < 0.01);
        assert!(smooth_up(-58.0, -5.0, -4.0) > 0.99);
    }

    #[test]
    fn endpoint_sign_convention_attracts_opposites() {
        let toward = endpoint_force_scalar(1.0, -1.0, 100.0, 1.0);
        let away = endpoint_force_scalar(1.0, 1.0, 100.0, 1.0);
        assert!(toward > 0.0);
        assert!(away < 0.0);
    }

    #[test]
    fn endpoint_torque_turns_attracting_tip_toward_connection() {
        let endpoint_i_x = 5.0;
        let endpoint_i_y = 0.0;
        let endpoint_j_x = 5.0;
        let endpoint_j_y = 4.0;
        let r_x = endpoint_j_x - endpoint_i_x;
        let r_y = endpoint_j_y - endpoint_i_y;
        let d2_force = r_x * r_x + r_y * r_y + SOFTEN;
        let (dir_x, dir_y) = normalize(r_x, r_y);
        let f_mag = endpoint_force_scalar(1.0, -1.0, d2_force, 1.0);
        let torque = cross(endpoint_i_x, endpoint_i_y, dir_x * f_mag, dir_y * f_mag);
        let desired_turn = axis_delta(endpoint_j_y.atan2(endpoint_j_x), 0.0);
        assert!(torque > 0.0);
        assert!(desired_turn > 0.0);
    }

    #[test]
    fn axis_delta_is_pi_periodic() {
        let a = axis_delta(0.0, core::f32::consts::PI - 0.1);
        assert!(a.abs() < 0.2);
    }

    #[test]
    fn wand_writer_policy_does_not_leak_next_phrase() {
        assert!(phrase_writes_wand(Phase::Assemble, 0, 0, 1));
        assert!(!phrase_writes_wand(Phase::Assemble, 1, 0, 1));
        assert!(phrase_writes_wand(Phase::Dwell, 0, 0, 1));
        assert!(!phrase_writes_wand(Phase::Dwell, 1, 0, 1));
        assert!(!phrase_writes_wand(Phase::Morph, 0, 0, 1));
        assert!(phrase_writes_wand(Phase::Morph, 1, 0, 1));
    }

    #[test]
    fn endpoint_broadphase_covers_facing_long_rods() {
        let len_i = MAX_ROD_LEN - 0.1;
        let len_j = MAX_ROD_LEN - 0.1;
        let center_d = PAIR_CENTER_REACH + MAX_ROD_LEN * 0.70;
        let endpoint_gap = center_d - len_i * 0.5 - len_j * 0.5;
        assert!(center_d > PAIR_CENTER_REACH);
        assert!(endpoint_gap <= CHAIN_FAR);
        assert!(center_d <= endpoint_broad_reach(len_i, len_j));
    }

    #[test]
    fn generated_rods_fit_hash_broadphase_bound() {
        assert!(ROD_LEN_MIN + ROD_LEN_SPAN <= MAX_ROD_LEN);
        assert!(PAIR_BROAD_REACH >= CHAIN_FAR + ROD_LEN_MIN + ROD_LEN_SPAN);
    }

    #[test]
    fn typeset_slot_tangent_stays_in_readability_lane() {
        for seed in [1, 17, 271, 4099, 65537, 999_983] {
            let (tx, ty) = typeset_slot_tangent(seed);
            assert!(tx > TYPESET_RAIL_JITTER_RAD.cos() - 0.0001);
            assert!(ty.abs() <= TYPESET_RAIL_JITTER_RAD.sin() + 0.0001);
        }
    }

    #[test]
    fn primary_slot_flow_weight_lets_glyph_rails_break_horizontal_lanes() {
        let blank = primary_slot_flow_weight(0.0, 0.0, 0.0, 0.0, 0.0);
        let vertical_edge = primary_slot_flow_weight(0.20, 0.0, 0.0, 1.0, 1.0);
        let interior_rail = primary_slot_flow_weight(0.55, 0.40, 1.0, 0.10, 0.35);

        assert!(blank < 0.001);
        assert!(vertical_edge > 0.72);
        assert!(interior_rail > 0.55);
    }

    #[test]
    fn endpoint_gate_uses_raw_distance_for_chain_far() {
        let raw_gap = CHAIN_FAR - 0.01;
        let r2_raw = raw_gap * raw_gap;
        let softened = r2_raw + SOFTEN;
        let gate = smooth_down(CHAIN_NEAR, CHAIN_FAR, raw_gap);
        assert!(r2_raw <= CHAIN_FAR * CHAIN_FAR);
        assert!(softened > CHAIN_FAR * CHAIN_FAR);
        assert!(gate > 0.0);
    }
}
