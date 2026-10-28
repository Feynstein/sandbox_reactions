//! Number formats (m0_contrat.md §4.12; English: "," groups thousands, "." is the decimal point). Every number the
//! app shows becomes text here, through the string table's templates (strings.rs, §4.1) — no caller formats a number
//! itself. Scope `formats` (`cargo test -p sr-app --release --bin sandbox-reactions format::`) holds every example
//! §4.12 gives, byte-identical, and each boundary.
//!
//! A boundary is judged on the *rounded* value: 999,999 K rounds to 1,000,000 at 3 significant digits, so it reads
//! "1.00 million K", never "1,000 thousand"; the same goes for ages, the inspector's scientific cut-offs and the rest.

use crate::strings;

/// 1 year = 365.25 days (§4.12).
const DAYS_PER_YEAR: f64 = 365.25;
const SECONDS_PER_DAY: f64 = 86_400.0;

/// Age and rate units, largest first: the string key and the unit's length in years (§4.12).
const AGE_UNITS: [(&str, f64); 8] = [
    ("units.years_trillion", 1e12),
    ("units.years_billion", 1e9),
    ("units.years_million", 1e6),
    ("units.years_thousand", 1e3),
    ("units.years", 1.0),
    ("units.days", 1.0 / DAYS_PER_YEAR),
    ("units.seconds", 1.0 / (DAYS_PER_YEAR * SECONDS_PER_DAY)),
    ("units.milliseconds", 1.0 / (DAYS_PER_YEAR * SECONDS_PER_DAY * 1000.0)),
];

