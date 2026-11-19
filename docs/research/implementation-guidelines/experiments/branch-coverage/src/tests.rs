use super::*;

#[test]
fn every_function_on_its_happy_path_only() {
    assert_eq!(short_assert(1), 2);
    assert_eq!(short_debug_assert(1), 3);
    assert_eq!(short_assert_eq(2), 5);
    assert_eq!(expect_on_one_line("1"), 5);
    assert_eq!(question_mark_on_one_line("1"), Ok(6));
    assert_eq!(if_else(1), 7);
    assert_eq!(multi_line_asserts(&[1, 2], 10), 3);
    assert_eq!(question_mark_on_its_own_line(&[1, 2]), Ok(3));
}
