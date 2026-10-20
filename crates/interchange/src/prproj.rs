//! Adobe Premiere Pro project files (`.prproj`), read-only.
//!
//! A `.prproj` written by Premiere Pro (2013 and later) is a gzip stream whose member is an XML
//! document; projects saved by older versions are the same XML uncompressed. The root element is
//! `<PremiereData>`, and the document is an object graph, not a tree: objects are declared once
//! anywhere in the file with an `ObjectID` (small integer) or `ObjectUID` (GUID) attribute, and
//! referenced elsewhere through placeholder elements carrying `ObjectRef`/`ObjectURef`.
//! Premiere's own time unit is an integer tick of which there are 254,016,000,000 per second —
//! the same rate as FilmCraft's [`Tick`], so every time in a project maps onto ours exactly.
//!
//! What we read:
//!
//! - the project tree: the root bin, bins, media items and sequences
//!   (`ClipProjectItem` → `MasterClip` → clip objects → media or sequence sources);
//! - sequences: video and audio track groups, tracks (locked, muted, sync-locked), placed clips
//!   (`ClipTrackItem`: start/end/in/out, enabled), nested sequences, audio/video links;
//! - transitions (`TransitionTrackItem` → `Transition`) with geometry-derived alignment;
//! - the common intrinsic effect parameters on placed clips — Motion (Position, Scale, Scale
//!   Width, Rotation, Anchor Point), Opacity, Volume, Pan — static and keyframed
//!   (`VideoComponentParam`/`AudioComponentParam` with `StartKeyframe`/`Keyframes` tuples);
//! - sequence and clip markers, and Premiere label colours.
//!
//! What we do not read is reported, never silently dropped: unknown effects and transitions,
//! keyframe interpolation styles (imported as linear), variable speed, and anything else the
//! graph contains that has no FilmCraft counterpart. Media that is missing on disk stays
//! offline, as with the other interchange formats.
//!
//! The container is described publicly (gzip + XML); the XML vocabulary is not documented by
//! Adobe, so this importer is written defensively: every step resolves what it recognizes and
//! reports the rest.

use std::collections::{HashMap, HashSet};
use std::io::Read;

use filmcraft_project::{
    BinId, ClipId, ItemId, ItemKind, Keyframe, Label, Marker, MarkerId, MarkerKind, Param, ParamValue, Sequence, Track, TrackItem, TrackKind, Transition,
    TransitionAlign, TransitionId,
};
use filmcraft_time::{FrameRate, TICKS_PER_SECOND, Tick};
use roxmltree::Node;

use crate::common::{Builder, MediaSpec, empty_sequence, kind_from_ext, rate_from_f64, resolve_path, set_param, settings_for, transition_effect};
use crate::xml::{child, child_bool, child_i64, child_text, elements, parse, text};
use crate::{Error, Format, ImportOptions, Imported, Report, Result};

/// A Premiere project can legitimately be hundreds of megabytes uncompressed, but a gzip bomb is
/// not a project: refuse to inflate past this and say so.
const MAX_UNPACKED: u64 = 1 << 30;
/// Refuse absurdly wide object graphs.
const MAX_OBJECTS: usize = 400_000;
/// Caps on lists read from one object.
const MAX_MARKERS: usize = 10_000;
const MAX_KEYFRAMES: usize = 10_000;
const MAX_ITEMS: usize = 4096;
/// Recursion depth cap for bins and nested sequences.
const MAX_DEPTH: usize = 64;

/// The GUID Premiere writes in `MediaType` elements for picture tracks and items.
const MEDIA_TYPE_VIDEO: &str = "228cda18-3625-4d2d-951e-348879e4ed93";
/// …and for sound.
const MEDIA_TYPE_AUDIO: &str = "80b8e3d5-6dca-4195-aefb-cb5f407ab009";

pub(crate) fn sniff(bytes: &[u8]) -> bool {
    bytes.starts_with(&[0x1f, 0x8b]) || head_is_premiere_xml(bytes)
}

fn head_is_premiere_xml(bytes: &[u8]) -> bool {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(4096)]);
    let head = head.trim_start_matches('\u{feff}').trim_start();
    (head.starts_with("<?xml") && head.contains("PremiereData")) || head.starts_with("<PremiereData")
}

/// Expand the container: gzip (2013+) or plain XML (older), both to text.
fn decode(bytes: &[u8]) -> Result<String> {
    if bytes.starts_with(&[0x1f, 0x8b]) {
        let mut out = Vec::new();
        let gz = flate2::read::MultiGzDecoder::new(bytes);
        gz.take(MAX_UNPACKED).read_to_end(&mut out).map_err(|e| Error::parse(Format::Prproj, format!("not a valid gzip stream: {e}")))?;
        if out.len() as u64 >= MAX_UNPACKED {
            return Err(Error::parse(Format::Prproj, "the project expands past 1 GiB; export FCP7 XML from Premiere and import that instead"));
        }
        return Ok(lossy_text(&out));
    }
    if head_is_premiere_xml(bytes) {
        let b = bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes);
        return Ok(lossy_text(b));
    }
    Err(Error::parse(Format::Prproj, "not a Premiere project (neither a gzip stream nor Premiere XML)"))
}

fn lossy_text(b: &[u8]) -> String {
    match std::str::from_utf8(b) {
        Ok(s) => s.to_string(),
        Err(_) => b.iter().map(|&c| c as char).collect(),
    }
}

// ---------------------------------------------------------------------------------------------
// Object graph
// ---------------------------------------------------------------------------------------------

/// Every object in the document, by its `ObjectID`/`ObjectUID`/`ObjectUId` attribute.
struct Objects<'a, 'i> {
    by_id: HashMap<&'a str, Node<'a, 'i>>,
}

impl<'a, 'i> Objects<'a, 'i> {
    fn index(doc: &'a roxmltree::Document<'i>) -> Result<Self> {
        let mut by_id = HashMap::new();
        for n in doc.descendants() {
            if !n.is_element() {
                continue;
            }
            for attr in ["ObjectID", "ObjectUID", "ObjectUId"] {
                if let Some(id) = n.attribute(attr) {
                    by_id.insert(id, n);
                    break;
                }
            }
            if by_id.len() > MAX_OBJECTS {
                return Err(Error::parse(Format::Prproj, "the project declares too many objects"));
            }
        }
        Ok(Self { by_id })
    }

    /// The object a placeholder element points at (`ObjectRef`/`ObjectURef` attribute).
    fn resolve(&self, n: Node<'a, 'i>) -> Option<Node<'a, 'i>> {
        for attr in ["ObjectRef", "ObjectURef"] {
            if let Some(r) = n.attribute(attr) {
                return self.by_id.get(r).copied();
            }
        }
        None
    }

    /// The element at `path` below `n`, following every placeholder along the way (Premiere
    /// placeholders like `<Clip ObjectRef="n"/>` can appear at any depth).
    fn resolve_path(&self, n: Node<'a, 'i>, path: &[&str]) -> Option<Node<'a, 'i>> {
        let mut cur = n;
        for p in path {
            cur = child(cur, p)?;
            if cur.attribute("ObjectRef").is_some() || cur.attribute("ObjectURef").is_some() {
                cur = self.resolve(cur)?;
            }
        }
        Some(cur)
    }
}

/// A node's own object id.
fn id_of<'a>(n: Node<'a, '_>) -> Option<&'a str> {
    n.attribute("ObjectID").or_else(|| n.attribute("ObjectUID")).or_else(|| n.attribute("ObjectUId"))
}

/// The first descendant element with `tag` (search bounded, like every walk here).
fn find_desc<'a, 'i>(n: Node<'a, 'i>, tag: &str) -> Option<Node<'a, 'i>> {
    n.descendants().take(512).find(|d| d.is_element() && d.has_tag_name(tag))
}

// ---------------------------------------------------------------------------------------------
// Times, rates and comma tuples
// ---------------------------------------------------------------------------------------------

