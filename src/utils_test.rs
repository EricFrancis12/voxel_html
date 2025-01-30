use crate::utils::is_whitespace;

#[test]
fn test_is_whitespace() {
    for s in ["", "\n", "\n "] {
        assert_eq!(is_whitespace(s), true);
    }
}
