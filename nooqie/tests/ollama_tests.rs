#![cfg(test)]

use nooqie::commands::*;

#[test]
fn test_fail() {
    assert_eq!(1, 0);
}