/// Premiere writes rates as ticks per frame (picture) or per sample (sound): 254016000000/24 for
/// 24 fps, 5292000 for 48 kHz. Convert to a frame rate, snapping to the common broadcast rates.
fn rate_from_tpf(tpf: i64) -> Option<FrameRate> {
    if tpf <= 0 || tpf > TICKS_PER_SECOND {
        return None;
    }
    let fps = TICKS_PER_SECOND as f64 / tpf as f64;
    if !fps.is_finite() || fps <= 0.0 {
        return None;
    }
    Some(rate_from_f64(fps))
}

/// Parse `a,b,c` tuples: Premiere writes floats like `960.`.
fn tuple_i64(s: &str) -> Vec<i64> {
    s.split(',').filter_map(|p| p.trim().parse::<i64>().ok()).collect()
}

/// A keyframe line: `time,value,…` (further fields — interpolation, easing — are version-specific).
fn parse_keyframe_line(line: &str) -> Option<(Tick, Vec<f64>)> {
    let mut it = line.split(',');
    let t = it.next()?.trim().parse::<i64>().ok()?;
    let values: Vec<f64> = it.filter_map(|p| p.trim().parse::<f64>().ok()).collect();
    if values.is_empty() {
        return None;
    }
    Some((Tick(t), values))
}

/// The keyframes of a param: the `Keyframes` element (one tuple per line) when it is animated,
/// else the single `StartKeyframe` tuple.
fn param_keyframes(n: Node) -> Vec<(Tick, Vec<f64>)> {
    if let Some(k) = child(n, "Keyframes").filter(|k| !text(*k).trim().is_empty()) {
        return text(k).lines().take(MAX_KEYFRAMES).filter_map(parse_keyframe_line).collect();
    }
    child_text(n, "StartKeyframe").and_then(parse_keyframe_line).into_iter().collect()
}

// ---------------------------------------------------------------------------------------------
// Import
// ---------------------------------------------------------------------------------------------

struct Imp<'a, 'i, 'r> {
    b: Builder,
    opts: &'a ImportOptions,
    report: &'r mut Report,
    objs: &'a Objects<'a, 'i>,
    /// Media node id → item.
    media_items: HashMap<String, ItemId>,
    /// Sequence node id → item (memoized; cycles guarded by `in_progress`).
    seq_items: HashMap<String, ItemId>,
    in_progress: HashSet<String>,
    /// ClipTrackItem node id → (sequence item, ClipId), for transitions and links.
    placed: HashMap<String, (ItemId, ClipId)>,
    /// Linked pairs (per sequence), resolved after all tracks are read.
    links: Vec<(ItemId, ClipId, ClipId)>,
    warned_interp: bool,
}

pub(crate) fn import(bytes: &[u8], opts: &ImportOptions, report: &mut Report) -> Result<Imported> {
    let text = decode(bytes)?;
    let doc = parse(&text).map_err(|e| Error::parse(Format::Prproj, e))?;
    let root = doc.root_element();
    if root.tag_name().name() != "PremiereData" {
        return Err(Error::parse(Format::Prproj, "root element is not <PremiereData>"));
    }
    let objs = Objects::index(&doc)?;
    let mut imp = Imp {
        b: Builder::new(opts.name.as_deref().unwrap_or("Imported Premiere Project")),
        opts,
        report,
        objs: &objs,
        media_items: HashMap::new(),
        seq_items: HashMap::new(),
        in_progress: HashSet::new(),
        placed: HashMap::new(),
        links: Vec::new(),
        warned_interp: false,
    };

    let project = doc.descendants().find(|n| n.is_element() && n.has_tag_name("Project") && id_of(*n).is_some());
    let root_bin = project.as_ref().and_then(|p| imp.objs.resolve_path(*p, &["RootProjectItem"]));
    if let Some(rb) = root_bin {
        imp.walk_container(rb, None, 0);
    } else {
        // No recognizable project tree: import every sequence the document declares.
        let seqs: Vec<Node> = doc.descendants().filter(|n| n.is_element() && n.has_tag_name("Sequence") && id_of(*n).is_some()).take(512).collect();
        for s in seqs {
            if let Some(id) = imp.sequence(s, 0) {
                imp.b.top.push(id);
            }
        }
    }
    imp.resolve_links();
    if imp.b.top.is_empty() && imp.b.p.root.children.is_empty() {
        return Err(Error::Empty);
    }
    Ok(imp.b.finish())
}

impl<'a, 'i, 'r> Imp<'a, 'i, 'r> {
    /// A bin-like object (`RootProjectItem`, `BinProjectItem`): recurse over its items.
    fn walk_container(&mut self, bin: Node<'a, 'i>, parent: Option<BinId>, depth: usize) {
        if depth > MAX_DEPTH {
            self.report.warn("the bin tree is nested too deeply; deeper bins were skipped");
            return;
        }
        let own = if bin.has_tag_name("RootProjectItem") { None } else { Some(self.b.bin(&self.name_of(bin), parent)) };
        let parent = own.or(parent);
        let Some(items) = self.objs.resolve_path(bin, &["ProjectItemContainer", "Items"]) else {
            return;
        };
        for item_ref in elements(items).take(MAX_ITEMS) {
            let Some(item) = self.objs.resolve(item_ref) else { continue };
            if self.objs.resolve_path(item, &["ProjectItemContainer"]).is_some() {
                self.walk_container(item, parent, depth + 1);
            } else {
                self.project_item(item, parent);
            }
        }
    }

    /// A leaf project item: media, a sequence, or (reported) anything else.
    fn project_item(&mut self, item: Node<'a, 'i>, bin: Option<BinId>) {
        let name = self.name_of(item);
        let label = self.label_of(item);
        let mc = self.objs.resolve_path(item, &["MasterClip"]);
        let source = mc.and_then(|mc| self.master_clip_item(mc, &name));
        let Some(id) = source else {
            self.report.warn(format!("project item \"{name}\" has no media or sequence source; skipped"));
            return;
        };
        if let Some(p) = self.b.p.item_mut(id) {
            p.name = name.clone();
            if let Some(l) = label {
                p.label = l;
            }
        }
        if let Some(b) = bin {
            self.b.p.move_to_bin(&[id], Some(b));
        }
        // A sequence that is its own project item is top-level; nested ones are items only.
        if matches!(self.b.p.item(id).map(|i| &i.kind), Some(ItemKind::Sequence(_))) && !self.b.top.contains(&id) {
            self.b.top.push(id);
        }
    }

    /// What a `MasterClip` points at: the first media or sequence source among its clip objects.
    fn master_clip_item(&mut self, mc: Node<'a, 'i>, name: &str) -> Option<ItemId> {
        let clips = self.objs.resolve_path(mc, &["Clips"])?;
        for c in elements(clips).take(16) {
            let Some(clip) = self.objs.resolve(c) else { continue };
            if let Some(id) = self.clip_source(clip, name) {
                return Some(id);
            }
        }
        None
    }

    /// A clip object (`VideoClip`/`AudioClip`/…) → its media or sequence item.
    fn clip_source(&mut self, clip: Node<'a, 'i>, name: &str) -> Option<ItemId> {
        // `resolve_path` follows the `Source` placeholder, landing on the media/sequence source.
        let src = self.objs.resolve_path(clip, &["Clip", "Source"])?;
        if let Some(seq_ref) = self.objs.resolve_path(src, &["SequenceSource", "Sequence"]) {
            return self.sequence(seq_ref, 0);
        }
        if let Some(media) = self.objs.resolve_path(src, &["MediaSource", "Media"]) {
            let shown = child_text(clip, "Name").filter(|s| !s.trim().is_empty()).map(str::to_string).unwrap_or_else(|| name.to_string());
            return Some(self.media_item(media, &shown, None));
        }
        None
    }

    /// An item's name: the `Name` under its `ProjectItem` wrapper, else any `Name` child.
    fn name_of(&self, n: Node<'a, 'i>) -> String {
        let name = child(n, "ProjectItem").and_then(|p| child_text(p, "Name")).or_else(|| child_text(n, "Name"));
        name.filter(|s| !s.trim().is_empty()).unwrap_or("Untitled").trim().to_string()
    }

