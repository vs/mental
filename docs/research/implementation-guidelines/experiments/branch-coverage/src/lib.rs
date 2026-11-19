pub fn short_assert(x: u32) -> u32 {
    assert!(x < 100, "x is below 100");
    x + 1
}

pub fn short_debug_assert(x: u32) -> u32 {
    debug_assert!(x < 100, "x is below 100");
    x + 2
}

pub fn short_assert_eq(x: u32) -> u32 {
    assert_eq!(x % 2, 0, "x is even");
    x + 3
}

pub fn expect_on_one_line(s: &str) -> u32 {
    s.parse::<u32>().expect("the caller passes digits") + 4
}

pub fn question_mark_on_one_line(s: &str) -> Result<u32, std::num::ParseIntError> {
    let n: u32 = s.parse()?;
    Ok(n + 5)
}

pub fn if_else(x: u32) -> u32 {
    if x < 100 { x + 6 } else { 0 }
}

pub fn multi_line_asserts(items: &[u32], limit: u32) -> u32 {
    assert!(
        items.iter().all(|item| *item < limit),
        "every item is below the limit {limit}, got {items:?} with {} items",
        items.len()
    );
    debug_assert_eq!(
        items.len() % 2,
        0,
        "an even number of items, got {}",
        items.len()
    );
    items.iter().sum()
}

pub fn question_mark_on_its_own_line(items: &[u32]) -> Result<u32, std::num::TryFromIntError> {
    let total = items
        .iter()
        .map(|item| u16::try_from(*item))
        .sum::<Result<u16, _>>()?;
    Ok(u32::from(total))
}

#[cfg(test)]
mod tests;
