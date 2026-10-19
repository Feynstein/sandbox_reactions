//! Real-equivalent translations (m0_contrat.md §1.10, G-READ; M0-T69).
//!
//! Display functions only: the physics never reads them (G-WATCH). Mass (§1.10.1) and temperature (§1.10.2) are each a
//! piecewise map through anchors, **log-linear**: between two anchors `ln y` is a straight line in `ln x`, so the map is
//! a power law on each segment and both sides stay positive. The sandbox side comes from calibration (mass) and
//! `physics.json` (temperature); the real side is the contract's table. Every map is strictly increasing — a construction
//! whose sandbox or real anchors are not strictly increasing is refused ([`TranslateError`]), which is how calibration
//! "fails otherwise" (§1.10.2).
//!
//! Beyond the anchors: mass extends its end segments (a power law below `M_ign` and above `M_BH`); temperature extends
//! the O–Si segment above `T_Si` and reads 10 K at and below `T_floor`.

use std::fmt;

use crate::calibration::Calibration;
use crate::registry::Physics;

/// §1.10.1's real masses in Suns, for the sandbox anchors `M_ign`, `M_Ch`, `M_up`, `M_BH` in this order.
pub const MASS_SUNS: [f64; 4] = [0.08, 1.44, 8.0, 25.0];

/// §1.10.2's real temperatures in kelvins, for the sandbox anchors `T_floor`, the Sun-like preset's T_eff, `T_H`, `T_He`,
/// `T_C`, `T_Ne`, `T_O`, `T_Si` in this order.
pub const TEMP_KELVIN: [f64; 8] = [10.0, 5772.0, 1.571e7, 1.8e8, 8.3e8, 1.6e9, 1.9e9, 3.3e9];

/// The presets' real masses in Suns (§2.10): Sun-like, Massive, Giant.
pub const PRESET_SUNS: [f64; 3] = [1.0, 15.0, 40.0];

/// Why a translation could not be built.
#[derive(Clone, Debug, PartialEq)]
pub enum TranslateError {
    /// An anchor is not a finite number > 0.
    NotPositive { side: &'static str, index: usize, value: f64 },
    /// The sandbox or the real side does not increase strictly at this anchor: the translation would not be monotonic.
    NotIncreasing { side: &'static str, index: usize, prev: f64, value: f64 },
    /// A `physics.json` key the temperature anchors need is missing.
    MissingConstant(&'static str),
}

impl fmt::Display for TranslateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TranslateError::NotPositive { side, index, value } => {
                write!(f, "{side} anchor {index} must be a finite number > 0, got {value}")
            }
            TranslateError::NotIncreasing { side, index, prev, value } => {
                write!(f, "{side} anchors must increase strictly: anchor {index} is {value}, after {prev}")
            }
            TranslateError::MissingConstant(key) => write!(f, "physics.json has no {key}"),
        }
    }
}

impl std::error::Error for TranslateError {}

/// A piecewise log-linear map through anchors `(x_i, y_i)`, both sides strictly increasing.
#[derive(Clone, Debug, PartialEq)]
struct LogMap {
    x: Vec<f64>,
    y: Vec<f64>,
    /// `true`: at and below the first anchor the map reads its `y` (a floor); `false`: the first segment is extended.
    floor: bool,
}

impl LogMap {
    fn new(x: Vec<f64>, y: Vec<f64>, floor: bool) -> Result<LogMap, TranslateError> {
        for (side, v) in [("sandbox", &x), ("real", &y)] {
            for (index, &value) in v.iter().enumerate() {
                if !(value.is_finite() && value > 0.0) {
                    return Err(TranslateError::NotPositive { side, index, value });
                }
                if index > 0 && value <= v[index - 1] {
                    return Err(TranslateError::NotIncreasing { side, index, prev: v[index - 1], value });
                }
            }
        }
        Ok(LogMap { x, y, floor })
    }

    /// `ln y` at `ln_in` on the segment `i` (between anchors `i` and `i + 1`, extended past either end).
    fn along(from: &[f64], to: &[f64], i: usize, input: f64) -> f64 {
        let slope = (to[i + 1].ln() - to[i].ln()) / (from[i + 1].ln() - from[i].ln());
        (to[i].ln() + slope * (input.ln() - from[i].ln())).exp()
    }

    /// The segment of `from` that `input` falls on: the last anchor at or below it, the end ones extended.
    fn segment(from: &[f64], input: f64) -> usize {
        from.iter().take(from.len() - 1).rposition(|&a| a <= input).unwrap_or(0)
    }

    fn apply(from: &[f64], to: &[f64], input: f64) -> f64 {
        match from.iter().position(|&a| a == input) {
            Some(i) => to[i],
            None => LogMap::along(from, to, LogMap::segment(from, input), input),
        }
    }

