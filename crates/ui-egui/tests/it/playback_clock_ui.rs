//! Playback clock: the audio device drives the playhead while it plays, and a device that opens
//! but never consumes samples (seen on Linux with ALSA, #136) must not freeze playback.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use egui_kittest::Harness;
use filmcraft_engine::Session;
use filmcraft_time::Tick;
use filmcraft_ui_egui::{AudioOut, FilmcraftApp};
use serde_json::json;

/// An output whose stream opens, then plays `per_poll` frames each time the clock is read
/// (0 = a stalled device that never calls back).
struct FakeOut {
    played: Arc<AtomicU64>,
    per_poll: u64,
    started: bool,
    note: Option<String>,
}

impl AudioOut for FakeOut {
    fn start(&mut self, _fill: Box<dyn FnMut(&mut [f32], usize) + Send>) -> Result<u32, String> {
        self.played.store(0, Ordering::SeqCst);
        self.started = true;
        Ok(48_000)
    }
    fn stop(&mut self) {
        self.started = false;
    }
    fn sample_rate(&self) -> u32 {
        48_000
    }
    fn played_frames(&self) -> Option<u64> {
        self.started.then(|| self.played.fetch_add(self.per_poll, Ordering::SeqCst) + self.per_poll)
    }
    fn note(&self) -> Option<String> {
        self.note.clone()
    }
}

/// Play for `steps` × 0.25 s of simulated time with the given output; the playhead in seconds.
fn play_with(per_poll: u64, steps: usize) -> (f64, bool) {
    let mut s = Session::default();
    s.execute("file.openDemoProject", json!({})).unwrap();
    s.set_playhead(Tick::ZERO);
    let mut app = FilmcraftApp::new(s);
    app.audio = Some(Box::new(FakeOut { played: Arc::new(AtomicU64::new(0)), per_poll, started: false, note: None }));
    let mut h = Harness::builder().with_size(egui::vec2(1280.0, 800.0)).with_step_dt(0.25).build_eframe(move |_cc| app);
    h.step();
    h.state_mut().play(1.0);
    for _ in 0..steps {
        h.step();
    }
    let app = h.state_mut();
    (app.session.playhead().seconds(), app.playback.audio_clock)
}

/// #136: the stream opened, the device never asked for samples, the audio clock stayed at 0 and
/// the playhead never moved. After a short grace period playback falls back to the wall clock.
#[test]
fn a_stalled_audio_device_does_not_freeze_playback() {
    let (t, audio_clock) = play_with(0, 16);
    assert!(t > 1.5, "the playhead should move with the wall clock, it is at {t:.2} s");
    assert!(!audio_clock, "the stalled device no longer drives the clock");
}

/// Control: a device that plays keeps driving the clock (0.1 s of audio per poll).
#[test]
fn a_playing_audio_device_drives_the_clock() {
    let (t, audio_clock) = play_with(4_800, 8);
    assert!(t > 0.2, "{t:.2} s");
    assert!(audio_clock);
}

/// A fallback note from the audio device (broken ALSA `default`, #23/#106) reaches the status
/// bar on the first frame, and survives while playback runs.
#[test]
fn a_fallback_note_from_the_audio_device_reaches_the_status_bar() {
    let mut s = Session::default();
    s.execute("file.openDemoProject", json!({})).unwrap();
    s.set_playhead(Tick::ZERO);
    let note = "Audio hardware: the default output device cannot be opened; using 'sofhdadsp'".to_string();
    let mut app = FilmcraftApp::new(s);
    app.audio = Some(Box::new(FakeOut { played: Arc::new(AtomicU64::new(0)), per_poll: 4_800, started: false, note: Some(note.clone()) }));
    let mut h = Harness::builder().with_size(egui::vec2(1280.0, 800.0)).with_step_dt(0.25).build_eframe(move |_cc| app);
    h.step();
    assert_eq!(h.state_mut().ui.status, note, "the note surfaces on the first frame (apply_prefs)");
    h.state_mut().play(1.0);
    for _ in 0..16 {
        h.step();
    }
    assert_eq!(h.state_mut().ui.status, note, "the note survives while playback runs");
}

/// A 1080p25 long-GOP H.264 clip with AAC sound (one keyframe in 250 frames), made with ffmpeg;
/// `None` without ffmpeg.
fn long_gop_clip() -> Option<std::path::PathBuf> {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/fixtures");
    std::fs::create_dir_all(&dir).ok()?;
    let path = dir.join("play-start-1080p.mp4");
    if !path.exists() {
        let tmp = dir.join(format!("play-start-{}.tmp.mp4", std::process::id()));
        let ok = std::process::Command::new("ffmpeg")
            .args([
                "-v",
                "error",
                "-y",
                "-f",
                "lavfi",
                "-i",
                "testsrc2=size=1920x1080:rate=25:duration=12",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=12",
            ])
            .args(["-c:v", "libx264", "-preset", "veryfast", "-g", "250", "-bf", "2", "-pix_fmt", "yuv420p", "-c:a", "aac", "-shortest"])
            .arg(&tmp)
            .status()
            .is_ok_and(|s| s.success());
        if !ok {
            return None;
        }
        std::fs::rename(&tmp, &path).ok()?;
    }
    Some(path)
}

/// Real seconds from `play` to the clock starting, on the long-GOP clip played from the middle of
/// its group of pictures after `idle_frames` UI frames of the paused picture; `None` without ffmpeg.
fn play_start_wait(idle_frames: usize) -> Option<f64> {
    let clip = long_gop_clip()?;
    let mut s = Session::default();
    let r = s.execute("file.import", json!({"paths": [clip.to_string_lossy()]})).unwrap();
    let item = r["items"][0].as_u64().unwrap();
    s.execute("file.newSequenceFromClip", json!({"items": [item]})).unwrap();
    s.set_playhead(Tick::from_seconds_f64(6.0));
    let mut app = FilmcraftApp::new(s);
    app.audio = Some(Box::new(FakeOut { played: Arc::new(AtomicU64::new(0)), per_poll: 800, started: false, note: None }));
    let mut h = Harness::builder().with_size(egui::vec2(1280.0, 800.0)).with_step_dt(1.0 / 60.0).build_eframe(move |_cc| app);
    let step = |h: &mut Harness<'_, FilmcraftApp>| {
        std::thread::sleep(std::time::Duration::from_millis(16));
        h.step();
    };
    // the paused frame is on screen
    for _ in 0..idle_frames {
        step(&mut h);
    }
    h.state_mut().play(1.0);
    let t0 = std::time::Instant::now();
    while h.state_mut().playback.preroll.is_some() && t0.elapsed().as_secs_f64() < 3.0 {
        step(&mut h);
    }
    Some(t0.elapsed().as_secs_f64())
}

/// #3600: Play starts the clock (and the sound) without a long wait: soon after the key press when
/// the next frames are ready, and after at most a short timeout when the decoder is still seeking
/// (it waited half a second, then a first block of sound).
#[test]
fn play_starts_within_a_fraction_of_a_second() {
    let (Some(warm), Some(cold)) = (play_start_wait(90), play_start_wait(3)) else {
        eprintln!("SKIPPED (play_starts_within_a_fraction_of_a_second): ffmpeg not found");
        return;
    };
    eprintln!("Play waited {warm:.3} s after the picture settled, {cold:.3} s right after the seek");
    assert!(warm < 0.2, "Play waited {warm:.2} s with the picture settled");
    assert!(cold < 0.25, "Play waited {cold:.2} s right after the seek");
}
