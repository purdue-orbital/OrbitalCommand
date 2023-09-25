// master bench file

use criterion::{criterion_group, criterion_main};

use ask_bench::ask_benchmark;
use bpsk_bench::bpsk_benchmark;
use fsk_bench::fsk_benchmark;
use qpsk_bench::qpsk_benchmark;

// ask benchmarks
mod ask_bench;

// fsk benchmarks
mod fsk_bench;

// mfsk benchmarks
mod mfsk_bench;

// bpsk benchmarks
mod bpsk_bench;

// qpsk benchmarks
mod qpsk_bench;

criterion_group!(fsk, fsk_benchmark);
criterion_group!(ask, ask_benchmark);
criterion_group!(bpsk, bpsk_benchmark);
criterion_group!(qpsk, qpsk_benchmark);

#[cfg(feature = "qpsk")]
criterion_main!(qpsk);

#[cfg(feature = "bpsk")]
criterion_main!(bpsk);

#[cfg(feature = "fsk")]
criterion_main!(fsk);

#[cfg(feature = "ask")]
criterion_main!(ask);