    fn forward(&self, x: f64) -> f64 {
        if x.is_nan() {
            return f64::NAN;
        }
        // A floor reads its first y at and below its first anchor; an extended power law reaches 0 at 0.
        if x <= self.x[0] && self.floor {
            return self.y[0];
        }
        if x <= 0.0 {
            return 0.0;
        }
        LogMap::apply(&self.x, &self.y, x)
    }

    fn inverse(&self, y: f64) -> f64 {
        if y.is_nan() {
            return f64::NAN;
        }
        if y <= self.y[0] && self.floor {
            return self.x[0];
        }
        if y <= 0.0 {
            return 0.0;
        }
        LogMap::apply(&self.y, &self.x, y)
    }
}

/// Sandbox mass ↔ Suns (§1.10.1).
#[derive(Clone, Debug, PartialEq)]
pub struct MassScale(LogMap);

impl MassScale {
    /// The four sandbox anchors `M_ign < M_Ch < M_up < M_BH`, paired with [`MASS_SUNS`].
    pub fn new(m_ign: f64, m_ch: f64, m_up: f64, m_bh: f64) -> Result<MassScale, TranslateError> {
        Ok(MassScale(LogMap::new(vec![m_ign, m_ch, m_up, m_bh], MASS_SUNS.to_vec(), false)?))
    }

    /// A sandbox mass in Suns; 0 reads 0.
    pub fn to_suns(&self, mass_sb: f64) -> f64 {
        self.0.forward(mass_sb)
    }

    /// The sandbox mass that reads `suns` (the preset masses, §2.10); the exact inverse of [`MassScale::to_suns`].
    pub fn to_sb(&self, suns: f64) -> f64 {
        self.0.inverse(suns)
    }
}

/// Sandbox temperature ↔ kelvins (§1.10.2).
#[derive(Clone, Debug, PartialEq)]
pub struct TempScale(LogMap);

impl TempScale {
    /// The eight sandbox anchors, paired with [`TEMP_KELVIN`]: `T_floor`, the Sun-like preset's T_eff at X_H,c = 0.35,
    /// then `T_H`, `T_He`, `T_C`, `T_Ne`, `T_O`, `T_Si`.
    pub fn new(anchors: [f64; 8]) -> Result<TempScale, TranslateError> {
        Ok(TempScale(LogMap::new(anchors.to_vec(), TEMP_KELVIN.to_vec(), true)?))
    }

    /// A sandbox temperature in kelvins; at and below `T_floor`, 10 K.
    pub fn to_kelvin(&self, t_sb: f64) -> f64 {
        self.0.forward(t_sb)
    }

    /// The sandbox temperature that reads `kelvin`; a reading at or under 10 K gives `T_floor` (the map is flat there).
    pub fn to_sb(&self, kelvin: f64) -> f64 {
        self.0.inverse(kelvin)
    }
}

/// §1.10.3's surface temperature in sandbox units: `L = 2π R σ T⁴` with `σ = a_r c_sb / 4`, so `T = (L / (2π R σ))^¼`.
/// `l` is the object's luminosity, `r` its radius; neither positive reads 0.
pub fn surface_temperature_sb(l: f64, r: f64, a_r: f64, c_sb: f64) -> f64 {
    let sigma = a_r * c_sb / 4.0;
    if !(l > 0.0 && r > 0.0 && sigma > 0.0) {
        return 0.0;
    }
    (l / (2.0 * std::f64::consts::PI * r * sigma)).powf(0.25)
}

/// Both translations of one calibration over one set of constants: what the readouts, the inspector and the colours use.
#[derive(Clone, Debug, PartialEq)]
pub struct Translations {
    pub mass: MassScale,
    pub temp: TempScale,
    /// `σ = a_r c_sb / 4` for [`Translations::surface_kelvin`].
    pub a_r: f64,
    pub c_sb: f64,
}

impl Translations {
    pub fn new(cal: &Calibration, physics: &Physics) -> Result<Translations, TranslateError> {
        let get = |key: &'static str| physics.get(key).ok_or(TranslateError::MissingConstant(key));
        let mass = MassScale::new(cal.m_ign_sb, cal.m_ch_sb, cal.m_up_sb, cal.m_bh_sb)?;
        let temp = TempScale::new([
            get("t_floor")?,
            cal.t_eff_sun_sb,
            get("t_h")?,
            get("t_he")?,
            get("t_c")?,
            get("t_ne")?,
            get("t_o")?,
            get("t_si")?,
        ])?;
        Ok(Translations { mass, temp, a_r: get("a_r")?, c_sb: get("c_sb")? })
    }

    /// An object's surface temperature in kelvins: T_eff from its luminosity and radius, translated (§1.10.3).
    pub fn surface_kelvin(&self, l: f64, r: f64) -> f64 {
        self.temp.to_kelvin(surface_temperature_sb(l, r, self.a_r, self.c_sb))
    }
}
