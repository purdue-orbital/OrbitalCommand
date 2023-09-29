use radio::dsp::ecc::viterbi::common::{BIT_MASK, combine, map_to, squish, state_to_bit, stretch};
use radio::dsp::ecc::viterbi::decode::single_bit_decode::Link;
use radio::dsp::ecc::viterbi::prelude::{DecoderState, EncoderState};

#[test]
fn test_squish() {
    assert_eq!(squish(0), 0);

    for i in 1u8..=255 {
        assert_eq!(squish(i), 1);
    }
}

#[test]
fn test_stretch() {
    assert_eq!(stretch(0), 0);

    for i in 1u8..=255 {
        assert_eq!(stretch(i), 0xFF);
    }
}

#[test]
fn test_map_to() {
    for x in 0..=255 {
        assert_eq!(map_to(0, x), 0);

        for i in 1u8..=255 {
            assert_eq!(map_to(i, x), x);
        }
    }
}

#[test]
fn test_combine() {
    for x in BIT_MASK {
        assert_eq!(combine(0, 0), 0);
        assert_eq!(combine(x, 0), 1);
        assert_eq!(combine(0, x), 2);
        assert_eq!(combine(x, x), 3);
    }
}

#[test]
fn test_bit_mask() {
    for i in 0..8 {
        assert_eq!(1 << i, BIT_MASK[i])
    }
}

fn test_state_to_bit() {
    for state in 0..4 {
        for bit in BIT_MASK {
            let x = match state {
                0 => 0,
                1 => bit,
                2 => 0,
                3 => bit,
                _ => unreachable!(),
            };

            assert_eq!(x, state_to_bit(state, bit))
        }
    }
}

#[test]
fn test_round_trip_1() {
    let bytes = vec![0xFF, 0x10, 0x00];

    let mut encoder: EncoderState<u8> = EncoderState::default();
    let data_encoded = encoder.push_slice(&bytes);

    let mut decoder = DecoderState::new(bytes.len());
    decoder.push_slice(&data_encoded);
    let output = decoder.read();

    assert_eq!(bytes, output);
}

#[test]
fn test_round_trip_full_00() {
    let bytes = vec![0x00; 127];

    let mut encoder: EncoderState<u8> = EncoderState::default();
    let data_encoded = encoder.push_slice(&bytes);

    let mut decoder = DecoderState::new(bytes.len());
    decoder.push_slice(&data_encoded);
    let output = decoder.read();

    assert_eq!(bytes, output);
}

#[test]
fn test_round_trip_full_ff() {
    let bytes = vec![0xFF; 127];

    let mut encoder: EncoderState<u8> = EncoderState::default();
    let data_encoded = encoder.push_slice(&bytes);

    let mut decoder = DecoderState::new(bytes.len());
    decoder.push_slice(&data_encoded);
    let output = decoder.read();

    assert_eq!(bytes, output);
}

#[test]
fn test_round_trip_ff_ff_00_00() {
    let bytes = vec![0xFF, 0xFF, 0x00, 0x00];

    let mut encoder: EncoderState<u8> = EncoderState::default();
    let data_encoded = encoder.push_slice(&bytes);

    let mut decoder = DecoderState::new(bytes.len());
    decoder.push_slice(&data_encoded);
    let output = decoder.read();

    assert_eq!(bytes, output);
}

#[test]
fn test_round_trip_ff_00_00_00_00() {
    let bytes = vec![0xFF, 0x00, 0x00, 0x00, 0x00];

    let mut encoder: EncoderState<u8> = EncoderState::default();
    let data_encoded = encoder.push_slice(&bytes);

    let mut decoder = DecoderState::new(bytes.len());
    decoder.push_slice(&data_encoded);
    let output = decoder.read();

    assert_eq!(bytes, output);
}

#[test]
fn test_round_trip_all_3_bit_sequences() {
    let bytes = vec![
        0b11110000,
        0b11001100,
        0b10101010,
    ];

    let mut encoder: EncoderState<u8> = EncoderState::default();
    let data_encoded = encoder.push_slice(&bytes);

    let mut decoder = DecoderState::new(bytes.len());
    decoder.push_slice(&data_encoded);

    let output = decoder.read();

    assert_eq!(bytes, output);
}

#[test]
fn test_round_trip_staircase() {
    let bytes = vec![
        0b10000000,
        0b11000000,
        0b11100000,
        0b11110000,
        0b11111000,
        0b11111100,
        0b11111110,
        0b11111111,
    ];

    let mut encoder: EncoderState<u8> = EncoderState::default();
    let data_encoded = encoder.push_slice(&bytes);

    let mut decoder = DecoderState::new(bytes.len());
    decoder.push_slice(&data_encoded);
    let output = decoder.read();

    assert_eq!(bytes, output);
}

