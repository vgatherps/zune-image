//! Benchmarks for
#![allow(clippy::field_reassign_with_default)]

use std::fs::read;
use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use zune_core::options::DecoderOptions;
use zune_jpeg::JpegDecoder;

fn decode_jpeg(buf: &[u8], options: DecoderOptions) -> Vec<u8>
{
    let mut d = JpegDecoder::new_with_options(options, buf);

    d.decode().unwrap()
}

fn decode_no_samp(c: &mut Criterion)
{
    let a = env!("CARGO_MANIFEST_DIR").to_string() + "/benches/images/speed_bench.jpg";

    let data = read(a).unwrap();
    let mut group = c.benchmark_group("jpeg: zune-jpeg Intrinsics");

    group.throughput(Throughput::Bytes(data.len() as u64));

    group.bench_function("intrinsics", |b| {
        b.iter(|| {
            let opt = DecoderOptions::default();
            black_box(decode_jpeg(data.as_slice(), opt));
        })
    });
    group.bench_function("no intrinsics", |b| {
        b.iter(|| {
            let opt = DecoderOptions::default().set_use_unsafe(false);
            black_box(decode_jpeg(data.as_slice(), opt));
        })
    });
}
criterion_group!(name=benches;
      config={
      let c = Criterion::default();
        c.measurement_time(Duration::from_secs(20))
      };
    targets=decode_no_samp);

criterion_main!(benches);
