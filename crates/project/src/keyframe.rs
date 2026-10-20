//! Keyframed parameters.
//!
//! A parameter is a static value or a list of keyframes (clip-relative ticks). Interpolation per
//! keyframe governs the segment leaving it: Linear, Hold, Bezier (with influence/speed handles),
//! Auto Bezier, Continuous Bezier, Ease In/Out (Bezier presets) — Premiere's temporal set.

use filmcraft_geom::Vec2;
use filmcraft_time::Tick;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ParamValue {
    Float(f64),
    /// A point. A coordinate may be NaN: point parameters use NaN for "auto" (the frame or source
    /// centre) until [`crate::resolve_auto_points`] fills it in. JSON has no NaN and writes it as
    /// `null`, so `null` reads back as NaN here, and only here: a `null` coordinate anywhere else
    /// in a project (a mask vertex, a layer rectangle) is still a damaged file.
    Vec2(#[serde(deserialize_with = "point_or_auto")] Vec2),
    Color([f32; 4]),
    Bool(bool),
    Choice(u32),
    Text(String),
    /// Curve control points (x, y) in 0..1, sorted by x (Lumetri curves).
    Curve(Vec<[f32; 2]>),
    /// A mask path (Bézier vertices in clip pixels); interpolates vertex-wise.
    Path(crate::mask::MaskPath),
}

/// Read a point parameter's value, taking a `null` coordinate as NaN ("auto").
fn point_or_auto<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec2, D::Error> {
    fn nan_if_null<'de, D: serde::Deserializer<'de>>(d: D) -> Result<f64, D::Error> {
        Ok(Option::<f64>::deserialize(d)?.unwrap_or(f64::NAN))
    }
    #[derive(Deserialize)]
    struct Point {
        #[serde(deserialize_with = "nan_if_null")]
        x: f64,
        #[serde(deserialize_with = "nan_if_null")]
        y: f64,
    }
    let p = Point::deserialize(d)?;
    Ok(Vec2::new(p.x, p.y))
}

impl ParamValue {
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            ParamValue::Float(v) => Some(*v),
            ParamValue::Choice(c) => Some(*c as f64),
            ParamValue::Bool(b) => Some(*b as u8 as f64),
            _ => None,
        }
    }
    pub fn as_vec2(&self) -> Option<Vec2> {
        match self {
            ParamValue::Vec2(v) => Some(*v),
            _ => None,
        }
    }
    pub fn as_color(&self) -> Option<[f32; 4]> {
        match self {
            ParamValue::Color(c) => Some(*c),
            _ => None,
        }
    }
    pub fn as_curve(&self) -> Option<&[[f32; 2]]> {
        match self {
            ParamValue::Curve(c) => Some(c),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            ParamValue::Bool(b) => Some(*b),
            _ => None,
        }
    }
    /// Whether values of this kind can be interpolated (others hold).
    pub fn interpolates(&self) -> bool {
        matches!(self, ParamValue::Float(_) | ParamValue::Vec2(_) | ParamValue::Color(_) | ParamValue::Path(_))
    }
    pub fn as_path(&self) -> Option<&crate::mask::MaskPath> {
        match self {
            ParamValue::Path(p) => Some(p),
            _ => None,
        }
    }
    fn components(&self) -> Vec<f64> {
        match self {
            ParamValue::Float(v) => vec![*v],
            ParamValue::Vec2(v) => vec![v.x, v.y],
            ParamValue::Color(c) => c.iter().map(|&x| x as f64).collect(),
            ParamValue::Path(p) => p.components(),
            _ => vec![],
        }
    }
    fn from_components(template: &ParamValue, c: &[f64]) -> ParamValue {
        match template {
            ParamValue::Float(_) => ParamValue::Float(c[0]),
            ParamValue::Vec2(_) => ParamValue::Vec2(Vec2::new(c[0], c[1])),
            ParamValue::Color(_) => ParamValue::Color([c[0] as f32, c[1] as f32, c[2] as f32, c[3] as f32]),
            ParamValue::Path(p) => ParamValue::Path(p.with_components(c)),
            other => other.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Interpolation {
    #[default]
    Linear,
    Hold,
    Bezier,
    AutoBezier,
    ContinuousBezier,
    EaseIn,
    EaseOut,
}

impl Interpolation {
    pub const ALL: [Interpolation; 7] = [
        Interpolation::Linear,
        Interpolation::Bezier,
        Interpolation::AutoBezier,
        Interpolation::ContinuousBezier,
        Interpolation::Hold,
        Interpolation::EaseIn,
        Interpolation::EaseOut,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Interpolation::Linear => "Linear",
            Interpolation::Hold => "Hold",
            Interpolation::Bezier => "Bezier",
            Interpolation::AutoBezier => "Auto Bezier",
            Interpolation::ContinuousBezier => "Continuous Bezier",
            Interpolation::EaseIn => "Ease In",
            Interpolation::EaseOut => "Ease Out",
        }
    }

    /// Whether a keyframe with this interpolation carries an outgoing (right) influence
    /// handle: the segment leaving it eases out of the keyframe. One rule shared by the easing
    /// math ([`ease`]) and the handles the Effect Controls graphs draw and drag, so the graph
    /// shows exactly the handles the interpolation uses — no more and no fewer.
    pub fn eases_out(self) -> bool {
        matches!(self, Interpolation::Bezier | Interpolation::AutoBezier | Interpolation::ContinuousBezier | Interpolation::EaseOut)
    }

    /// Whether a keyframe with this interpolation carries an incoming (left) influence handle:
    /// a segment arriving at it eases into the keyframe.
    pub fn eases_in(self) -> bool {
        matches!(self, Interpolation::Bezier | Interpolation::AutoBezier | Interpolation::ContinuousBezier | Interpolation::EaseIn)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    /// Time relative to the start of the track item (clip time).
    pub time: Tick,
    pub value: ParamValue,
    /// Interpolation of the segment leaving this keyframe.
    pub interp: Interpolation,
    /// Temporal ease: influence (0..1) of the outgoing/incoming handles.
    #[serde(default = "default_influence")]
    pub out_influence: f64,
    #[serde(default = "default_influence")]
    pub in_influence: f64,
}

fn default_influence() -> f64 {
    1.0 / 3.0
}

impl Keyframe {
    pub fn new(time: Tick, value: ParamValue) -> Self {
        Self { time, value, interp: Interpolation::Linear, out_influence: default_influence(), in_influence: default_influence() }
    }
}

/// A (possibly animated) parameter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Param {
    pub value: ParamValue,
    /// Sorted by time. Empty = static.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keyframes: Vec<Keyframe>,
}

impl Param {
    pub fn new(value: ParamValue) -> Self {
        Self { value, keyframes: Vec::new() }
    }
    pub fn is_animated(&self) -> bool {
        !self.keyframes.is_empty()
    }
    /// Apply `f` to the static value and every keyframe value.
    pub fn map_values(&mut self, f: impl Fn(ParamValue) -> ParamValue) {
        self.value = f(std::mem::replace(&mut self.value, ParamValue::Float(0.0)));
        for k in &mut self.keyframes {
            k.value = f(std::mem::replace(&mut k.value, ParamValue::Float(0.0)));
        }
    }

    /// Set a value at `t`: updates the static value, or adds/replaces a keyframe when animated.
    pub fn set_at(&mut self, t: Tick, value: ParamValue) {
        if self.keyframes.is_empty() {
            self.value = value;
            return;
        }
        match self.keyframes.binary_search_by_key(&t, |k| k.time) {
            Ok(i) => self.keyframes[i].value = value,
            Err(i) => {
                let interp = self.keyframes.get(i.saturating_sub(1)).map(|k| k.interp).unwrap_or_default();
                let mut k = Keyframe::new(t, value);
                k.interp = interp;
                self.keyframes.insert(i, k);
            }
        }
    }

    /// Toggle animation (the stopwatch): on → one keyframe at `t` with the current value; off → keep value at `t`.
    pub fn toggle_animation(&mut self, t: Tick) {
        if self.keyframes.is_empty() {
            self.keyframes.push(Keyframe::new(t, self.value.clone()));
        } else {
            self.value = self.value_at(t);
            self.keyframes.clear();
        }
    }

    pub fn add_keyframe(&mut self, t: Tick) {
        let v = self.value_at(t);
        if self.keyframes.is_empty() {
            self.value = v.clone();
        }
        self.set_at_force(t, v);
    }

    /// Add or replace a keyframe at `t` (the parameter becomes animated).
    pub fn put_keyframe(&mut self, t: Tick, value: ParamValue) {
        self.set_at_force(t, value);
    }

    fn set_at_force(&mut self, t: Tick, value: ParamValue) {
        match self.keyframes.binary_search_by_key(&t, |k| k.time) {
            Ok(i) => self.keyframes[i].value = value,
            Err(i) => self.keyframes.insert(i, Keyframe::new(t, value)),
        }
    }

    pub fn remove_keyframe_at(&mut self, t: Tick) -> bool {
        if let Ok(i) = self.keyframes.binary_search_by_key(&t, |k| k.time) {
            let removed = self.keyframes.remove(i);
            if self.keyframes.is_empty() {
                self.value = removed.value;
            }
            true
        } else {
            false
        }
    }

    /// Evaluate at clip time `t`.
    pub fn value_at(&self, t: Tick) -> ParamValue {
        let k = &self.keyframes;
        if k.is_empty() {
            return self.value.clone();
        }
        if t <= k[0].time {
            return k[0].value.clone();
        }
        let last = &k[k.len() - 1];
        if t >= last.time {
            return last.value.clone();
        }
        let i = match k.binary_search_by_key(&t, |kf| kf.time) {
            Ok(i) => return k[i].value.clone(),
            Err(i) => i - 1,
        };
        let (a, b) = (&k[i], &k[i + 1]);
        if a.interp == Interpolation::Hold || !a.value.interpolates() {
            return a.value.clone();
        }
        let span = (b.time - a.time).0 as f64;
        let u = (t - a.time).0 as f64 / span;
        let e = ease(a, b, u);
        let ca = a.value.components();
        let cb = b.value.components();
        if ca.len() != cb.len() {
            // e.g. mask paths with different vertex counts: hold
            return a.value.clone();
        }
        let c: Vec<f64> = ca.iter().zip(&cb).map(|(x, y)| x + (y - x) * e).collect();
        ParamValue::from_components(&a.value, &c)
    }

    pub fn f64_at(&self, t: Tick) -> f64 {
        self.value_at(t).as_f64().unwrap_or(0.0)
    }

    /// Allocation-free [`Param::f64_at`] for scalar parameters (Float / Choice / Bool), for
    /// per-sample evaluation in the audio mixer. Non-scalar values evaluate to 0.
    pub fn scalar_at(&self, t: Tick) -> f64 {
        let k = &self.keyframes;
        let s = |v: &ParamValue| v.as_f64().unwrap_or(0.0);
        if k.is_empty() {
            return s(&self.value);
        }
        if t <= k[0].time {
            return s(&k[0].value);
        }
        let last = &k[k.len() - 1];
        if t >= last.time {
            return s(&last.value);
        }
        let i = match k.binary_search_by_key(&t, |kf| kf.time) {
            Ok(i) => return s(&k[i].value),
            Err(i) => i - 1,
        };
        let (a, b) = (&k[i], &k[i + 1]);
        let (va, vb) = (s(&a.value), s(&b.value));
        if a.interp == Interpolation::Hold || !matches!(a.value, ParamValue::Float(_)) {
            return va;
        }
        let u = (t - a.time).0 as f64 / (b.time - a.time).0 as f64;
        va + (vb - va) * ease(a, b, u)
    }

    pub fn vec2_at(&self, t: Tick) -> Vec2 {
        self.value_at(t).as_vec2().unwrap_or_default()
    }

    /// Previous/next keyframe times for navigation.
    pub fn prev_keyframe(&self, t: Tick) -> Option<Tick> {
        self.keyframes.iter().rev().map(|k| k.time).find(|&k| k < t)
    }
    pub fn next_keyframe(&self, t: Tick) -> Option<Tick> {
        self.keyframes.iter().map(|k| k.time).find(|&k| k > t)
    }
}

/// Temporal easing for u∈[0,1] between two keyframes (1D cubic Bezier in time/value space).
/// Each end follows its own keyframe: `a`'s outgoing handle ([`Interpolation::eases_out`])
/// shapes the start of the segment, and `b`'s incoming handle ([`Interpolation::eases_in`])
/// its arrival. A Bezier leaving a keyframe no longer also forces a flat, invisible arrival
/// on a Linear one: `b`'s own interpolation decides how the segment comes in, exactly as
/// `a`'s decides how it leaves — the handles the graph draws are the ones used.
fn ease(a: &Keyframe, b: &Keyframe, u: f64) -> f64 {
    let out_ease = a.interp.eases_out();
    let in_ease = b.interp.eases_in();
    if !out_ease && !in_ease {
        return u;
    }
    // Control points in normalized (time, value) space; flat handles = ease (velocity 0).
    let x1 = if out_ease { a.out_influence.clamp(0.01, 1.0) } else { 1.0 / 3.0 };
    let y1 = if out_ease { 0.0 } else { 1.0 / 3.0 };
    let x2 = if in_ease { 1.0 - b.in_influence.clamp(0.01, 1.0) } else { 2.0 / 3.0 };
    let y2 = if in_ease { 1.0 } else { 2.0 / 3.0 };
    cubic_bezier_y_for_x(x1, y1, x2, y2, u)
}

/// Solve a CSS-style cubic-bezier(x1,y1,x2,y2) for y at x (Newton + bisection).
pub fn cubic_bezier_y_for_x(x1: f64, y1: f64, x2: f64, y2: f64, x: f64) -> f64 {
    let bx = |s: f64| 3.0 * (1.0 - s) * (1.0 - s) * s * x1 + 3.0 * (1.0 - s) * s * s * x2 + s * s * s;
    let by = |s: f64| 3.0 * (1.0 - s) * (1.0 - s) * s * y1 + 3.0 * (1.0 - s) * s * s * y2 + s * s * s;
    let dbx = |s: f64| 3.0 * (1.0 - s) * (1.0 - s) * x1 + 6.0 * (1.0 - s) * s * (x2 - x1) + 3.0 * s * s * (1.0 - x2);
    let mut s = x;
    for _ in 0..8 {
        let d = dbx(s);
        if d.abs() < 1e-9 {
            break;
        }
        let ns = s - (bx(s) - x) / d;
        if !(0.0..=1.0).contains(&ns) {
            break;
        }
        s = ns;
    }
    if (bx(s) - x).abs() > 1e-7 {
        let (mut lo, mut hi) = (0.0, 1.0);
        for _ in 0..60 {
            s = (lo + hi) / 2.0;
            if bx(s) < x { lo = s } else { hi = s }
        }
    }
    by(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn linear_and_hold() {
        let mut p = Param::new(ParamValue::Float(0.0));
        p.toggle_animation(Tick(0));
        p.set_at(Tick(100), ParamValue::Float(10.0));
        assert_eq!(p.f64_at(Tick(50)), 5.0);
        assert_eq!(p.f64_at(Tick(-5)), 0.0);
        assert_eq!(p.f64_at(Tick(500)), 10.0);
        p.keyframes[0].interp = Interpolation::Hold;
        assert_eq!(p.f64_at(Tick(99)), 0.0);
    }

    #[test]
    fn bezier_eases() {
        let mut p = Param::new(ParamValue::Vec2(Vec2::new(0.0, 0.0)));
        p.toggle_animation(Tick(0));
        p.set_at(Tick(1000), ParamValue::Vec2(Vec2::new(100.0, 50.0)));
        p.keyframes[0].interp = Interpolation::Bezier;
        p.keyframes[1].interp = Interpolation::Bezier;
        let early = p.vec2_at(Tick(100)).x;
        let mid = p.vec2_at(Tick(500)).x;
        assert!(early < 10.0, "ease-in start should be slow: {early}");
        assert!((mid - 50.0).abs() < 1.0, "symmetric ease midpoint: {mid}");
        // monotonic
        let mut last = -1.0;
        for i in 0..=100 {
            let v = p.vec2_at(Tick(i * 10)).x;
            assert!(v >= last - 1e-9);
            last = v;
        }
    }

    /// An interpolation has an out-handle exactly for the modes that ease out of a keyframe,
    /// an in-handle exactly for those that ease into one — the rule the easing math and the
    /// Effect Controls handles share.
    #[test]
    fn interpolation_handle_visibility() {
        let handles = |i: Interpolation| (i.eases_out(), i.eases_in());
        assert_eq!(handles(Interpolation::Linear), (false, false));
        assert_eq!(handles(Interpolation::Hold), (false, false));
        assert_eq!(handles(Interpolation::Bezier), (true, true));
        assert_eq!(handles(Interpolation::AutoBezier), (true, true));
        assert_eq!(handles(Interpolation::ContinuousBezier), (true, true));
        assert_eq!(handles(Interpolation::EaseIn), (false, true));
        assert_eq!(handles(Interpolation::EaseOut), (true, false));
    }

    /// A Bezier keyframe leaving into a Linear one used to force a flat, invisible arrival on
    /// the Linear keyframe (the graph draws no in-handle there, so nothing could control it);
    /// the arrival now follows `b`'s own interpolation, as the start follows `a`'s.
    #[test]
    fn a_segment_arrives_on_its_own_keyframes_handles() {
        let key = |interp| Keyframe { interp, out_influence: 1.0 / 3.0, in_influence: 1.0 / 3.0, ..Keyframe::new(Tick(0), ParamValue::Float(0.0)) };
        let a = key(Interpolation::Bezier);
        let linear = key(Interpolation::Linear);
        let bezier = key(Interpolation::Bezier);
        // b Linear: no in-handle, so the segment may not flatten as it arrives — it keeps
        // coming in on b's own slope (it still leaves a slowly; only a's handle is flat).
        let end = ease(&a, &linear, 0.99) - ease(&a, &linear, 0.95);
        assert!(end > 0.02, "a Linear b arrives still moving, not flat: {end}");
        // b Bezier: an in-handle exists, and the segment eases in flat against it.
        let flat = ease(&a, &bezier, 0.99) - ease(&a, &bezier, 0.95);
        assert!(flat < end, "a Bezier b flattens where a Linear b keeps coming: {flat} < {end}");
        // and the eased position stays within the segment for the whole run
        for i in 0..=100 {
            let eased = ease(&a, &linear, i as f64 / 100.0);
            assert!((0.0..=1.0).contains(&eased), "easing stays between the keyframes: {eased}");
        }
    }

    #[test]
    fn toggle_off_keeps_value() {
        let mut p = Param::new(ParamValue::Float(1.0));
        p.toggle_animation(Tick(0));
        p.set_at(Tick(10), ParamValue::Float(3.0));
        p.toggle_animation(Tick(5));
        assert!(!p.is_animated());
        assert_eq!(p.value, ParamValue::Float(2.0));
    }
}