    /// Premiere label colours are stored as `BE.Prefs.LabelColors.N`; ours are an enum in order.
    fn label_of(&self, n: Node<'a, 'i>) -> Option<Label> {
        let holder = child(n, "ProjectItem").unwrap_or(n);
        let key = find_desc(holder, "asl.clip.label.name")?;
        let idx: usize = text(key).rsplit('.').next().and_then(|d| d.trim().parse().ok())?;
        Label::ALL.get(idx.checked_sub(1)?).copied()
    }

    /// A `Media` object → a media item (deduplicated by node id and path).
    fn media_item(&mut self, media: Node<'a, 'i>, name: &str, bin: Option<BinId>) -> ItemId {
        let key = id_of(media).unwrap_or_default().to_string();
        if let Some(id) = self.media_items.get(&key) {
            return *id;
        }
        let path_raw = child_text(media, "ActualMediaFilePath").or_else(|| child_text(media, "FilePath")).filter(|p| !p.trim().is_empty());
        let title = child_text(media, "Title").filter(|t| !t.trim().is_empty());
        let mut spec = MediaSpec::default();
        if let Some(vs) = self.objs.resolve_path(media, &["VideoStream"]) {
            if let Some(fr) = child_text(vs, "FrameRect").map(tuple_i64)
                && fr.len() >= 4
                && fr[2] > fr[0]
                && fr[3] > fr[1]
                && let (Ok(w), Ok(h)) = (u32::try_from(fr[2] - fr[0]), u32::try_from(fr[3] - fr[1]))
            {
                spec.video = Some((w, h, child_i64(vs, "FrameRate").and_then(rate_from_tpf).unwrap_or_default()));
            }
            if let Some(d) = child_i64(vs, "Duration").filter(|d| *d > 0) {
                spec.duration = Some(Tick(d));
            }
        }
        if let Some(asr) = self.objs.resolve_path(media, &["AudioStream"]) {
            let sr = child_i64(asr, "FrameRate").filter(|t| *t > 0 && *t <= TICKS_PER_SECOND).map(|t| (TICKS_PER_SECOND / t) as u32);
            let ch = child_i64(asr, "NumChannels")
                .map(|c| c.clamp(1, 64) as u32)
                .unwrap_or_else(|| asr.children().filter(|c| c.is_element() && c.has_tag_name("ChannelType")).count().clamp(1, 64) as u32);
            if sr.is_some() || child(asr, "ChannelType").is_some() {
                spec.audio = Some((sr.unwrap_or(48_000), ch));
            }
        }
        let shown = title.unwrap_or(name).to_string();
        let id = match path_raw {
            Some(p) => {
                let resolved = resolve_path(p, self.opts.base_dir.as_deref());
                let key = format!("file:{resolved}");
                self.b.file_media(&key, &shown, &resolved, &spec, bin)
            }
            None => {
                self.report.warn(format!("media \"{shown}\" has no file path; imported offline"));
                let p = resolve_path(&shown, self.opts.base_dir.as_deref());
                let key = format!("offline:{p}");
                let id = self.b.file_media(&key, &shown, &p, &spec, bin);
                if let Some(m) = self.b.p.item_mut(id).and_then(|i| i.as_media_mut()) {
                    m.offline = true;
                }
                id
            }
        };
        if spec.video.is_none() && spec.audio.is_none() {
            // No stream info: guess from the extension so stills land as stills.
            let path = child_text(media, "ActualMediaFilePath").or_else(|| child_text(media, "FilePath"));
            match kind_from_ext(path.unwrap_or(&shown)) {
                filmcraft_media::MediaKind::AudioOnly => {
                    if let Some(m) = self.b.p.item_mut(id).and_then(|i| i.as_media_mut()) {
                        m.info.audio_streams.push(filmcraft_media::AudioStreamInfo {
                            sample_rate: 48_000,
                            channels: 2,
                            codec: String::new(),
                            bits_per_sample: None,
                        });
                    }
                }
                k => {
                    if let Some(m) = self.b.p.item_mut(id).and_then(|i| i.as_media_mut()) {
                        m.info.kind = k;
                    }
                }
            }
        }
        self.media_items.insert(key, id);
        id
    }

    /// A `Sequence` object → a sequence item (memoized; cycles return the reserved item).
    fn sequence(&mut self, seq: Node<'a, 'i>, depth: usize) -> Option<ItemId> {
        let key = id_of(seq).unwrap_or_default().to_string();
        if let Some(id) = self.seq_items.get(&key) {
            return Some(*id);
        }
        if depth > MAX_DEPTH {
            self.report.warn("sequences are nested too deeply; deeper levels were skipped");
            return None;
        }
        if !self.in_progress.insert(key.clone()) {
            self.report.warn("a sequence contains itself; the nesting was cut");
            return None;
        }
        let name = child_text(seq, "Name").filter(|s| !s.trim().is_empty()).unwrap_or("Sequence").trim().to_string();

        // Settings: the sequence's own VideoSettings when present, else the video track group's.
        let (mut width, mut height, mut rate) = (1920, 1080, FrameRate::FPS_24);
        let video_group = self.track_group(seq, MEDIA_TYPE_VIDEO);
        let settings_src =
            self.objs.resolve_path(seq, &["VideoSettings"]).or_else(|| video_group.as_ref().and_then(|g| self.objs.resolve_path(*g, &["TrackGroup"])));
        if let Some(s) = settings_src {
            if let Some(fr) = child_text(s, "FrameRect").map(tuple_i64)
                && fr.len() >= 4
                && fr[2] > fr[0]
                && fr[3] > fr[1]
            {
                width = u32::try_from(fr[2] - fr[0]).unwrap_or(width);
                height = u32::try_from(fr[3] - fr[1]).unwrap_or(height);
            }
            if let Some(r) = child_i64(s, "FrameRate").and_then(rate_from_tpf) {
                rate = r;
            }
        }
        let item = self.b.reserve_sequence(&name, settings_for(rate, width, height, false), None);
        self.seq_items.insert(key.clone(), item);

        let mut out = empty_sequence(settings_for(rate, width, height, false));
        // Track definitions and clips first; transitions afterwards, once the clips are placed.
        for (kind, guid) in [(TrackKind::Video, MEDIA_TYPE_VIDEO), (TrackKind::Audio, MEDIA_TYPE_AUDIO)] {
            let Some(group) = self.track_group(seq, guid) else { continue };
            let Some(tracks) = self.objs.resolve_path(group, &["TrackGroup", "Tracks"]) else { continue };
            for (index, t) in elements(tracks).take(64).enumerate() {
                let Some(tnode) = self.objs.resolve(t) else { continue };
                if let Some(items) = self.objs.resolve_path(tnode, &["ClipTrack", "ClipItems"]) {
                    for c in elements(items).take(MAX_ITEMS) {
                        let Some(cnode) = self.objs.resolve(c) else { continue };
                        if let Some(ti) = self.clip_item(cnode, kind, item, rate, (width, height)) {
                            place(&mut self.b, &mut out, kind, index, ti, self.report);
                        }
                    }
                }
                // Track properties (locked, muted, sync lock, name) from the nested <Track>.
                ensure_len(&mut self.b, &mut out, kind, index + 1);
                if let Some(p) = find_desc(tnode, "Track") {
                    let slot = &mut out.tracks_mut(kind)[index];
                    slot.locked = child_bool(p, "IsLocked").unwrap_or(false);
                    slot.muted = child_bool(p, "IsMuted").unwrap_or(false);
                    slot.sync_lock = child_bool(p, "IsSyncLocked").unwrap_or(false);
                    if let Some(n) = child_text(p, "Name").filter(|n| !n.trim().is_empty()) {
                        slot.name = n.trim().to_string();
                    }
                }
            }
        }
        if let Some(markers) = self.objs.resolve_path(seq, &["MarkerOwner", "Markers"]) {
            out.markers = self.markers(markers, rate);
        }
        // Transitions need the clips in place, so they run once the sequence content is stored.
        self.b.put_sequence(item, out);
        for (kind, guid) in [(TrackKind::Video, MEDIA_TYPE_VIDEO), (TrackKind::Audio, MEDIA_TYPE_AUDIO)] {
            let Some(group) = self.track_group(seq, guid) else { continue };
            let Some(tracks) = self.objs.resolve_path(group, &["TrackGroup", "Tracks"]) else { continue };
            for (index, t) in elements(tracks).take(64).enumerate() {
                let Some(tnode) = self.objs.resolve(t) else { continue };
                if let Some(defs) = self.objs.resolve_path(tnode, &["ClipTrack", "TransitionItems"]) {
                    let defs: Vec<Node<'a, 'i>> = elements(defs).filter_map(|c| self.objs.resolve(c)).take(1024).collect();
                    for c in defs {
                        self.transition(c, kind, item, index, rate);
                    }
                }
            }
        }
        self.in_progress.remove(&key);
        Some(item)
    }

