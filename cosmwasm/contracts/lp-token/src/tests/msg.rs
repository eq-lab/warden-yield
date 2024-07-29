use crate::msg::InstantiateMsg;

#[test]
fn validate_instantiate_msg_name() {
    // Too short
    let mut msg = InstantiateMsg {
        name: str::repeat("a", 2),
        ..InstantiateMsg::default()
    };
    assert!(!msg.has_valid_name());

    // In the correct length range
    msg.name = str::repeat("a", 3);
    assert!(msg.has_valid_name());

    // Too long
    msg.name = str::repeat("a", 51);
    assert!(!msg.has_valid_name());
}

#[test]
fn validate_instantiate_msg_symbol() {
    // Too short
    let mut msg = InstantiateMsg {
        symbol: str::repeat("a", 2),
        ..InstantiateMsg::default()
    };
    assert!(!msg.has_valid_symbol());

    // In the correct length range
    msg.symbol = str::repeat("a", 3);
    assert!(msg.has_valid_symbol());

    // Too long
    msg.symbol = str::repeat("a", 13);
    assert!(!msg.has_valid_symbol());

    // Has illegal char
    let illegal_chars = [[64u8], [91u8], [123u8]];
    illegal_chars.iter().for_each(|c| {
        let c = std::str::from_utf8(c).unwrap();
        msg.symbol = str::repeat(c, 3);
        assert!(!msg.has_valid_symbol());
    });
}
