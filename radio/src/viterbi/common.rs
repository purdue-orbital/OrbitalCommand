#![allow(dead_code)]

#[inline]
/// combine s1 and s0 into the lower 2 bits of a u8
pub fn combine(s0: u8, s1: u8) -> u8 {
	map_to(s1, BIT_MASK[1]) | squish(s0)
}

pub const BIT_MASK: [u8; 8] = [
	1,
	1 << 1,
	1 << 2,
	1 << 3,
	1 << 4,
	1 << 5,
	1 << 6,
	1 << 7
];

#[inline]
/// Any input >= 1 becomes the desired u8
pub fn map_to(num: u8, desired: u8) -> u8 {
	match num {
		0 => 0,
		_ => desired,
	}
}

#[inline]
/// Any input >= 1 becomes 1
pub fn squish(num: u8) -> u8 {
	map_to(num, 1)
}

#[inline]
/// Any input >= 1 becomes 255 (aka `0b11111111`)
pub fn stretch(num: u8) -> u8 {
	map_to(num, 0xFF)
}

pub fn eprint_bin(arr: &[u8], tag: Option<&str>) {
	if let Some(s) = tag {
		eprintln!("{}:", s);
	}
	for b in arr {
		eprintln!("{:#010b}, {b:#3}", b);
	}
	eprintln!();
}

pub fn eprint_diff(arr1: &[u8], arr2: &[u8]) {
	let diff: Vec<u8> = arr1.iter().zip(arr2)
		.map(|x| {
			x.0 ^ x.1
		}).collect();

	eprint_bin(&diff, Some("diff"));
}

pub fn eprint_bytes_masked(arr: &[u8], mask: u8) {
	for b in arr {
		eprintln!("{}", b & mask);
	}
}

pub fn state_to_bit(state: u8, bit: u8) -> u8 {
	debug_assert!(state < 4);
	map_to(state & BIT_MASK[0], bit)
}

#[cfg(test)]
mod tests {
	use super::*;

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
}