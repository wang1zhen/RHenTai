/// Parse a UTC date string in the format "YYYY-MM-DD HH:MM" into unix timestamp in seconds.
pub fn parse_eh_date(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.len() < 16 {
        return None;
    }
    let parts: Vec<&str> = s[..16].split(['-', ' ', ':']).collect();
    if parts.len() != 5 {
        return None;
    }
    let year: i64 = parts[0].parse().ok()?;
    let month: i64 = parts[1].parse().ok()?;
    let day: i64 = parts[2].parse().ok()?;
    let hour: i64 = parts[3].parse().ok()?;
    let minute: i64 = parts[4].parse().ok()?;

    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || !(0..=23).contains(&hour)
        || !(0..=59).contains(&minute)
    {
        return None;
    }

    // Howard Hinnant algorithm for civil days from Unix epoch 1970-01-01
    let y = year - if month <= 2 { 1 } else { 0 };
    let era = (if y >= 0 { y } else { y - 399 }) / 400;
    let yoe = y - era * 400;
    let m_adj = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * m_adj + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe - 719468;

    let seconds = days * 86400 + hour * 3600 + minute * 60;
    Some(seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_eh_date() {
        assert_eq!(parse_eh_date("1970-01-01 00:00"), Some(0));
        assert_eq!(parse_eh_date("1970-01-01 01:00"), Some(3600));
        assert_eq!(parse_eh_date("2026-10-14 10:00"), Some(1791972000));
        assert_eq!(parse_eh_date("invalid"), None);
    }
}
