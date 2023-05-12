// master bench file

use criterion::{criterion_group, criterion_main};

use ask_bench::ask_benchmark;
use fsk_bench::fsk_benchmark;
use mfsk_bench::mfsk_benchmark;

// ask benchmarks
mod ask_bench;

// fsk benchmarks
mod fsk_bench;

// mfsk benchmarks
mod mfsk_bench;

criterion_group!(mfsk, mfsk_benchmark);
criterion_group!(fsk, fsk_benchmark);
criterion_group!(ask, ask_benchmark);

#[cfg(feature = "mfsk")]
criterion_main!(mfsk);

#[cfg(feature = "fsk")]
criterion_main!(fsk);

#[cfg(feature = "ask")]
criterion_main!(ask);