//! Hostile-input robustness: truncation, header-region sweeps and random mutation of small
//! ffmpeg-generated AV1 streams (frame by frame) must never panic or run away. Skips without ffmpeg.

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

use filmcraft_av1::Decoder;

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
}

/// (name, encoder, encoder args, pix_fmt, height). SVT requires at least 64 pixels;
/// libaom keeps the smaller-frame hostile coverage independently.
const SEEDS: &[(&str, &str, &str, &str, u32)] = &[
    ("av1_fz_svt_64_default", "libsvtav1", "-g 4", "yuv420p", 64),
    ("av1_fz_svt_64_10bit", "libsvtav1", "-g 6", "yuv420p10le", 64),
    ("av1_fz_aom_48_default", "libaom-av1", "-cpu-used 8 -threads 1 -g 4", "yuv420p", 48),
    ("av1_fz_aom_48_10bit", "libaom-av1", "-cpu-used 8 -threads 1 -g 6", "yuv420p10le", 48),
];

fn ivf_frames(data: &[u8]) -> Vec<Vec<u8>> {
    let mut out = Vec::new();
    let Some(h) = data.get(6..8) else { return out };
    let mut pos = usize::from(u16::from_le_bytes([h[0], h[1]]));
    while let Some(hd) = data.get(pos..pos + 12) {
        let size = u32::from_le_bytes([hd[0], hd[1], hd[2], hd[3]]) as usize;
        pos += 12;
        let end = pos.saturating_add(size).min(data.len());
        out.push(data[pos..end].to_vec());
        pos = end;
    }
    out
}

fn make(ff: &std::path::Path, name: &str, encoder: &str, args: &str, pix: &str, height: u32) -> Option<Vec<Vec<u8>>> {
    let out = filmcraft_testkit::fixtures_dir("av1").join(format!("{name}.ivf"));
    let p = filmcraft_testkit::fixtures::generate(&out, |tmp| {
        let a: Vec<&str> = args.split_whitespace().collect();
        let source = format!("testsrc2=size=64x{height}:rate=25");
        matches!(
            Command::new(ff)
                .args(["-hide_banner", "-loglevel", "error", "-y", "-f", "lavfi", "-i", &source, "-frames:v", "6", "-pix_fmt", pix])
                .args(["-c:v", encoder])
                .args(a)
                .args(["-f", "ivf"])
                .arg(tmp)
                .stdin(Stdio::null())
                .status(),
            Ok(s) if s.success()
        )
    })?;
    Some(ivf_frames(&std::fs::read(p).ok()?))
}

static PANICS: AtomicUsize = AtomicUsize::new(0);

/// Decoder worker threads may catch panics and turn them into errors; count them so they still fail the test.
fn count_panics() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        std::panic::set_hook(Box::new(|info| {
            PANICS.fetch_add(1, Ordering::SeqCst);
            eprintln!("PANIC: {info}");
        }));
    });
}

fn run_all(frames: &[Vec<u8>], label: &str) {
    filmcraft_testkit::set_progress(label);
    count_panics();
    let before = PANICS.load(Ordering::SeqCst);
    let res = catch_unwind(AssertUnwindSafe(|| {
        for threads in [1, 3] {
            let mut dec = Decoder::with_threads(threads);
            for (i, f) in frames.iter().enumerate() {
                let _ = dec.decode_pts(f, i as i64);
            }
            let _ = dec.flush_result();
        }
    }));
    assert!(res.is_ok() && PANICS.load(Ordering::SeqCst) == before, "panic: {label}");
}

/// Random-mutation rounds per seed (raise with FILMCRAFT_FUZZ_ITERS for a long hunt).
fn iters() -> usize {
    std::env::var("FILMCRAFT_FUZZ_ITERS").ok().and_then(|s| s.parse().ok()).unwrap_or(300)
}

/// Seconds the whole sweep may take before it counts as a hang (override: FILMCRAFT_HOSTILE_TIMEOUT_SECS).
const DEADLINE_SECS: u64 = 600;

#[test]
fn hostile_streams_never_panic() {
    filmcraft_testkit::with_deadline(DEADLINE_SECS, sweep);
}

fn sweep() {
    let Some(ff) = filmcraft_testkit::ffmpeg_or_skip("AV1 fuzz") else { return };
    for &(name, encoder, args, pix, height) in SEEDS {
        if !filmcraft_testkit::ffmpeg_has_encoder(&ff, encoder) {
            filmcraft_testkit::skip("AV1 fuzz", &format!("this ffmpeg has no {encoder} encoder (seed {name})"));
            continue;
        }
        let Some(seed) = make(&ff, name, encoder, args, pix, height) else {
            panic!("ffmpeg has {encoder} but could not encode the {name} seed");
        };
        if seed.is_empty() {
            filmcraft_testkit::skip("AV1 fuzz", &format!("the {name} seed has no frames"));
            continue;
        }
        // truncate each of the first frames at every length
        for fi in 0..seed.len().min(3) {
            let len = seed[fi].len();
            for n in (0..len).step_by((len / 200).max(1)) {
                let mut d = seed.clone();
                d[fi].truncate(n);
                run_all(&d, &format!("{name} frame {fi} truncate {n}"));
            }
        }
        // header region of every frame: every byte to a few values, every bit
        for fi in 0..seed.len() {
            for p in 0..seed[fi].len().min(96) {
                for v in [0u8, 0xff, 0x80, 0x01, 0x7f] {
                    let mut d = seed.clone();
                    d[fi][p] = v;
                    run_all(&d, &format!("{name} frame {fi} byte {p}={v:#x}"));
                }
                for b in 0..8 {
                    let mut d = seed.clone();
                    d[fi][p] ^= 1 << b;
                    run_all(&d, &format!("{name} frame {fi} flip {p}.{b}"));
                }
            }
        }
        let mut rng = Rng(0x9e3779b97f4a7c15 ^ name.len() as u64);
        for i in 0..iters() {
            let mut d = seed.clone();
            for _ in 0..1 + rng.below(6) {
                let fi = rng.below(d.len());
                if d[fi].is_empty() {
                    continue;
                }
                let p = rng.below(d[fi].len());
                if rng.below(2) == 0 {
                    d[fi][p] ^= 1 << rng.below(8);
                } else {
                    d[fi][p] = rng.next() as u8;
                }
            }
            run_all(&d, &format!("{name} random {i}"));
        }
    }
}
