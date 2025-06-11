use crate::widget::*;

#[test]
fn get_cursor_pos() {
    let input = InputField::new("A wrap occurs".to_string(), (5, 5));
    // "A "
    // "wrap "
    // "occur"
    // "s"
    assert_eq!(input.get_cursor_at(0), (0, 0));
    assert_eq!(input.get_cursor_at(6), (4, 1));
    assert_eq!(input.get_cursor_at(12), (0, 3));
}
