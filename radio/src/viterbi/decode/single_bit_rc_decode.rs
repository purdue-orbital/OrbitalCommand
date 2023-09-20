#![allow(dead_code)]

use crate::common::*;
use crate::encode::EncoderState;

use std::rc::Rc;

#[derive(Debug)]
pub struct BitDecoderState {
	end_links: [Option<RcLink>; 4],
	bit: u8,
	len: usize, // should it be a u32?
}

impl BitDecoderState {
	const EMPTY: [Option<RcLink>; 4] = [None, None, None, None];

	/// create a new decoder for a single bit.
	pub fn new(bit: u8) -> Self {
		Self {
			end_links: Self::EMPTY,
			bit,
			len: 0
		}
	}

	fn is_empty(&self) -> bool {
		self.end_links == Self::EMPTY
	}

	/// push a pair of bits to be decoded
	///
	/// takes u8s instead of bools for conveince (just do a bitwise and between the mask and the byte)
	pub fn push(&mut self, s0: u8, s1: u8) {
		let bit_pair = combine(s0, s1);

		let mut new_endlinks = Self::EMPTY;

		if !self.is_empty() {
			let mut link_vec = Vec::with_capacity(8);
			for each in &self.end_links {
				if let Some(link) = each {
					let link_pair = Link::next_links(link, bit_pair, self.bit);
					link_vec.extend_from_slice(&link_pair);
				}
			}

			let mut min_costs = [u32::MAX; 4];

			for each in link_vec {
				let index = each.position();
				if each.cost < min_costs[index] {
					min_costs[index] = each.cost;
					new_endlinks[index] = Some(each.to_rc_link());
				}
			}
		} else {
			for each in Link::first_links(bit_pair, self.bit) {
				let index = each.position();
				new_endlinks[index] = Some(each.to_rc_link());
			}
		}

		self.end_links = new_endlinks;
		self.len += 1;
	}

	// TODO: optimize
	pub fn read(&self) -> Vec<u8> {
		let mut ans = vec![0; self.len];

		let mut link = self.end_links.iter()
			.min_by_key(|link| link.as_ref().unwrap().cost)
			.unwrap().clone().unwrap();

		for backwards_index in 1..=self.len {
			let i = self.len - backwards_index;
			ans[i] = link.bit;
			link = link.prev_link.clone().unwrap_or_else(|| Link::DEAD_LINK.to_rc_link());
		}

		ans
	}
}

type RcLink = Rc<Link>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Link {
	pub prev_link: Option<RcLink>,
	pub bit: u8,
	pub state: u8, // should I remove this? it is usefull for debuging and doesn't cost more memory
	pub cost: u32, // enough for just over 2 gigabytes/gibibytes
}

impl Link {
	/// reads the bits of self into the slice given, combining using a bitwise or.
	/// it starts at the end and works towards the start
	/// NOTE: it must be long enough
	// pub fn read_into(&self, arr: &mut [u8]) {
	// 	let mut next = self.link_to();
	// 	let mut i = arr.len();

	// 	while let Some(link) = next {
	// 		i -= 1;

	// 		arr[i] |= link.bit;
	// 		next = link.prev_link;
	// 	}
	// }

	pub fn first_links(bit_pair: u8, bit: u8) -> [Self; 2] {
		let starting_link = Self::DEAD_LINK.to_rc_link();

		let mut links = [
			Self::new_link(&starting_link, bit_pair, bit),
			Self::new_link(&starting_link, bit_pair, 0),
		];

		links[0].remove_prev_link();
		links[1].remove_prev_link();

		links
	}

	pub fn next_links(link: &RcLink, bit_pair: u8, bit: u8) -> [Self; 2] {
		[
			Self::new_link(link, bit_pair, bit),
			Self::new_link(link, bit_pair, 0),
		]
	}

	fn new_link(link: &RcLink, bit_pair: u8, bit: u8) -> Self {
		// TODO: see if making this use a seperate, more simple encoder (a lookup table?) would be faster
		// create an encoder to figure stuff out
		let mut encoder: EncoderState<u8> = link.encoder();
		let hypothetical_bit_pair = encoder.push_return_bitpair(stretch(bit));

		Self {
			prev_link: Self::link_to(link),
			bit,
			state: encoder.into(),
			cost: link.cost + Self::hamming_dist(bit_pair, hypothetical_bit_pair),
		}
	}

	#[inline]
	pub fn link_to(link: &RcLink) -> Option<RcLink> {
		Some(Rc::clone(link))
	}

	#[inline]
	pub fn to_rc_link(self) -> RcLink {
		Rc::new(self)
	}

	const DEAD_LINK: Self = Self {
		prev_link: None,
		bit: 0,
		state: 0,
		cost: 0,
	};

	#[inline]
	fn remove_prev_link(&mut self) {
		self.prev_link = None;
	}

	#[inline]
	fn encoder(&self) -> EncoderState<u8> {
		self.state.into()
	}

	/// get the index where this link belongs in the `end_links` array
	#[inline]
	pub fn position(&self) -> usize {
		self.state as usize
	}

	#[inline]
	fn hamming_dist(a: u8, b: u8) -> u32 {
		(a ^ b).count_ones() as u32
	}
}

#[cfg(test)]
mod tests {
	use super::*;

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
}