#[test]
fn test_round_trip_inverted_staircase() {
    let bytes = vec![
        0b11111111,
        0b01111111,
        0b00111111,
        0b00011111,
        0b00001111,
        0b00000111,
        0b00000011,
        0b00000001,
    ];

    let mut encoder: EncoderState<u8> = EncoderState::default();
    let data_encoded = encoder.push_slice(&bytes);

    let mut decoder = DecoderState::new(bytes.len());
    decoder.push_slice(&data_encoded);
    let output = decoder.read();

    assert_eq!(bytes, output);
}


fn state_eq(state: &EncoderState<u8>, correct: u8) {
    let x: EncoderState<u8> = correct.into();
    assert_eq!(state.0, x.0);
    assert_eq!(state.1, x.1);
}

#[test]
fn test_state_updating() {
    let mut state = EncoderState::<u8>::default();

    state.push(0x00);
    state_eq(&state, 0);

    state.push(0xFF);
    state_eq(&state, 1);

    state.push(0xFF);
    state_eq(&state, 3);
    state = 1.into();

    state.push(0x00);
    state_eq(&state, 2);

    state.push(0x00);
    state_eq(&state, 0);
    state = 2.into();

    state.push(0xFF);
    state_eq(&state, 1);
    state = 3.into();

    state.push(0x00);
    state_eq(&state, 2);
    state = 3.into();

    state.push(0xFF);
    state_eq(&state, 3);
}

#[test]
fn test_encoder_ouptut() {
    let mut state: EncoderState<u8> = EncoderState::default();

    let pair = state.push(0xFF);
    assert_eq!(pair, (0xFF, 0xFF));
}

#[test]
fn test_to_from_encoder_state() {
    for x in 0u8..4 {
        let state: EncoderState<u8> = x.into();
        assert_eq!(x, <EncoderState<u8> as Into<u8>>::into(state));
    }
}

#[test]
fn test_from_u8() {
    let arr_a: [EncoderState<u8>; 5] = [
        0.into(),
        1.into(),
        2.into(),
        3.into(),
        7.into(),
    ];

    let arr_b: [EncoderState<u8>; 5] = [
        EncoderState(0, 0),
        EncoderState(0xFF, 0),
        EncoderState(0, 0xFF),
        EncoderState(0xFF, 0xFF),
        EncoderState(0, 0),
    ];

    for (index, x) in arr_b.iter().enumerate() {
        assert_eq!(x.0, arr_b[index].0);
        assert_eq!(x.1, arr_b[index].1);
    }
}

#[test]
fn test_to_u8() {
    let arr_a: [u8; 4] = [
        EncoderState(0, 0).into(),
        EncoderState(0xFF, 0).into(),
        EncoderState(0, 0xFF).into(),
        EncoderState(0xFF, 0xFF).into(),
    ];

    let arr_b: [u8; 4] = [0, 1, 2, 3];

    assert_eq!(arr_a, arr_b);
}

#[test]
fn test_next_link() {
    let arr = Link::next(1, 2, 0);

    assert_eq!(arr[0].0.prev_state, 1);
    assert_eq!(arr[0].0.cost, 0);

    assert_eq!(arr[1].0.prev_state, 1);
    assert_eq!(arr[1].0.cost, 2);
}

#[test]
fn test_generate_link() {
    let (link_0, _) = Link::generate(1, 2, 0, 1);

    assert_eq!(link_0.prev_state, 1);
    assert_eq!(link_0.cost, 2);
}

#[test]
fn test_hamming_distance() {
    assert_eq!(Link::hamming_dist(255, 0), 8);
    assert_eq!(Link::hamming_dist(1, 0), 1);
    assert_eq!(Link::hamming_dist(2, 0), 1);
    assert_eq!(Link::hamming_dist(2, 1), 2);
    assert_eq!(Link::hamming_dist(0, 0), 0);
    assert_eq!(Link::hamming_dist(255, 255), 0);
    assert_eq!(Link::hamming_dist(0b00010101, 0b00000100), 2);
}

#[test]
fn test_minimize_cost() {
    let mut link_a = Link {
        prev_state: 0,
        cost: 10,
    };

    let link_b = Link {
        prev_state: 1,
        cost: 11,
    };

    link_a.minimize_cost(link_b);

    assert_eq!(link_a.prev_state, 0);
    assert_eq!(link_a.cost, 10);

    let link_c = Link {
        prev_state: 2,
        cost: 9,
    };

    link_a.minimize_cost(link_c);

    assert_eq!(link_a.prev_state, 2);
    assert_eq!(link_a.cost, 9);
}