pub fn is_whitespace(s: impl Into<String>) -> bool {
    s.into().trim().is_empty()
}