    /// The track group of a sequence matching `guid` (by `MediaType`, else by group index).
    fn track_group(&self, seq: Node<'a, 'i>, guid: &str) -> Option<Node<'a, 'i>> {
        let groups = child(seq, "TrackGroups")?;
        for g in elements(groups).take(8) {
            let mt = find_desc(g, "TrackGroup").and_then(|tg| child_text(tg, "MediaType")).or_else(|| child_text(g, "MediaType"));
            let second = self.objs.resolve_path(g, &["Second"]);
            if mt.is_some_and(|m| m.eq_ignore_ascii_case(guid)) {
                return second;
            }
            // The `First` element of the pair carries the media-type GUID in plain text.
            if child_text(g, "First").is_some_and(|f| f.eq_ignore_ascii_case(guid)) {
                return second;
            }
            if mt.is_none()
                && child_text(g, "First").is_none()
                && let Some(idx) = g.attribute("Index")
            {
                let want = if guid == MEDIA_TYPE_VIDEO { "0" } else { "1" };
                if idx == want {
                    return second;
                }
            }
        }
        None
    }

    /// A `ClipTrackItem`: timing, source, intrinsic effect parameters, markers.
    fn clip_item(&mut self, n: Node<'a, 'i>, kind: TrackKind, seq_item: ItemId, rate: FrameRate, frame: (u32, u32)) -> Option<TrackItem> {
        // Timing lives in the nested `<TrackItem>` wrapper; effect chains and links too.
        let holder = child(n, "TrackItem").unwrap_or(n);
        let start = child_i64(holder, "Start")?;
        let end = child_i64(holder, "End")?;
        if end <= start {
            return None;
        }
        let in_p = child_i64(holder, "InPoint").unwrap_or(0);
        let out_p = child_i64(holder, "OutPoint").unwrap_or(in_p);
        let kind = match child_text(holder, "MediaType") {
            Some(m) if m.eq_ignore_ascii_case(MEDIA_TYPE_VIDEO) => TrackKind::Video,
            Some(m) if m.eq_ignore_ascii_case(MEDIA_TYPE_AUDIO) => TrackKind::Audio,
            _ => kind,
        };
        let item = self.source_item(n, kind, frame)?;

        let (start_t, dur, src_in) = (Tick(start), Tick(end - start), Tick(in_p.min(out_p)));
        let src_len = (out_p - in_p).abs();
        let mut speed = 1.0;
        if src_len > 0 && src_len != dur.0 && dur.0 > 0 {
            speed = ((src_len as f64 / dur.0 as f64) * 10_000.0).round() / 10_000.0;
        }
        let mut ti = self.b.clip(item, kind, &self.clip_name(n, item), start_t, dur, src_in);
        ti.speed = speed;
        ti.enabled = child_bool(n, "IsEnabled").unwrap_or(true);
        if let Some(l) = self.label_of(n) {
            ti.label = l;
        }
        // Clip markers: on the track item itself, the placed clip object, or inside that
        // clip object's `Clip` wrapper, depending on the Premiere version.
        let markers = self
            .objs
            .resolve_path(n, &["MarkerOwner", "Markers"])
            .or_else(|| self.objs.resolve_path(n, &["Clip", "MarkerOwner", "Markers"]))
            .or_else(|| self.objs.resolve_path(n, &["Clip", "Clip", "MarkerOwner", "Markers"]));
        if let Some(markers) = markers {
            ti.markers = self.markers(markers, rate);
        }
        self.effects(n, &mut ti, frame);
        let cid = ti.id;
        if let Some(id) = id_of(n) {
            self.placed.insert(id.to_string(), (seq_item, cid));
        }
        // Linked items (an A/V pair Premiere keeps in step).
        if let Some(linked) = child(holder, "LinkedItems") {
            let objs = self.objs;
            for l in elements(linked).take(16) {
                if let Some(other) = objs.resolve(l).and_then(id_of)
                    && let Some((s, c)) = self.placed.get(other)
                    && *s == seq_item
                {
                    self.links.push((seq_item, cid, *c));
                }
            }
        }
        Some(ti)
    }

    fn clip_name(&self, n: Node<'a, 'i>, item: ItemId) -> String {
        for holder in [n, self.objs.resolve_path(n, &["Clip"]).unwrap_or(n)] {
            if let Some(name) = child_text(holder, "Name").filter(|s| !s.trim().is_empty()) {
                return name.trim().to_string();
            }
        }
        self.b.p.item(item).map(|i| i.name.clone()).unwrap_or_else(|| "Clip".into())
    }

    /// Follow a placed clip to its media item or nested sequence.
    fn source_item(&mut self, n: Node<'a, 'i>, kind: TrackKind, frame: (u32, u32)) -> Option<ItemId> {
        // Newer projects name the project item directly; older ones link the clip object.
        if let Some(pi) = self.objs.resolve_path(n, &["ProjectItem"])
            && let Some(mc) = self.objs.resolve_path(pi, &["MasterClip"])
            && let Some(id) = self.master_clip_item(mc, &self.name_of(pi))
        {
            return Some(id);
        }
        let clip = self.objs.resolve_path(n, &["Clip"])?;
        let clip = match child(clip, "CopiedFrom") {
            Some(cf) => self.objs.resolve(cf).unwrap_or(clip),
            None => clip,
        };
        if let Some(id) = self.clip_source(clip, "Untitled") {
            return Some(id);
        }
        // Offline placeholder so the edit still lands.
        let _ = (kind, frame);
        self.report.warn("a clip item's media reference could not be resolved; it was skipped");
        None
    }

    /// Intrinsic effect parameters of a placed clip: Motion, Opacity, Volume, Pan.
    ///
    /// Premiere nests parameters in component wrappers (`Component → Params → Param` refs); a few
    /// versions inline them, so both shapes are read.
    fn effects(&mut self, n: Node<'a, 'i>, ti: &mut TrackItem, frame: (u32, u32)) {
        let chain = find_desc(n, "Components").or_else(|| find_desc(n, "ComponentChain")).or_else(|| self.objs.resolve_path(n, &["Clip", "Components"]));
        let Some(chain) = chain else { return };
        let objs = self.objs;
        let mut unknown = 0usize;
        for entry in chain.children().filter(|c| c.is_element()).take(128) {
            let comp = objs.resolve(entry).unwrap_or(entry);
            let params: Vec<Node<'a, 'i>> = if let Some(ps) = objs.resolve_path(comp, &["Component", "Params"]) {
                elements(ps).take(64).filter_map(|p| objs.resolve(p)).collect()
            } else if child(comp, "Name").is_some() {
                vec![comp]
            } else {
                continue;
            };
            let mut handled = false;
            for p in &params {
                if let Some(name) = child_text(*p, "Name").map(str::trim) {
                    handled |= self.apply_param(*p, name, ti, frame);
                }
            }
            if !params.is_empty() && !handled {
                unknown += 1;
            }
        }
        if unknown > 0 {
            self.report.warn(format!("effect(s) on \"{}\" are not supported and were skipped", ti.name));
        }
    }

