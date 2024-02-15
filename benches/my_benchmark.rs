use cosmic_system::simulation::{create_bodies, update_bodies};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn criterion_benchmark(c: &mut Criterion) {
    let (bodies, _) = create_bodies(10001);
    let bounding_box = cosmic_system::bounding_box::BoundingBox::new(
        glam::DVec3::ONE * -4.0 * cosmic_system::simulation::AU,
        glam::DVec3::ONE * 4.0 * cosmic_system::simulation::AU,
    );
    c.bench_function("update_bodies", |b| {
        b.iter(|| {
            update_bodies(black_box(bounding_box), black_box(&mut bodies.clone()));
        })
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
