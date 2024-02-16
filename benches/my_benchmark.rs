use cosmic_system::simulation::{create_bodies, update_bodies};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let (bodies, bodies_forces, _) = create_bodies(10001);
    let bounding_box = cosmic_system::bounding_box::BoundingBox::new(
        glam::DVec3::ONE * -4.0 * cosmic_system::simulation::AU,
        glam::DVec3::ONE * 4.0 * cosmic_system::simulation::AU,
    );
    c.bench_function("update_bodies", |b| {
        b.iter(|| {
            update_bodies(
                black_box(bounding_box),
                black_box(&mut bodies.clone()),
                black_box(&mut bodies_forces.clone()),
            );
        })
    });
}

pub fn z_order_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Z Order");
    let bounding_box = cosmic_system::bounding_box::BoundingBox::new(
        glam::DVec3::ONE * -4.0 * cosmic_system::simulation::AU,
        glam::DVec3::ONE * 4.0 * cosmic_system::simulation::AU,
    );
    let position = glam::DVec3::new(0., 0.5, 0.75);
    group.bench_function("z_order_curve", |b| {
        b.iter(|| {
            cosmic_system::z_order::z_order_curve(black_box(position), black_box(&bounding_box));
        })
    });
    group.bench_function("z_order_curve_alt", |b| {
        b.iter(|| {
            cosmic_system::z_order::_z_order_curve_slow(
                black_box(position),
                black_box(&bounding_box),
            );
        })
    });
    group.finish();
}

criterion_group!(benches, criterion_benchmark, z_order_benchmark);
criterion_main!(benches);
