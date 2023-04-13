use criterion::{
	black_box, criterion_group, criterion_main, AxisScale, BatchSize, Criterion, PlotConfiguration,
};
use ndarray::prelude::*;
use ndarray_slice::Slice1Ext;
use rand::prelude::*;

fn select_nth_unstable(c: &mut Criterion) {
	let lens = vec![10, 100, 1_000, 10_000, 100_000];
	let mut group = c.benchmark_group("select_nth_unstable");
	group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
	for len in &lens {
		group.bench_with_input(format!("{}", len), len, |b, &len| {
			let mut rng = StdRng::seed_from_u64(42);
			let mut data: Vec<_> = (0..len).collect();
			data.shuffle(&mut rng);
			let indices: Vec<_> = (0..len).step_by(len / 10).collect();
			b.iter_batched(
				|| Array1::from(data.clone()),
				|mut arr| {
					for &i in &indices {
						black_box(arr.select_nth_unstable(i));
					}
				},
				BatchSize::SmallInput,
			)
		});
	}
	group.finish();
}

fn select_many_nth_unstable(c: &mut Criterion) {
	let lens = vec![10, 100, 1_000, 10_000, 100_000];
	let mut group = c.benchmark_group("select_many_nth_unstable");
	group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
	for len in &lens {
		group.bench_with_input(format!("{}", len), len, |b, &len| {
			let mut rng = StdRng::seed_from_u64(42);
			let mut data: Vec<_> = (0..len).collect();
			data.shuffle(&mut rng);
			let indices: Array1<_> = (0..len).step_by(len / 10).collect();
			b.iter_batched(
				|| Array1::from(data.clone()),
				|mut arr| {
					let mut values = Vec::with_capacity(indices.len());
					arr.select_many_nth_unstable(&indices, &mut values);
					black_box(());
				},
				BatchSize::SmallInput,
			)
		});
	}
	group.finish();
}

#[cfg(feature = "rayon")]
fn par_select_many_nth_unstable(c: &mut Criterion) {
	let lens = vec![10, 100, 1_000, 10_000, 100_000];
	let mut group = c.benchmark_group("par_select_many_nth_unstable");
	group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));
	for len in &lens {
		group.bench_with_input(format!("{}", len), len, |b, &len| {
			let mut rng = StdRng::seed_from_u64(42);
			let mut data: Vec<_> = (0..len).collect();
			data.shuffle(&mut rng);
			let indices: Array1<_> = (0..len).step_by(len / 10).collect();
			b.iter_batched(
				|| Array1::from(data.clone()),
				|mut arr| {
					let mut values = Vec::with_capacity(indices.len());
					arr.par_select_many_nth_unstable(&indices, &mut values);
					black_box(());
				},
				BatchSize::SmallInput,
			)
		});
	}
	group.finish();
}

#[cfg(not(feature = "rayon"))]
criterion_group! {
	name = benches;
	config = Criterion::default();
	targets = select_nth_unstable, select_many_nth_unstable
}
#[cfg(feature = "rayon")]
criterion_group! {
	name = benches;
	config = Criterion::default();
	targets = select_nth_unstable, select_many_nth_unstable, par_select_many_nth_unstable
}
criterion_main!(benches);