    /// Map one Premiere component parameter onto ours. Returns whether it was recognized.
    fn apply_param(&mut self, p: Node<'a, 'i>, name: &str, ti: &mut TrackItem, frame: (u32, u32)) -> bool {
        let kfs = param_keyframes(p);
        let animated = kfs.len() > 1;
        if animated && !self.warned_interp {
            self.warned_interp = true;
            self.report.info("keyframe interpolation styles are imported as linear");
        }
        let scalar = |v: &[f64]| v.first().copied();
        let vector = |v: &[f64]| v.first().zip(v.get(1)).map(|(x, y)| ParamValue::Vec2(filmcraft_geom::Vec2::new(*x, *y)));
        // Premiere's Volume is in dB when `UnitsString` says so, else a linear gain.
        let units_db = child_text(p, "UnitsString").is_some_and(|u| u.eq_ignore_ascii_case("db"));
        let make = |to_value: &dyn Fn(&[f64]) -> Option<ParamValue>| -> Option<Param> {
            if animated {
                let mut keys = Vec::with_capacity(kfs.len());
                for (t, v) in &kfs {
                    let value = to_value(v)?;
                    keys.push(Keyframe::new(*t, value));
                }
                Some(Param { value: keys.first().map(|k| k.value.clone()).unwrap_or(ParamValue::Float(0.0)), keyframes: keys })
            } else {
                let v = kfs.first().map(|(_, v)| v.as_slice())?;
                Some(Param::new(to_value(v)?))
            }
        };
        let (fw, fh) = (frame.0 as f64, frame.1 as f64);
        match name.to_ascii_lowercase().as_str() {
            "position" => {
                let Some(pr) = make(&vector) else { return true };
                let is_default =
                    |x: &Param| !x.is_animated() && x.value.as_vec2().is_some_and(|v| (v.x - fw / 2.0).abs() < 1e-9 && (v.y - fh / 2.0).abs() < 1e-9);
                if pr.is_animated() || !is_default(&pr) {
                    set_param(ti, "motion", "position", pr);
                }
                true
            }
            "scale" => {
                let Some(pr) = make(&|v| scalar(v).map(|f| ParamValue::Float(f.clamp(-10_000.0, 10_000.0)))) else { return true };
                let is_default = |x: &Param| !x.is_animated() && x.value.as_f64().is_some_and(|f| (f - 100.0).abs() < 1e-9);
                if pr.is_animated() || !is_default(&pr) {
                    set_param(ti, "motion", "scale", pr);
                }
                true
            }
            "scale width" => {
                let Some(pr) = make(&|v| scalar(v).map(|f| ParamValue::Float(f.clamp(-10_000.0, 10_000.0)))) else { return true };
                if pr.is_animated() || pr.value.as_f64().is_some_and(|f| (f - 100.0).abs() > 1e-9) {
                    set_param(ti, "motion", "scale_width", pr);
                }
                true
            }
            "rotation" => {
                let Some(pr) = make(&|v| scalar(v).map(|f| ParamValue::Float(f.clamp(-36_000.0, 36_000.0)))) else { return true };
                if pr.is_animated() || pr.value.as_f64().is_some_and(|f| f.abs() > 1e-9) {
                    set_param(ti, "motion", "rotation", pr);
                }
                true
            }
            "anchor point" => {
                let Some(pr) = make(&vector) else { return true };
                if pr.is_animated() || pr.value.as_vec2().is_some_and(|v| v.x.abs() > 1e-9 || v.y.abs() > 1e-9) {
                    set_param(ti, "motion", "anchor", pr);
                }
                true
            }
            "opacity" => {
                let Some(pr) = make(&|v| scalar(v).map(|f| ParamValue::Float(f.clamp(-100.0, 100.0)))) else { return true };
                if pr.is_animated() || pr.value.as_f64().is_some_and(|f| (f - 100.0).abs() > 1e-9) {
                    set_param(ti, "opacity", "opacity", pr);
                }
                true
            }
            "volume" | "level" => {
                let to_db = |v: &[f64]| {
                    scalar(v).map(|f| {
                        let db = if units_db { f.clamp(-96.0, 48.0) } else { crate::common::gain_to_db(f.clamp(0.0, 32.0)) };
                        ParamValue::Float(db)
                    })
                };
                let Some(pr) = make(&to_db) else { return true };
                if pr.is_animated() || pr.value.as_f64().is_some_and(|f| f.abs() > 1e-9) {
                    set_param(ti, "volume", "level", pr);
                }
                true
            }
            "pan" => {
                let Some(pr) = make(&|v| scalar(v).map(|f| ParamValue::Float(f.clamp(-1.0, 1.0) * 100.0))) else { return true };
                if pr.is_animated() || pr.value.as_f64().is_some_and(|f| f.abs() > 1e-9) {
                    set_param(ti, "panner", "balance", pr);
                }
                true
            }
            _ => false,
        }
    }

    /// A `TransitionTrackItem` on track `index` of the sequence.
    fn transition(&mut self, n: Node<'a, 'i>, kind: TrackKind, seq_item: ItemId, index: usize, rate: FrameRate) {
        let holder = child(n, "TrackItem").unwrap_or(n);
        let start = child_i64(holder, "Start").unwrap_or_default();
        let end = child_i64(holder, "End").unwrap_or(start);
        if end <= start {
            return;
        }
        let (s, dur) = (Tick(start), Tick(end - start));
        let end_t = Tick(end);
        let def = self.objs.resolve_path(n, &["Transition"]);
        let (name, duration, reverse) = match def {
            Some(d) => {
                let name = child_text(d, "Name")
                    .or_else(|| child_text(d, "DisplayName"))
                    .or_else(|| d.children().find(|c| c.is_element() && c.tag_name().name().contains("TransitionType")).map(text))
                    .unwrap_or("Cross Dissolve")
                    .trim()
                    .to_string();
                (name, child_i64(d, "Duration").filter(|d| *d > 0).map(Tick).unwrap_or(dur), child_bool(d, "IsReverse").unwrap_or(false))
            }
            None => ("Cross Dissolve".into(), dur, false),
        };
        let effect = transition_effect(&name, kind == TrackKind::Audio, self.report);
        // From/To references when present; adjacency otherwise.
        let from = self.objs.resolve_path(holder, &["From"]).and_then(id_of).and_then(|id| self.placed.get(id).filter(|p| p.0 == seq_item).map(|p| p.1));
        let to = self.objs.resolve_path(holder, &["To"]).and_then(id_of).and_then(|id| self.placed.get(id).filter(|p| p.0 == seq_item).map(|p| p.1));
        let Some(seq) = self.b.p.sequence(seq_item) else { return };
        let track = seq.tracks(kind).get(index);
        let (from, to) = match (from, to) {
            (Some(f), Some(t)) => (Some(f), Some(t)),
            (f, t) => {
                let near = track.map(|tr| {
                    let from = f.or_else(|| tr.items.iter().filter(|i| i.end() >= s && i.end() <= end_t).max_by_key(|i| i.end()).map(|i| i.id));
                    let to = t.or_else(|| tr.items.iter().filter(|i| i.start >= s && i.start <= end_t).min_by_key(|i| i.start).map(|i| i.id));
                    (from, to)
                });
                near.unwrap_or((f, t))
            }
        };
        if from.is_none() && to.is_none() {
            self.report.warn(format!("transition \"{name}\" at {} has no adjacent clip; skipped", rate.frame_at(s)));
            return;
        }
        // Alignment from the cut position, like our other importers.
        let find_edge = |id: Option<ClipId>, edge: usize| -> Option<Tick> {
            let id = id?;
            track?.items.iter().find(|i| i.id == id).map(|i| if edge == 0 { i.end() } else { i.start })
        };
        let cut = find_edge(from, 0).or_else(|| find_edge(to, 1));
        let align = match cut {
            Some(c) if c == s => TransitionAlign::StartAtCut,
            Some(c) if c == end_t => TransitionAlign::EndAtCut,
            _ => TransitionAlign::CenterAtCut,
        };
        let tid = TransitionId(self.b.alloc());
        if let Some(seq) = self.b.p.sequence_mut(seq_item)
            && let Some(t) = seq.tracks_mut(kind).get_mut(index)
        {
            t.transitions.push(Transition { id: tid, effect, start: s, duration, from, to, align, reverse });
            t.sort();
        }
    }

