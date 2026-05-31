use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use image::DynamicImage;
use sc_core::types::ScaleMode;
use sc_image::{
    cache::ImageCache,
    compositor::Compositor,
    scale::{scale_image, ScaleOptions},
    thumbnail::{generate_thumbnails_sorted, ThumbnailSpec},
};

/// 800x1200 세로형 이미지를 1024x768 윈도우에 FitWindow 스케일링하는 속도를 측정한다.
/// Lanczos3 필터를 사용하는 리사이즈가 주요 비용이다.
fn bench_scale_fit_window(c: &mut Criterion) {
    let img = DynamicImage::new_rgb8(800, 1200);
    let opts = ScaleOptions {
        mode: ScaleMode::FitWindow,
        window_width: 1024,
        window_height: 768,
    };

    let mut group = c.benchmark_group("image_scale");

    group.bench_with_input(
        BenchmarkId::new("fit_window", "800x1200_to_1024x768"),
        &(&img, &opts),
        |b, (image, options)| {
            b.iter(|| {
                // black_box 로 컴파일러가 리사이즈 결과를 제거하는 것을 방지한다
                black_box(scale_image(black_box(image), black_box(options)))
            });
        },
    );

    group.finish();
}

/// 두 800x1200 이미지를 나란히 합성하는 two-page spread 속도를 측정한다.
/// 내부적으로 height 정규화 → canvas 할당 → copy_from 두 번이 발생한다.
fn bench_compositor_two_page(c: &mut Criterion) {
    let left = DynamicImage::new_rgb8(800, 1200);
    let right = DynamicImage::new_rgb8(800, 1200);

    let mut group = c.benchmark_group("image_compositor");

    group.bench_with_input(
        BenchmarkId::new("two_page_spread", "800x1200_plus_800x1200"),
        &(&left, &right),
        |b, (l, r)| {
            b.iter(|| {
                // black_box 로 합성 결과 이미지 버퍼가 최적화 제거되지 않도록 한다
                black_box(Compositor::two_page_spread(black_box(l), black_box(r)))
            });
        },
    );

    group.finish();
}

/// capacity 50인 캐시에 60개 이미지를 순서대로 삽입해 eviction 경로를 포함한 삽입 비용을 측정한다.
/// LRU eviction이 51번째 삽입부터 발생한다.
fn bench_image_cache_insert_evict(c: &mut Criterion) {
    // 미리 60개 이미지 객체를 생성해 둔다 (벤치마크 루프 안에서 생성 비용이 측정되지 않도록)
    let images: Vec<DynamicImage> = (0..60).map(|_| DynamicImage::new_rgb8(64, 64)).collect();

    let mut group = c.benchmark_group("image_cache");

    group.bench_function("insert_evict_60_into_cap50", |b| {
        b.iter(|| {
            // 매 이터레이션마다 빈 캐시를 새로 생성해 상태 누적을 방지한다
            let mut cache = ImageCache::new(50);
            for (idx, img) in images.iter().enumerate() {
                // black_box 로 idx·img 가 컴파일 시점에 인라인되지 않도록 한다
                cache.insert(black_box(idx), black_box(img.clone()));
            }
            // 최종 캐시 크기를 소비해 컴파일러가 루프를 제거하지 못하게 한다
            black_box(cache.len())
        });
    });

    group.finish();
}

/// 지정된 개수의 PNG 이미지 바이트를 생성한다.
/// 각 이미지는 valid한 1×1 RGB PNG이다.
fn create_image_entries(count: usize) -> Vec<(usize, Vec<u8>)> {
    (0..count)
        .map(|i| {
            let img = DynamicImage::new_rgb8(64, 64);
            let mut buf = Vec::new();
            img.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
                .expect("failed to encode image");
            (i, buf)
        })
        .collect()
}

/// rayon으로 10개 이미지의 썸네일을 병렬 생성하는 속도를 측정한다.
/// generate_thumbnails_sorted()를 사용하여 결과가 원래 순서대로 정렬되는지 확인한다.
fn bench_thumbnail_parallel_10(c: &mut Criterion) {
    let entries = black_box(create_image_entries(10));
    let spec = black_box(ThumbnailSpec::default());

    let mut group = c.benchmark_group("thumbnail_parallel");

    group.bench_function("parallel_10_entries", |b| {
        b.iter(|| {
            black_box(generate_thumbnails_sorted(
                black_box(&entries),
                black_box(spec),
            ))
        });
    });

    group.finish();
}

/// rayon으로 50개 이미지의 썸네일을 병렬 생성하는 속도를 측정한다.
/// 병렬 처리의 이점이 명확해지는 데이터 크기이다.
fn bench_thumbnail_parallel_50(c: &mut Criterion) {
    let entries = black_box(create_image_entries(50));
    let spec = black_box(ThumbnailSpec::default());

    let mut group = c.benchmark_group("thumbnail_parallel");

    group.bench_function("parallel_50_entries", |b| {
        b.iter(|| {
            black_box(generate_thumbnails_sorted(
                black_box(&entries),
                black_box(spec),
            ))
        });
    });

    group.finish();
}

/// 50개 이미지의 썸네일을 순서대로(serial) 생성하고, 병렬 버전과 비교한다.
/// 이 벤치마크는 rayon 파이프라인의 오버헤드를 측정한다.
fn bench_thumbnail_serial_50_comparison(c: &mut Criterion) {
    let entries = black_box(create_image_entries(50));
    let spec = black_box(ThumbnailSpec::default());

    let mut group = c.benchmark_group("thumbnail_serial_vs_parallel");

    group.bench_function("serial_50_entries", |b| {
        b.iter(|| {
            // 순차 처리: par_iter() 대신 iter() 사용
            let results: Vec<(usize, DynamicImage)> = entries
                .iter()
                .filter_map(|(idx, bytes)| {
                    let img = image::load_from_memory(bytes).ok()?;
                    let thumb = sc_image::thumbnail::generate_thumbnail(&img, spec);
                    Some((*idx, thumb))
                })
                .collect();
            black_box(results)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_scale_fit_window,
    bench_compositor_two_page,
    bench_image_cache_insert_evict,
    bench_thumbnail_parallel_10,
    bench_thumbnail_parallel_50,
    bench_thumbnail_serial_50_comparison
);
criterion_main!(benches);