/// The template `key` with each `{name}` replaced by its value.
fn fill(key: &str, args: &[(&str, &str)]) -> String {
    let mut text = strings::get(key).to_string();
    for (name, value) in args {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

/// `x` rounded to `n` significant digits, as the digits (no point, `n` of them) and the decimal exponent of the first.
fn digits_exp(x: f64, n: usize) -> (String, i32) {
    let s = format!("{:.*e}", n - 1, x.abs());
    let (mantissa, exp) = s.split_once('e').expect("`{:e}` always writes an exponent");
    (mantissa.replace('.', ""), exp.parse().expect("`{:e}` writes an integer exponent"))
}

/// `x` rounded to `n` significant digits, as a number — the value a reader sees, for the boundary decisions.
fn round_sig(x: f64, n: usize) -> f64 {
    format!("{:.*e}", n - 1, x).parse().expect("`{:e}` output parses back")
}

/// "-" for a negative `x` that does not round to nothing.
fn sign(x: f64) -> &'static str {
    if x < 0.0 {
        "-"
    } else {
        ""
    }
}

/// `x` at `n` significant digits in fixed notation, trailing zeros kept ("1.00", "15.0", "0.0800", "150000").
fn fixed(x: f64, n: usize) -> String {
    if !x.is_finite() {
        return x.to_string();
    }
    let (d, e) = digits_exp(x, n);
    let body = if e >= n as i32 - 1 {
        format!("{d}{}", "0".repeat((e - (n as i32 - 1)) as usize))
    } else if e >= 0 {
        let at = e as usize + 1;
        format!("{}.{}", &d[..at], &d[at..])
    } else {
        format!("0.{}{d}", "0".repeat((-e - 1) as usize))
    };
    format!("{}{body}", sign(x))
}

/// Thousands grouped by "," in the integer part of a plain number ("5770" → "5,770", "-1234.5" → "-1,234.5").
fn group(s: &str) -> String {
    let (sign, rest) = s.strip_prefix('-').map_or(("", s), |r| ("-", r));
    let int_len = rest.find('.').unwrap_or(rest.len());
    if !rest[..int_len].bytes().all(|b| b.is_ascii_digit()) {
        return s.to_string();
    }
    let mut out = String::from(sign);
    for (i, c) in rest[..int_len].chars().enumerate() {
        if i > 0 && (int_len - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out.push_str(&rest[int_len..]);
    out
}

/// `x` at `n` significant digits, thousands grouped.
fn grouped(x: f64, n: usize) -> String {
    group(&fixed(x, n))
}

/// Mass in Suns: 3 significant digits, `units.suns` ("1.00 Suns", "15.0 Suns", "0.0800 Suns").
pub fn mass(suns: f64) -> String {
    fill("units.suns", &[("value", &grouped(suns, 3))])
}

/// Temperature in kelvins: below 10⁶ K 3 significant digits with grouping (`units.kelvin`), 10⁶ to 10⁹ K
/// `units.kelvin_million`, above `units.kelvin_billion`. The unit is chosen after rounding: 999,999 K is "1.00 million K".
pub fn temperature(kelvin: f64) -> String {
    let (key, value) = if round_sig(kelvin / 1e9, 3) >= 1.0 {
        ("units.kelvin_billion", kelvin / 1e9)
    } else if round_sig(kelvin / 1e6, 3) >= 1.0 {
        ("units.kelvin_million", kelvin / 1e6)
    } else {
        ("units.kelvin", kelvin)
    };
    fill(key, &[("value", &grouped(value, 3))])
}

/// An age in years: the largest unit whose value is ≥ 1 — trillion, billion, million, thousand years, years, days,
/// seconds, milliseconds — at 3 significant digits ("4.57 billion years", "18.0 days", "3.00 milliseconds"). The unit
/// is chosen after rounding: 999.6 milliseconds is "1.00 seconds".
pub fn age(years: f64) -> String {
    let (key, size) = AGE_UNITS
        .iter()
        .find(|(_, size)| round_sig(years / size, 3) >= 1.0)
        .unwrap_or(&AGE_UNITS[AGE_UNITS.len() - 1]);
    fill(key, &[("value", &grouped(years / size, 3))])
}

/// The age rate "1 s ≈ …": years gained per wall second, in the age's units.
pub fn rate(years_per_second: f64) -> String {
    age(years_per_second)
}

/// A ladder rung as written (×0.1 … ×1000): the number alone, "0.1", "1", "1000".
pub fn rung_number(rung: f64) -> String {
    rung.to_string()
}

/// The actual speed at 2 significant digits, the number alone ("42", "0.83"); the template carries the ×.
pub fn actual_number(actual: f64) -> String {
    grouped(actual, 2)
}

/// A rung button: `time.rung`.
pub fn rung(rung: f64) -> String {
    fill("time.rung", &[("rung", &rung_number(rung))])
}

/// Max's button: `time.rung_max` (TOP is never shown, M0-TJ2).
pub fn rung_max() -> String {
    strings::get("time.rung_max").to_string()
}

/// The speed readout at a ladder rung that runs as asked: `time.speed`.
pub fn speed(rung: f64) -> String {
    fill("time.speed", &[("rung", &rung_number(rung))])
}

/// The speed readout at a rung the box cannot keep up with: `time.speed_capped`.
pub fn speed_capped(rung: f64, actual: f64) -> String {
    fill("time.speed_capped", &[("rung", &rung_number(rung)), ("actual", &actual_number(actual))])
}

/// The speed readout at Max, always: `time.speed_max`.
pub fn speed_max(actual: f64) -> String {
    fill("time.speed_max", &[("actual", &actual_number(actual))])
}

/// An inspector number: 3 significant digits; scientific ("1.23e-5") below 0.001 or at 100,000 and above, judged on
/// the rounded value (0.0009996 is "0.00100", 99,999 is "1.00e5"). Zero is "0.00".
pub fn number(x: f64) -> String {
    if !x.is_finite() {
        return x.to_string();
    }
    let rounded = round_sig(x, 3).abs();
    if rounded != 0.0 && !(1e-3..1e5).contains(&rounded) {
        let (d, e) = digits_exp(x, 3);
        format!("{}{}.{}e{e}", sign(x), &d[..1], &d[1..])
    } else {
        grouped(x, 3)
    }
}

/// A mix percentage: 3 significant digits ("73.5", "0.770").
pub fn percent(p: f64) -> String {
    grouped(p, 3)
}

/// An integer count — fps, steps per frame, cells — rounded to the nearest, thousands grouped.
pub fn integer(x: f64) -> String {
    if !x.is_finite() {
        return x.to_string();
    }
    group(&format!("{:.0}", x.round()))
}

/// `inspect.temperature`: the temperature in kelvins, translated as in `temperature`.
pub fn inspect_temperature(kelvin: f64) -> String {
    fill("inspect.temperature", &[("temperature", &temperature(kelvin))])
}

/// `inspect.density`.
pub fn inspect_density(density: f64) -> String {
    fill("inspect.density", &[("density", &number(density))])
}

/// `inspect.speed`: the speed in cells per time unit and its Mach number.
pub fn inspect_speed(speed: f64, mach: f64) -> String {
    fill("inspect.speed", &[("speed", &number(speed)), ("mach", &number(mach))])
}

/// The cell's mix: each `(element name, percent)` through `inspect.mix_item`, joined by `inspect.mix_sep`.
pub fn inspect_mix(mix: &[(&str, f64)]) -> String {
    let items: Vec<String> =
        mix.iter().map(|(element, p)| fill("inspect.mix_item", &[("element", element), ("percent", &percent(*p))])).collect();
    items.join(strings::get("inspect.mix_sep"))
}

/// `status.line`: fps and steps per frame are integers.
pub fn status_line(adapter: &str, fps: f64, steps: f64, width: u32, height: u32) -> String {
    fill(
        "status.line",
        &[
            ("adapter", adapter),
            ("fps", &integer(fps)),
            ("steps", &integer(steps)),
            ("width", &integer(f64::from(width))),
            ("height", &integer(f64::from(height))),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const YEAR_S: f64 = DAYS_PER_YEAR * SECONDS_PER_DAY;

    #[test]
    fn mass_examples() {
        assert_eq!(mass(1.0), "1.00 Suns");
        assert_eq!(mass(15.0), "15.0 Suns");
        assert_eq!(mass(0.08), "0.0800 Suns");
        assert_eq!(mass(25.0), "25.0 Suns");
        assert_eq!(mass(100.0), "100 Suns");
    }

    #[test]
    fn mass_boundaries() {
        // Rounding crosses a digit count: 9.996 → 10.0, 0.0999996 → 0.100, 999.6 → 1,000.
        assert_eq!(mass(9.996), "10.0 Suns");
        assert_eq!(mass(0.099_999_6), "0.100 Suns");
        assert_eq!(mass(999.6), "1,000 Suns");
        assert_eq!(mass(1_234_567.0), "1,230,000 Suns");
    }

    #[test]
    fn temperature_examples() {
        assert_eq!(temperature(5770.0), "5,770 K");
        assert_eq!(temperature(150_000.0), "150,000 K");
        assert_eq!(temperature(15.7e6), "15.7 million K");
        assert_eq!(temperature(2.5e9), "2.50 billion K");
        assert_eq!(temperature(10.0), "10.0 K");
    }

    #[test]
    fn temperature_boundaries_pick_the_unit_after_rounding() {
        assert_eq!(temperature(999_999.0), "1.00 million K");
        assert_eq!(temperature(999_499.0), "999,000 K");
        assert_eq!(temperature(1e6), "1.00 million K");
        assert_eq!(temperature(999.6e6), "1.00 billion K");
        assert_eq!(temperature(999.4e6), "999 million K");
        assert_eq!(temperature(1e9), "1.00 billion K");
        assert_eq!(temperature(1.5e12), "1,500 billion K");
    }

    #[test]
    fn age_examples() {
        assert_eq!(age(4.57e9), "4.57 billion years");
        assert_eq!(age(18.0 / DAYS_PER_YEAR), "18.0 days");
        assert_eq!(age(0.003 / YEAR_S), "3.00 milliseconds");
    }

    #[test]
    fn age_takes_the_largest_unit_at_or_above_one() {
        assert_eq!(age(1e12), "1.00 trillion years");
        assert_eq!(age(2.5e12), "2.50 trillion years");
        assert_eq!(age(1e9), "1.00 billion years");
        assert_eq!(age(1e6), "1.00 million years");
        assert_eq!(age(1e3), "1.00 thousand years");
        assert_eq!(age(1.0), "1.00 years");
        assert_eq!(age(1.0 / DAYS_PER_YEAR), "1.00 days");
        assert_eq!(age(1.0 / YEAR_S), "1.00 seconds");
        assert_eq!(age(0.5e-3 / YEAR_S), "0.500 milliseconds");
        assert_eq!(age(0.0), "0.00 milliseconds");
    }

    #[test]
    fn age_boundaries_pick_the_unit_after_rounding() {
        assert_eq!(age(999_600.0), "1.00 million years");
        assert_eq!(age(999_400.0), "999 thousand years");
        assert_eq!(age(364.9 / DAYS_PER_YEAR), "365 days");
        assert_eq!(age(365.25 / DAYS_PER_YEAR), "1.00 years");
        assert_eq!(age(86_399.9 / YEAR_S), "1.00 days");
        assert_eq!(age(999.6e-3 / YEAR_S), "1.00 seconds");
        assert_eq!(age(999.4e-3 / YEAR_S), "999 milliseconds");
    }

    #[test]
    fn rate_reads_as_an_age() {
        assert_eq!(rate(4.57e9), age(4.57e9));
        assert_eq!(rate(12_300.0), "12.3 thousand years");
    }

    #[test]
    fn speeds_rungs_as_written_actual_at_two_digits() {
        assert_eq!(rung(0.1), "×0.1");
        assert_eq!(rung(1.0), "×1");
        assert_eq!(rung(1000.0), "×1000");
        assert_eq!(speed(10.0), "Speed ×10");
        assert_eq!(speed_capped(100.0, 42.0), "Speed ×100 (running ×42)");
        assert_eq!(speed_capped(10.0, 0.834), "Speed ×10 (running ×0.83)");
        assert_eq!(actual_number(41.6), "42");
        assert_eq!(actual_number(99.6), "100");
        assert_eq!(speed_max(1234.0), "Speed max (running ×1,200)");
    }

    #[test]
    fn max_is_the_table_text_and_top_is_never_shown() {
        assert_eq!(rung_max(), "Max");
        assert_eq!(speed_max(42.0), "Speed max (running ×42)");
        assert!(!rung_max().contains("TOP") && !speed_max(1.0).contains("TOP"));
    }

    #[test]
    fn inspector_numbers_three_digits() {
        assert_eq!(number(12.3456), "12.3");
        assert_eq!(number(1.0), "1.00");
        assert_eq!(number(0.5), "0.500");
        assert_eq!(number(12_345.0), "12,300");
        assert_eq!(number(-2.5), "-2.50");
        assert_eq!(number(0.0), "0.00");
    }

    #[test]
    fn inspector_scientific_below_a_thousandth_and_from_a_hundred_thousand() {
        assert_eq!(number(1.23e-5), "1.23e-5");
        assert_eq!(number(0.000_999), "9.99e-4");
        assert_eq!(number(-4.5e-7), "-4.50e-7");
        assert_eq!(number(123_456.0), "1.23e5");
        assert_eq!(number(100_000.0), "1.00e5");
        assert_eq!(number(2.5e12), "2.50e12");
    }

    #[test]
    fn inspector_boundaries_are_judged_after_rounding() {
        assert_eq!(number(0.001), "0.00100");
        assert_eq!(number(0.000_999_6), "0.00100");
        assert_eq!(number(99_949.0), "99,900");
        assert_eq!(number(99_999.0), "1.00e5");
    }

    #[test]
    fn mix_percentages() {
        assert_eq!(percent(73.5), "73.5");
        assert_eq!(percent(0.77), "0.770");
        assert_eq!(percent(100.0), "100");
        assert_eq!(percent(99.96), "100");
        assert_eq!(percent(0.0), "0.00");
    }

    #[test]
    fn inspector_lines_go_through_the_templates() {
        assert_eq!(inspect_temperature(15.7e6), "Temperature: 15.7 million K");
        assert_eq!(inspect_density(0.5), "Density: 0.500 (sandbox units)");
        assert_eq!(inspect_speed(3.0, 0.0001), "Speed: 3.00 cells per time unit (Mach 1.00e-4)");
        assert_eq!(inspect_mix(&[("Hydrogen", 73.5), ("Helium", 26.5)]), "Hydrogen 73.5% · Helium 26.5%");
        assert_eq!(inspect_mix(&[("Hydrogen", 100.0)]), "Hydrogen 100%");
    }

    #[test]
    fn fps_and_steps_are_integers() {
        assert_eq!(integer(59.6), "60");
        assert_eq!(integer(0.4), "0");
        assert_eq!(integer(12_345.0), "12,345");
        assert_eq!(
            status_line("NVIDIA GeForce RTX 5090", 59.6, 12.0, 600, 400),
            "NVIDIA GeForce RTX 5090 · 60 fps · 12 steps per frame · 600 × 400 cells"
        );
    }

    #[test]
    fn grouping_is_by_thousands_in_the_integer_part_only() {
        assert_eq!(group("1"), "1");
        assert_eq!(group("999"), "999");
        assert_eq!(group("1000"), "1,000");
        assert_eq!(group("1234567"), "1,234,567");
        assert_eq!(group("-1234.5678"), "-1,234.5678");
        assert_eq!(group("NaN"), "NaN");
    }

    #[test]
    fn non_finite_values_are_shown_as_they_are() {
        assert_eq!(mass(f64::NAN), "NaN Suns");
        assert_eq!(temperature(f64::INFINITY), "inf billion K");
        assert_eq!(number(f64::NEG_INFINITY), "-inf");
        assert_eq!(integer(f64::NAN), "NaN");
    }
}