    /// Marker objects under a `Markers` container.
    fn markers(&mut self, markers: Node<'a, 'i>, rate: FrameRate) -> Vec<Marker> {
        let mut out = Vec::new();
        let objs = self.objs;
        for entry in elements(markers).take(MAX_MARKERS) {
            let m = objs.resolve(entry).unwrap_or(entry);
            let name = child_text(m, "Name").filter(|s| !s.trim().is_empty()).unwrap_or("Marker").trim().to_string();
            let Some(start) = child_i64(m, "Start") else { continue };
            let duration = child_i64(m, "Duration").filter(|d| *d > 0).map(Tick).unwrap_or(rate.frame_duration());
            let comment = child_text(m, "Comment").unwrap_or("").to_string();
            out.push(Marker {
                id: MarkerId(self.b.alloc()),
                start: Tick(start.max(0)),
                duration,
                name,
                comment,
                kind: MarkerKind::Comment,
                color: Label::Yellow,
            });
        }
        out
    }

    /// Apply the A/V link pairs collected while reading tracks.
    fn resolve_links(&mut self) {
        let pending = std::mem::take(&mut self.links);
        let mut group_of: HashMap<(ItemId, ClipId), u64> = HashMap::new();
        for (seq, a, b) in pending {
            if a == b {
                continue;
            }
            let existing = group_of.get(&(seq, a)).copied().or_else(|| group_of.get(&(seq, b)).copied());
            let g = existing.unwrap_or_else(|| self.b.link_id());
            group_of.insert((seq, a), g);
            group_of.insert((seq, b), g);
        }
        for ((seq, clip), g) in group_of {
            if let Some(s) = self.b.p.sequence_mut(seq) {
                for t in s.all_tracks_mut() {
                    if let Some(c) = t.items.iter_mut().find(|c| c.id == clip) {
                        c.link = Some(g);
                    }
                }
            }
        }
    }
}

/// Place `item` on track `preferred`, moving to a new track when it would overlap.
fn place(b: &mut Builder, seq: &mut Sequence, kind: TrackKind, preferred: usize, item: TrackItem, report: &mut Report) {
    let mut i = preferred;
    loop {
        ensure_len(b, seq, kind, i + 1);
        let t = &mut seq.tracks_mut(kind)[i];
        if !t.items.iter().any(|o| o.start < item.end() && item.start < o.end()) {
            t.items.push(item);
            t.sort();
            return;
        }
        if i == preferred {
            report.warn("overlapping clip items on one track; the later one was moved to a new track");
        }
        i += 1;
        if i > 64 {
            return;
        }
    }
}

