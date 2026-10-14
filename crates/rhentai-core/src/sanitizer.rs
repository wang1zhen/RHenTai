pub fn sanitize_title(title: &str) -> String {
    // Replace `<>:"/\|?*` and control characters with a space
    let s: String = title
        .chars()
        .map(|c| {
            if c.is_control() || matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') {
                ' '
            } else {
                c
            }
        })
        .collect();

    // Collapse whitespace
    let mut collapsed = String::with_capacity(s.len());
    let mut last_was_ws = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !last_was_ws {
                collapsed.push(' ');
                last_was_ws = true;
            }
        } else {
            collapsed.push(c);
            last_was_ws = false;
        }
    }

    // Trim trailing dots and spaces
    let mut trimmed = collapsed.trim().trim_end_matches(['.', ' ']).to_string();

    // Cap at 150 bytes on a UTF-8 boundary
    if trimmed.len() > 150 {
        let mut end = 150;
        while !trimmed.is_char_boundary(end) {
            end -= 1;
        }
        trimmed.truncate(end);
        trimmed = trimmed.trim_end_matches(['.', ' ']).to_string();
    }

    if trimmed.is_empty() {
        "untitled".to_string()
    } else {
        trimmed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitizer_illegal_chars() {
        assert_eq!(
            sanitize_title(r#"a<b>c:d"e/f\g|h?i*j"#),
            "a b c d e f g h i j"
        );
    }

    #[test]
    fn test_sanitizer_control_chars() {
        assert_eq!(sanitize_title("Hello\x00\x07World\n!"), "Hello World !");
    }

    #[test]
    fn test_sanitizer_collapse_whitespace() {
        assert_eq!(sanitize_title("   Hello    world   "), "Hello world");
    }

    #[test]
    fn test_sanitizer_trailing_dots_and_spaces() {
        assert_eq!(sanitize_title("Title with dots...   "), "Title with dots");
        assert_eq!(sanitize_title("Dots and spaces. . ."), "Dots and spaces");
    }

    #[test]
    fn test_sanitizer_empty_and_all_illegal() {
        assert_eq!(sanitize_title(""), "untitled");
        assert_eq!(sanitize_title("???:::***"), "untitled");
        assert_eq!(sanitize_title("..."), "untitled");
    }

    #[test]
    fn test_sanitizer_cap_150_bytes_utf8() {
        // Japanese characters: 3 bytes each
        let jp_title = "あ".repeat(60); // 180 bytes
        let sanitized = sanitize_title(&jp_title);
        assert!(sanitized.len() <= 150);
        assert_eq!(sanitized.len(), 150); // 50 * 3 = 150 bytes exactly
        assert_eq!(sanitized, "あ".repeat(50));

        // Mixed 1-byte and multi-byte:
        let mixed = format!("{}あ", "a".repeat(149)); // 149 bytes of 'a' + 3 bytes 'あ' = 152 bytes
        let sanitized_mixed = sanitize_title(&mixed);
        assert!(sanitized_mixed.len() <= 150);
        assert_eq!(sanitized_mixed.len(), 149);
    }
}
