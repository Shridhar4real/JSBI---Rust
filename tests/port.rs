use jsbi::JSBI;

#[test]
fn test_readme_example() {
    let max = JSBI::from_u64(9007199254740991); // Number.MAX_SAFE_INTEGER
    let other = JSBI::from_u64(2);
    let result = JSBI::add(&max, &other);
    assert_eq!(result.to_string(10), "9007199254740993");
    assert_eq!(result.to_number(), 9007199254740993.0);
}

#[test]
fn test_string_parsing() {
    assert_eq!(JSBI::from_str("123").unwrap().to_string(10), "123");
    assert_eq!(JSBI::from_str(" 123 ").unwrap().to_string(10), "123");
    assert_eq!(JSBI::from_str("0xFFFFFFFF").unwrap().to_string(16), "ffffffff");
    assert_eq!(JSBI::from_str("0b1010").unwrap().to_string(2), "1010");
    assert!(JSBI::from_str("x123").is_err());
    assert!(JSBI::from_str("-0x1").is_err());
}

#[test]
fn test_arithmetic_ops() {
    let a = JSBI::from_i64(100);
    let b = JSBI::from_i64(25);
    assert_eq!(JSBI::add(&a, &b), JSBI::from_i64(125));
    assert_eq!(JSBI::subtract(&a, &b), JSBI::from_i64(75));
    assert_eq!(JSBI::multiply(&a, &b), JSBI::from_i64(2500));
    assert_eq!(JSBI::divide(&a, &b).unwrap(), JSBI::from_i64(4));
    assert_eq!(JSBI::remainder(&a, &b).unwrap(), JSBI::zero());
}

#[test]
fn test_bitwise_and_shifts() {
    let a = JSBI::from_i64(0b1100);
    let b = JSBI::from_i64(0b1010);
    assert_eq!(JSBI::bitwise_and(&a, &b), JSBI::from_i64(0b1000));
    assert_eq!(JSBI::bitwise_or(&a, &b), JSBI::from_i64(0b1110));
    assert_eq!(JSBI::bitwise_xor(&a, &b), JSBI::from_i64(0b0110));

    let val = JSBI::from_i64(1);
    assert_eq!(JSBI::left_shift(&val, &JSBI::from_u64(10)), JSBI::from_i64(1024));
    assert_eq!(JSBI::signed_right_shift(&JSBI::from_i64(1024), &JSBI::from_u64(10)), JSBI::from_i64(1));
}

#[test]
fn test_as_int_n_and_as_uint_n() {
    let x = JSBI::from_i64(15);
    assert_eq!(JSBI::as_int_n(3, &x).to_string(10), "-1");
    assert_eq!(JSBI::as_uint_n(3, &x).to_string(10), "7");
}

#[test]
fn test_dataview() {
    let mut buf = [0u8; 8];
    let val = JSBI::from_i64(0x1234567890abcdef);
    JSBI::data_view_set_big_int64(&mut buf, &val, true).unwrap();
    let read_back = JSBI::data_view_get_big_int64(&buf, true).unwrap();
    assert_eq!(read_back.to_string(16), "1234567890abcdef");
}
