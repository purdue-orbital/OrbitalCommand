// master bench file

use criterion::{criterion_group, criterion_main};

// ask benchmarks
mod ask_bench;
use ask_bench::ask_benchmark;

// fsk benchmarks
mod fsk_bench;
use fsk_bench::fsk_benchmark;

// mfsk benchmarks
mod mfsk_bench;
use mfsk_bench::mfsk_benchmark;

criterion_group!(mfsk, mfsk_benchmark);
criterion_group!(fsk, fsk_benchmark);
criterion_group!(ask, ask_benchmark);
criterion_main!(mfsk,fsk,ask);