fn ensure_len(b: &mut Builder, seq: &mut Sequence, kind: TrackKind, n: usize) {
    while seq.tracks(kind).len() < n {
        let i = seq.tracks(kind).len();
        let t: Track = b.track(kind, i);
        seq.tracks_mut(kind).push(t);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use filmcraft_project::{BinEntry, ItemKind};

    fn opts() -> ImportOptions {
        ImportOptions::default()
    }

    /// A minimal but structurally faithful project: one bin, one movie with a video+audio master
    /// clip, one sequence with a placed A/V pair, a cross dissolve, motion keyframes and markers.
    fn sample_project() -> String {
        let tpf24 = TICKS_PER_SECOND / 24;
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<PremiereData Version="3">
  <Project ObjectRef="1"/>
  <Project ObjectID="1" ClassID="62ad66dd-0dcd-42da-a660-6d8fbde94876" Version="22">
    <RootProjectItem ObjectURef="root"/>
  </Project>
  <RootProjectItem ObjectUID="root" ClassID="1c307a89-9318-47d7-a583-bf2553736543" Version="1">
    <ProjectItemContainer Version="1">
      <Items Version="1">
        <Item Index="0" ObjectURef="bin1"/>
        <Item Index="1" ObjectURef="seqitem"/>
      </Items>
    </ProjectItemContainer>
    <ProjectItem Version="1"><Node Version="1"><ID>1000000</ID></Node><Name>Root Bin</Name></ProjectItem>
  </RootProjectItem>
  <BinProjectItem ObjectUID="bin1" ClassID="d5c9c126-5a3f-4e3b-9a6f-8e1a8b5c9d2e" Version="1">
    <ProjectItemContainer Version="1">
      <Items Version="1">
        <Item Index="0" ObjectURef="clipitem"/>
      </Items>
    </ProjectItemContainer>
    <ProjectItem Version="1"><Node Version="1"><Properties Version="1"><asl.clip.label.name>BE.Prefs.LabelColors.5</asl.clip.label.name></Properties></Node><Name>Interviews</Name></ProjectItem>
  </BinProjectItem>
  <ClipProjectItem ObjectUID="clipitem" ClassID="cb4e0ed7-aca1-4171-8525-e3658dec06dd" Version="1">
    <ProjectItem Version="1"><Node Version="1"><Properties Version="1"><asl.clip.label.name>BE.Prefs.LabelColors.1</asl.clip.label.name></Properties></Node><Name>A001_C001.mov</Name></ProjectItem>
    <MasterClip ObjectURef="mc"/>
  </ClipProjectItem>
  <MasterClip ObjectUID="mc" ClassID="fb11c33a-b0a9-4465-aa94-b6d5db2628cf" Version="5">
    <Clips Version="1">
      <Clip Index="0" ObjectRef="vclip"/>
      <Clip Index="1" ObjectRef="aclip"/>
    </Clips>
    <Name>A001_C001.mov</Name>
  </MasterClip>
  <VideoClip ObjectID="vclip" ClassID="9308dbef-2440-4acb-9ab2-953b9a4e82ec" Version="7">
    <Clip Version="14"><Source ObjectRef="vsrc"/></Clip>
  </VideoClip>
  <AudioClip ObjectID="aclip" ClassID="b8830d03-de02-41ee-84ec-fe566dc70cd9" Version="7">
    <Clip Version="14"><Source ObjectRef="asrc"/></Clip>
  </AudioClip>
  <VideoMediaSource ObjectID="vsrc" ClassID="e64ddf74-8fac-4682-8aa8-0e0ca2248949" Version="2">
    <MediaSource Version="3"><Media ObjectURef="media"/></MediaSource>
  </VideoMediaSource>
  <AudioMediaSource ObjectID="asrc" ClassID="f588da05-fc2a-4fbc-9383-74d653b379e3" Version="2">
    <MediaSource Version="3"><Media ObjectURef="media"/></MediaSource>
  </AudioMediaSource>
  <Media ObjectUID="media" ClassID="7a5c103e-f3ac-4391-b6b4-7cc3d2f9a7ff" Version="19">
    <ActualMediaFilePath>/media/A001_C001.mov</ActualMediaFilePath>
    <Title>A001_C001.mov</Title>
    <VideoStream ObjectRef="vstream"/>
    <AudioStream ObjectRef="astream"/>
    <FilePath>/media/A001_C001.mov</FilePath>
  </Media>
  <VideoStream ObjectID="vstream" ClassID="a36e4719-3ec6-4a0c-ab11-8b4aab377aa5" Version="12">
    <FrameRect>0,0,1920,1080</FrameRect>
    <Duration>{dur}</Duration>
    <FrameRate>{tpf}</FrameRate>
  </VideoStream>
  <AudioStream ObjectID="astream" ClassID="0b5cf52f-2b85-4863-890b-8844b64ecfe9" Version="8">
    <FrameRate>5292000</FrameRate>
    <ChannelType>1</ChannelType>
    <ChannelType>1</ChannelType>
  </AudioStream>
  <ClipProjectItem ObjectUID="seqitem" ClassID="cb4e0ed7-aca1-4171-8525-e3658dec06dd" Version="1">
    <ProjectItem Version="1"><Node Version="1"></Node><Name>Seq 01</Name></ProjectItem>
    <MasterClip ObjectURef="seqmc"/>
  </ClipProjectItem>
  <MasterClip ObjectUID="seqmc" ClassID="fb11c33a-b0a9-4465-aa94-b6d5db2628cf" Version="5">
    <Clips Version="1"><Clip Index="0" ObjectRef="seqv"/></Clips>
    <Name>Seq 01</Name>
  </MasterClip>
  <VideoClip ObjectID="seqv" ClassID="9308dbef-2440-4acb-9ab2-953b9a4e82ec" Version="7">
    <Clip Version="14"><Source ObjectRef="seqsrc"/></Clip>
  </VideoClip>
  <VideoSequenceSource ObjectID="seqsrc" ClassID="4752dfa9-7a7e-4a3b-a25b-cafde1a8d036" Version="3">
    <SequenceSource Version="3"><Sequence ObjectURef="seq"/></SequenceSource>
  </VideoSequenceSource>
  <Sequence ObjectUID="seq" ClassID="6a15d903-8739-11d5-af2d-9b7855ad8974" Version="10">
    <TrackGroups Version="1">
      <TrackGroup Version="1" Index="0"><Second ObjectRef="vg"/><First>{video_guid}</First></TrackGroup>
      <TrackGroup Version="1" Index="1"><Second ObjectRef="ag"/><First>{audio_guid}</First></TrackGroup>
    </TrackGroups>
    <MarkerOwner Version="1"><Markers ObjectRef="smarkers"/></MarkerOwner>
    <Name>Seq 01</Name>
  </Sequence>
  <VideoTrackGroup ObjectID="vg" ClassID="9e9abf7a-0918-49c2-91ae-991b5dde77bb" Version="8">
    <TrackGroup Version="1">
      <Tracks Version="1"><Track Index="0" ObjectRef="vt"/></Tracks>
      <FrameRate>{tpf}</FrameRate>
    </TrackGroup>
    <FrameRect>0,0,1920,1080</FrameRect>
  </VideoTrackGroup>
  <AudioTrackGroup ObjectID="ag" ClassID="9b9238b9-53a8-4cc3-b03f-b36246d052e6" Version="5">
    <TrackGroup Version="1">
      <Tracks Version="1"><Track Index="0" ObjectRef="at"/></Tracks>
      <FrameRate>5292000</FrameRate>
    </TrackGroup>
  </AudioTrackGroup>
  <VideoClipTrack ObjectID="vt" ClassID="f68dcd81-8805-11d5-af2d-9bfa89d4ddd4" Version="1">
    <ClipTrack Version="2">
      <Track Version="3">
        <IsSyncLocked>true</IsSyncLocked><Index>0</Index><IsMuted>false</IsMuted><IsLocked>false</IsLocked><ID>1</ID>
      </Track>
      <TransitionItems Version="3"><Index>0</Index><MediaType>{video_guid}</MediaType><TrackItem ObjectRef="tr1"/></TransitionItems>
      <ClipItems Version="3">
        <Index>0</Index><MediaType>{video_guid}</MediaType>
        <TrackItem ObjectRef="c1"/>
        <TrackItem ObjectRef="c2"/>
      </ClipItems>
    </ClipTrack>
  </VideoClipTrack>
  <AudioClipTrack ObjectID="at" ClassID="097f6203-99ae-11d5-84f2-8cf14bde7040" Version="1">
    <ClipTrack Version="2">
      <Track Version="3"><IsMuted>false</IsMuted><IsLocked>false</IsLocked></Track>
      <TransitionItems Version="3"><MediaType>{audio_guid}</MediaType></TransitionItems>
      <ClipItems Version="3">
        <MediaType>{audio_guid}</MediaType>
        <TrackItem ObjectRef="a1"/>
        <TrackItem ObjectRef="a2"/>
      </ClipItems>
    </ClipTrack>
  </AudioClipTrack>
  <ClipTrackItem ObjectID="c1" ClassID="9c3b872a-24a8-4efc-937d-1e8a3197822f" Version="4">
    <TrackItem Version="4">
      <Start>0</Start><End>{half}</End><InPoint>0</InPoint><OutPoint>{half}</OutPoint>
      <MediaType>{video_guid}</MediaType>
      <LinkedItems Version="1"><LinkedTrackItem ObjectRef="a1"/></LinkedItems>
      <Components Version="1">
        <Component ObjectRef="motionwrapper"/>
      </Components>
    </TrackItem>
    <Clip ObjectRef="pvclip"/>
  </ClipTrackItem>
  <ClipTrackItem ObjectID="c2" ClassID="9c3b872a-24a8-4efc-937d-1e8a3197822f" Version="4">
    <TrackItem Version="4">
      <Start>{half}</Start><End>{dur}</End><InPoint>{half}</InPoint><OutPoint>{dur}</OutPoint>
      <MediaType>{video_guid}</MediaType>
    </TrackItem>
    <Clip ObjectRef="pvclip2"/>
  </ClipTrackItem>
  <ClipTrackItem ObjectID="a1" ClassID="9c3b872a-24a8-4efc-937d-1e8a3197822f" Version="4">
    <TrackItem Version="4">
      <Start>0</Start><End>{half}</End><InPoint>0</InPoint><OutPoint>{half}</OutPoint>
      <MediaType>{audio_guid}</MediaType>
      <LinkedItems Version="1"><LinkedTrackItem ObjectRef="c1"/></LinkedItems>
    </TrackItem>
    <Clip ObjectRef="paclip"/>
  </ClipTrackItem>
  <ClipTrackItem ObjectID="a2" ClassID="9c3b872a-24a8-4efc-937d-1e8a3197822f" Version="4">
    <TrackItem Version="4">
      <Start>{half}</Start><End>{dur}</End><InPoint>{half}</InPoint><OutPoint>{dur}</OutPoint>
      <MediaType>{audio_guid}</MediaType>
    </TrackItem>
    <Clip ObjectRef="paclip2"/>
  </ClipTrackItem>
  <VideoClip ObjectID="pvclip" ClassID="9308dbef-2440-4acb-9ab2-953b9a4e82ec" Version="7">
    <Clip Version="14"><Source ObjectRef="vsrc"/><MarkerOwner Version="1"><Markers ObjectRef="cmarkers"/></MarkerOwner></Clip>
    <Name>A - first half</Name>
  </VideoClip>
  <VideoClip ObjectID="pvclip2" ClassID="9308dbef-2440-4acb-9ab2-953b9a4e82ec" Version="7">
    <Clip Version="14"><Source ObjectRef="vsrc"/></Clip>
    <Name>A - second half</Name>
  </VideoClip>
  <AudioClip ObjectID="paclip" ClassID="b8830d03-de02-41ee-84ec-fe566dc70cd9" Version="7">
    <Clip Version="14"><Source ObjectRef="asrc"/></Clip>
    <Name>A - first half</Name>
  </AudioClip>
  <AudioClip ObjectID="paclip2" ClassID="b8830d03-de02-41ee-84ec-fe566dc70cd9" Version="7">
    <Clip Version="14"><Source ObjectRef="asrc"/></Clip>
    <Name>A - second half</Name>
  </AudioClip>
  <VideoComponent ObjectID="motionwrapper" ClassID="f19fda22-bbbe-4f9a-9a68-357a40c2a7b0" Version="2">
    <Component Version="3">
      <Node Version="1"/>
      <Intrinsic>true</Intrinsic>
      <Params Version="1">
        <Param Index="0" ObjectRef="motionparam"/>
      </Params>
      <DisplayName>Motion</DisplayName>
      <Bypass>false</Bypass>
      <ID>1</ID>
    </Component>
  </VideoComponent>
  <VideoComponentParam ObjectID="motionparam" ClassID="a714635e-a628-4b27-9d59-77eba47dbc1a" Version="9">
    <Name>Position</Name>
    <UnitsString></UnitsString>
    <StartKeyframe>0,960.,540.</StartKeyframe>
    <Keyframes>0,900.,540.
{halfk},1020.,540.</Keyframes>
  </VideoComponentParam>
  <TransitionTrackItem ObjectID="tr1" ClassID="f1e9d8c1-651f-4a4f-9a3a-6a0a5b7c5d9e" Version="1">
    <TrackItem Version="2">
      <Start>{q}</Start><End>{threeq}</End>
      <From ObjectRef="c1"/><To ObjectRef="c2"/>
    </TrackItem>
    <Transition ObjectRef="td"/>
  </TransitionTrackItem>
  <Transition ObjectID="td" ClassID="9b3f872a-24a8-4efc-937d-1e8a3197822f" Version="1">
    <Alignment>1</Alignment><Duration>{q}</Duration><Name>Cross Dissolve</Name>
  </Transition>
  <Markers ObjectID="smarkers" ClassID="bee50706-b524-416c-9f03-b596ce5f6866" Version="1">
    <Marker ObjectID="m1"><Name>Look here</Name><Start>{half}</Start><Duration>0</Duration><Comment>Check focus</Comment></Marker>
  </Markers>
  <Markers ObjectID="cmarkers" ClassID="bee50706-b524-416c-9f03-b596ce5f6866" Version="1">
    <Marker ObjectID="m2"><Name>Bad take</Name><Start>{half}</Start><Duration>0</Duration><Comment></Comment></Marker>
  </Markers>
</PremiereData>
"#,
            dur = TICKS_PER_SECOND * 4,
            half = TICKS_PER_SECOND * 2,
            halfk = TICKS_PER_SECOND,
            tpf = tpf24,
            threeq = TICKS_PER_SECOND * 2 + TICKS_PER_SECOND / 2,
            q = TICKS_PER_SECOND / 2,
            video_guid = MEDIA_TYPE_VIDEO,
            audio_guid = MEDIA_TYPE_AUDIO,
        )
    }

    fn gzip(data: &[u8]) -> Vec<u8> {
        use std::io::Write;
        let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        e.write_all(data).ok();
        e.finish().ok().unwrap_or_default()
    }

    #[test]
    fn sniffs_gzip_and_xml() {
        assert!(sniff(b"<?xml version=\"1.0\"?><PremiereData Version=\"3\">"));
        assert!(sniff(&gzip(sample_project().as_bytes())));
        assert!(!sniff(b"<?xml version=\"1.0\"?><xmeml/>"));
        assert!(!sniff(b"garbage"));
    }

    #[test]
    fn imports_bins_media_and_sequence() {
        let mut report = Report::default();
        let imported = import(&gzip(sample_project().as_bytes()), &opts(), &mut report).ok().unwrap();
        let p = &imported.project;
        // The root bin holds the bin and the sequence; the movie lives inside the bin.
        let names: Vec<String> = p
            .root
            .children
            .iter()
            .map(|e| match e {
                BinEntry::Bin(b) => b.name.clone(),
                BinEntry::Item(id) => p.item(*id).map(|i| i.name.clone()).unwrap_or_else(|| "?".into()),
            })
            .collect();
        assert!(names.iter().any(|n| n == "Interviews"));
        assert!(names.iter().any(|n| n == "Seq 01"));
        assert_eq!(imported.sequences.len(), 1);
        // The bin contains the movie.
        let bin = p
            .root
            .children
            .iter()
            .find_map(|e| match e {
                BinEntry::Bin(b) if b.name == "Interviews" => Some(b.clone()),
                _ => None,
            })
            .unwrap();
        let movie = bin
            .children
            .iter()
            .find_map(|e| match e {
                BinEntry::Item(id) => Some(*id),
                _ => None,
            })
            .unwrap();
        assert!(p.item(movie).is_some_and(|i| i.name == "A001_C001.mov"));
        match &p.item(movie).unwrap().kind {
            ItemKind::Media(m) => {
                assert_eq!(m.info.video.as_ref().unwrap().width, 1920);
                assert_eq!(m.info.video.as_ref().unwrap().frame_rate, FrameRate::FPS_24);
                assert_eq!(m.info.duration, Tick(TICKS_PER_SECOND * 4));
                assert_eq!(m.info.audio_streams.len(), 1);
                assert_eq!(m.info.audio_streams[0].sample_rate, 48_000);
                assert_eq!(m.info.audio_streams[0].channels, 2);
            }
            other => panic!("not media: {other:?}"),
        }
        // Sequence: 1 video + 1 audio track, 2 clips each, one transition, one marker.
        let seq_id = imported.sequences[0];
        let seq = p.sequence(seq_id).unwrap();
        assert_eq!(seq.video_tracks.len(), 1);
        assert_eq!(seq.audio_tracks.len(), 1);
        assert_eq!(seq.video_tracks[0].items.len(), 2);
        assert_eq!(seq.audio_tracks[0].items.len(), 2);
        assert_eq!(seq.video_tracks[0].items[0].start, Tick(0));
        assert_eq!(seq.video_tracks[0].items[0].duration, Tick(TICKS_PER_SECOND * 2));
        assert_eq!(seq.video_tracks[0].items[0].source_in, Tick(0));
        assert_eq!(seq.video_tracks[0].items[1].start, Tick(TICKS_PER_SECOND * 2));
        assert_eq!(seq.settings.width, 1920);
        assert_eq!(seq.settings.frame_rate, FrameRate::FPS_24);
        assert_eq!(seq.video_tracks[0].transitions.len(), 1);
        let tr = &seq.video_tracks[0].transitions[0];
        assert_eq!(tr.start, Tick(TICKS_PER_SECOND / 2));
        assert_eq!(tr.duration, Tick(TICKS_PER_SECOND / 2));
        assert_eq!(tr.align, TransitionAlign::CenterAtCut);
        assert_eq!(seq.markers.len(), 1);
        assert_eq!(seq.markers[0].name, "Look here");
        // Motion keyframes on the first placed clip.
        let c1 = &seq.video_tracks[0].items[0];
        let motion = c1.effect("motion").unwrap();
        let pos = motion.param("position").unwrap();
        assert!(pos.is_animated());
        assert_eq!(pos.keyframes.len(), 2);
        assert_eq!(pos.keyframes[1].time, Tick(TICKS_PER_SECOND));
        // A/V link between the first video and audio items.
        assert!(c1.link.is_some());
        assert_eq!(c1.link, seq.audio_tracks[0].items[0].link);
        // Clip markers on the placed clip (read through the placed clip object).
        assert_eq!(c1.markers.len(), 1);
    }

    #[test]
    fn static_params_and_volume_units() {
        // A static Position at frame centre is left at the default.
        let needle = format!("<Keyframes>0,900.,540.\n{},1020.,540.</Keyframes>", TICKS_PER_SECOND);
        let xml = sample_project().replace(&needle, "<Keyframes></Keyframes>");
        let mut report = Report::default();
        let imported = import(&gzip(xml.as_bytes()), &opts(), &mut report).ok().unwrap();
        let seq = imported.project.sequence(imported.sequences[0]).unwrap();
        let c1 = &seq.video_tracks[0].items[0];
        let motion = c1.effect("motion").unwrap();
        let pos = motion.param("position").unwrap();
        assert!(!pos.is_animated(), "centre position stays at the default");
    }

    #[test]
    fn plain_xml_projects_from_before_2013_import() {
        let mut report = Report::default();
        let imported = import(sample_project().as_bytes(), &opts(), &mut report).ok().unwrap();
        assert_eq!(imported.sequences.len(), 1);
    }

    #[test]
    fn garbage_is_an_error_never_a_panic() {
        let mut report = Report::default();
        for bytes in [&b""[..], b"\x1f\x8b\x08\x00garbagegarbage", b"<html><body></body></html>", b"<PremiereData Version=\"3\"></PremiereData>"] {
            assert!(import(bytes, &opts(), &mut report).is_err(), "{bytes:?} should not import");
        }
    }

    #[test]
    fn keyframe_tuples_parse() {
        assert_eq!(parse_keyframe_line("254016000000,960.,540."), Some((Tick(TICKS_PER_SECOND), vec![960.0, 540.0])));
        assert_eq!(parse_keyframe_line("0,1."), Some((Tick(0), vec![1.0])));
        assert_eq!(parse_keyframe_line("junk"), None);
        assert_eq!(parse_keyframe_line(""), None);
    }

    #[test]
    fn rates_snap_to_broadcast_rates() {
        assert_eq!(rate_from_tpf(TICKS_PER_SECOND / 24), Some(FrameRate::FPS_24));
        assert_eq!(rate_from_tpf(TICKS_PER_SECOND / 25), Some(FrameRate::FPS_25));
        assert_eq!(rate_from_tpf(TICKS_PER_SECOND / 30), Some(FrameRate::FPS_30));
        assert_eq!(rate_from_tpf(0), None);
        assert_eq!(rate_from_tpf(-5), None);
        assert_eq!(rate_from_tpf(TICKS_PER_SECOND * 2), None);
    }
}
