use regex::Regex;
use std::sync::LazyLock;

static IR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"background-position:\s*(-?\d+)px\s+(-?\d+)px").expect("valid regex")
});

static RATING_LABEL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"Average:\s*([0-9.]+)").expect("valid regex"));

/// Compute rating (0.0..=5.0) from the .ir element style:
/// background-position:-Xpx -Ypx -> 5 - X/16 - (Y == 21 ? 0.5 : 0)
pub fn parse_ir_rating(style: &str) -> Option<f32> {
    let caps = IR_REGEX.captures(style)?;
    let x: f32 = caps[1].parse().ok()?;
    let y: f32 = caps[2].parse().ok()?;

    // x is negative or zero, e.g. -16px. We want positive X.
    let x_pos = (-x).max(0.0);
    let y_pos = (-y).max(0.0);

    let half = if (y_pos - 21.0).abs() < 1.0 { 0.5 } else { 0.0 };
    let rating = 5.0 - (x_pos / 16.0) - half;
    Some(rating.clamp(0.0, 5.0))
}

/// Parse rating from #rating_label text ("Average: 3.70" or "Not Yet Rated")
pub fn parse_rating_label(text: &str) -> f32 {
    if let Some(caps) = RATING_LABEL_REGEX.captures(text)
        && let Ok(v) = caps[1].parse::<f32>()
    {
        return v.clamp(0.0, 5.0);
    }
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ir_rating() {
        // 5 stars: 0px -1px
        assert!((parse_ir_rating("background-position:0px -1px").unwrap() - 5.0).abs() < 0.01);
        // 4.5 stars: 0px -21px
        assert!((parse_ir_rating("background-position:0px -21px").unwrap() - 4.5).abs() < 0.01);
        // 4 stars: -16px -1px
        assert!((parse_ir_rating("background-position:-16px -1px").unwrap() - 4.0).abs() < 0.01);
        // 3.5 stars: -16px -21px
        assert!((parse_ir_rating("background-position:-16px -21px").unwrap() - 3.5).abs() < 0.01);
        // 0 stars: -80px -1px
        assert!((parse_ir_rating("background-position:-80px -1px").unwrap() - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_rating_label() {
        assert!((parse_rating_label("Average: 3.70") - 3.70).abs() < 0.01);
        assert_eq!(parse_rating_label("Not Yet Rated"), 0.0);
    }
}
