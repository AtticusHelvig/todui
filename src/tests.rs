use crate::widget::*;
use ratatui::layout::Rect;

#[test]
fn get_cursor_pos() {
    let area = Rect {
        x: 1,
        y: 1,
        width: 5,
        height: 5,
    };
    let input = InputField::new(String::from("A wrap occurs"), Wrap::Word);
    assert_eq!(input.get_cursor_at(area, 0), (1, 1));
    assert_eq!(input.get_cursor_at(area, 6), (5, 2));
    assert_eq!(input.get_cursor_at(area, 12), (1, 4));
    assert_eq!(input.get_cursor_at(area, usize::MAX), (1, 4));
    let input = InputField::new(String::from(""), Wrap::Word);
    assert_eq!(input.get_cursor_at(area, 1), (1, 1));
}
