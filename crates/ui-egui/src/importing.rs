//! Background media imports.
//!
//! Opening a media source is the slow part of importing: a container's index is scanned at open,
//! and a long-GOP recording (an OBS-style Matroska, seconds per GB) is read through to the end.
//! Doing that on the UI thread froze the whole app for every file, where Premiere opens media in
//! the background — so the open runs on one worker thread, and each finished source is imported
//! on the UI thread by the cheap tail of the import ([`commands::import_source`]), appearing in
//! the Project panel as it lands. Files that are not media the streaming path can take (captions,
//! interchange documents, image sequences) and files small enough to open faster than a thread
//! hop stay on the synchronous path, exactly as before.
//!
//! Each worker result requests a repaint when the caller that started the batch could give one
//! (an OS drop, pumped by the frame loop), so a finished source is applied the frame it arrives;
//! a caller without a repaint context (the menu's file dialog) is covered by the frame loop
//! polling the batches. When the last file of a batch lands, after-work runs only for
//! worker-imported items: automatic transcription, ingest copies, and relinking (a newly
//! imported file may bring offline items back).
//!
//! An OS file drop that imports onto the timeline places its clips when the last batch of the
//! drop lands — its files, each dropped folder — all together at the position the drop was made
//! (see [`begin_drop`]): the Timeline lays clips end to end from where they were dropped, so
//! batches placing as they land would place each over the ones before it.
//!
//! A folder dropped from the OS walks on the worker too and becomes a bin named after it, its
//! sub-folders bins inside it — the tree the Media Browser's folder import builds, under the same
//! budget and depth. The walk lands as one message that makes the bins and imports the files not
//! worth a worker; the opened files import into their folder's bin as they arrive.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender, channel};

use egui::Pos2;
use filmcraft_engine::{MediaPool, Services, Session};
use filmcraft_media::SharedSource;
use filmcraft_project::{BinId, ItemId, MediaIdentity};
use serde_json::{Value, json};

use crate::FilmcraftApp;

/// Files at least this large are opened on the worker; smaller ones open faster than a thread
/// hop, and importing them synchronously keeps small imports immediate.
const BACKGROUND_MIN_BYTES: u64 = 16 << 20;

/// Most folders one dropped-folder import walks, and how deep it goes — the engine's own limits
/// (`media_browser`), so an OS drop and the Media Browser build the same tree.
const FOLDER_IMPORT_MAX_FOLDERS: usize = 4096;
const FOLDER_IMPORT_MAX_DEPTH: usize = 32;

/// One file the worker opened, or why it could not.
enum Msg {
    Opened {
        path: String,
        src: SharedSource,
        identity: MediaIdentity,
    },
    Failed {
        path: String,
        error: String,
    },
    /// A dropped folder, walked: the bins to make, the files that import synchronously, and how
    /// many the worker is opening. Arrives before those files' `Opened`/`Failed` messages.
    Folders {
        root: String,
        folders: Vec<WalkedFolder>,
    },
}

/// One folder of a dropped directory, as the walk found it.
struct WalkedFolder {
    name: String,
    /// The folder that holds this one, as an index into the walk; the dropped folder itself has
    /// none — its bin goes into the drop target.
    parent: Option<usize>,
    /// Files on the synchronous path (small media, captions, interchange documents, image
    /// sequences): `file.import`, exactly as the engine's folder import runs them.
    sync: Vec<String>,
    /// Files the worker opens; each imports into this folder's bin as it lands.
    opening: Vec<String>,
}

/// An import in flight.
pub(crate) struct Batch {
    rx: Receiver<Msg>,
    /// Items imported by the background workers so far.
    ids: Vec<ItemId>,
    /// Items already imported synchronously; their command already ran the after-work.
    sync_ids: Vec<ItemId>,
    /// Messages the worker still owes (a folder's walk counts as one).
    remaining: usize,
    /// File opens the worker still owes (a walk is not one): what the status line counts.
    files: usize,
    bin: Option<BinId>,
    /// The OS drop this batch belongs to (see [`begin_drop`]): when it lands, its items join the
    /// drop's held placement.
    group: Option<u64>,
    errors: Vec<String>,
    /// A dropped folder's bins, by file path: where each landed file imports.
    bins: HashMap<String, BinId>,
    /// This batch's undo-merge key (see [`batch_key`]).
    key: String,
}

/// One OS drop that imports onto the timeline: its batches — its files, each dropped folder —
/// hold what they import until every one of them has landed, then place together at the point of
/// the drop.
#[derive(Clone, Copy)]
pub(crate) struct DropGroup {
    /// The drop's identity; its batches carry it until they land.
    id: u64,
}

/// A drop whose batches have not all landed (see [`begin_drop`]).
pub(crate) struct HeldDrop {
    id: u64,
    /// Where the drop was made: where its clips place.
    at: Pos2,
    /// Items landed so far, in landing order.
    ids: Vec<ItemId>,
    /// Batches still owed: one per batch started, plus the registration itself until every batch
    /// of the drop has started.
    pending: usize,
}

fn next_drop_id() -> u64 {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// Begin an OS drop made at `at` that imports onto the timeline (Timeline ▸ drop imports to the
/// timeline): every batch of the drop ([`start`], [`start_folder`]) holds its placed items until
/// the last of them lands, then they place together. `None`: the drop does not place on the
/// timeline, and its batches import into the bin only.
pub(crate) fn begin_drop(app: &mut FilmcraftApp, at: Option<Pos2>) -> Option<DropGroup> {
    let at = at?;
    let id = next_drop_id();
    app.import_drops.push(HeldDrop { id, at, ids: Vec::new(), pending: 1 });
    Some(DropGroup { id })
}

/// Every batch of the drop has started (its registration is no longer owed): the drop places
/// once the last of them lands. A drop without a group placed nothing to begin with.
pub(crate) fn end_drop(app: &mut FilmcraftApp, drop: Option<DropGroup>) {
    if let Some(g) = drop {
        land_drop(app, g.id, Vec::new());
    }
}

/// A batch of the drop starts: its landing is owed to the group.
fn owe_drop(app: &mut FilmcraftApp, drop: &Option<DropGroup>) {
    if let Some(g) = drop
        && let Some(h) = app.import_drops.iter_mut().find(|h| h.id == g.id)
    {
        h.pending += 1;
    }
}

/// A batch of drop `id` landed with `ids`: the drop places everything it held at its point once
/// nothing is owed anymore (every batch started has landed). A group that is gone already placed.
fn land_drop(app: &mut FilmcraftApp, id: u64, ids: Vec<ItemId>) {
    let Some(i) = app.import_drops.iter().position(|h| h.id == id) else { return };
    let h = &mut app.import_drops[i];
    h.pending = h.pending.saturating_sub(1);
    h.ids.extend(ids);
    if h.pending == 0 {
        let h = app.import_drops.remove(i);
        place_drop(app, h.ids, Some(h.at));
    }
}

/// A batch of drop `drop` (a batch started without one never places) landed with `ids`.
fn land_batch(app: &mut FilmcraftApp, drop: &Option<DropGroup>, ids: Vec<ItemId>) {
    if let Some(g) = drop {
        land_drop(app, g.id, ids);
    }
}

/// A fresh undo-merge key: the imports of one batch share it (one undo step), and no two batches
/// share one, so two drops stay two undos even when nothing else was edited between them.
fn batch_key() -> String {
    static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
    format!("import-{}", NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
}

/// Begin importing `paths` into `bin`. Media files large enough for the open to be worth a
/// background worker (see [`BACKGROUND_MIN_BYTES`]) are opened there and imported as they finish;
/// everything else imports synchronously through `file.import`, as before. `drop`, for an OS file
/// drop, holds the batch's items until every batch of the drop has landed, then they place on the
/// timeline together at the point of the drop; `ctx`, when the caller holds one, is asked for a
/// repaint as each worker result arrives.
///
/// Returns the synchronous import's result when nothing went to the background, else what
/// imported here and now plus how many files are still opening (`{"items": [...],
/// "importing": <count>}`, with `errors` when some already failed).
pub(crate) fn start(app: &mut FilmcraftApp, ctx: Option<&egui::Context>, paths: Vec<String>, bin: BinId, drop: Option<DropGroup>) -> Result<Value, String> {
    // the drop's timeline placement waits for every batch of the drop, this one included
    owe_drop(app, &drop);
    // no worker threads on the web: everything imports synchronously
    #[cfg(target_arch = "wasm32")]
    let (streamed, rest): (Vec<String>, Vec<String>) = (Vec::new(), paths);
    #[cfg(not(target_arch = "wasm32"))]
    let (streamed, rest) = {
        let mut streamed = Vec::new();
        let mut rest = Vec::new();
        for path in paths {
            if backgroundable(&*app.session.services, &path) {
                streamed.push(path);
            } else {
                rest.push(path);
            }
        }
        (streamed, rest)
    };
    if streamed.is_empty() {
        // nothing worth a worker: the command does it all, including its after-work
        if rest.is_empty() {
            land_batch(app, &drop, Vec::new());
            return Ok(json!({"items": []}));
        }
        let (r, ids, errors) = sync_import(app, rest, bin);
        if !errors.is_empty() {
            app.ui.status = errors.join("; ");
        }
        land_batch(app, &drop, ids);
        return r;
    }
    // Captions, interchange documents, image sequences and small files use their synchronous paths.
    let key = batch_key();
    let had_sync = !rest.is_empty();
    // a failed command's error is already among `errors` (`sync_import` folds it in)
    let (_, sync_ids, errors) = if rest.is_empty() { (Ok(Value::Null), Vec::new(), Vec::new()) } else { sync_import(app, rest, bin) };
    // the files that land later merge into the same undo step as the synchronous ones
    if had_sync {
        app.session.history.merge_key = Some(key.clone());
    }
    let count = streamed.len();
    let (tx, rx) = channel();
    // the worker owns the paths through the Arc so the fallback below can run the very same job
    // inline when no thread could be started for it
    let payload = Arc::new((app.session.media.clone(), app.session.services.clone(), streamed));
    let worker_ctx = ctx.cloned();
    let run = {
        let payload = payload.clone();
        move || open_files(&payload, &tx, &worker_ctx)
    };
    if std::thread::Builder::new().name("filmcraft-import".into()).spawn(run.clone()).is_err() {
        // resource exhaustion, not a media problem: open them here and now
        run();
    }
    app.ui.status = tlf!("Importing {n} media…", n = count);
    // what imported here and now is the caller's answer (the file dialog's); the rest lands as
    // the workers open it
    let mut out = json!({"items": sync_ids.iter().map(|i| i.0).collect::<Vec<_>>(), "importing": count});
    if !errors.is_empty() {
        out["errors"] = json!(errors);
    }
    app.imports.push(Batch {
        rx,
        ids: Vec::new(),
        sync_ids,
        remaining: count,
        files: count,
        bin: Some(bin),
        group: drop.map(|g| g.id),
        errors,
        bins: HashMap::new(),
        key,
    });
    Ok(out)
}

/// Import a folder dropped from the OS into `parent`: the walk and the file opens run on the
/// worker, because a deep tree or a folder of long recordings must not stall the UI. The bins the
/// tree becomes — named after the folder, sub-folders inside it, as the Media Browser's folder
/// import makes them — and the files not worth a worker are made when the walk lands
/// ([`land_walked`]); each opened file imports into its folder's bin as it arrives. `drop` holds
/// the batch's items for the drop's placement like [`start`] does. On the web (no threads) the
/// engine's own folder import runs instead, synchronously.
pub(crate) fn start_folder(app: &mut FilmcraftApp, ctx: Option<&egui::Context>, path: String, parent: BinId, drop: Option<DropGroup>) -> Result<Value, String> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = ctx;
        owe_drop(app, &drop);
        let r = app.session.execute("mediaBrowser.import", json!({"paths": [path], "bin": parent.0})).map_err(|e| e.to_string());
        let ids = r.as_ref().ok().map(items_of).unwrap_or_default();
        land_batch(app, &drop, ids);
        return r;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        owe_drop(app, &drop);
        let name = filmcraft_engine::media_browser::base_name(&path);
        let key = batch_key();
        let (tx, rx) = channel();
        let payload = Arc::new((app.session.media.clone(), app.session.services.clone(), path));
        let worker_ctx = ctx.cloned();
        let run = {
            let payload = payload.clone();
            move || walk_and_open(&payload, &tx, &worker_ctx)
        };
        if std::thread::Builder::new().name("filmcraft-import".into()).spawn(run.clone()).is_err() {
            // resource exhaustion, not a media problem: walk and open here and now
            run();
        }
        app.ui.status = tlf!("Importing {name}…", name);
        app.imports.push(Batch {
            rx,
            ids: Vec::new(),
            sync_ids: Vec::new(),
            remaining: 1,
            files: 0,
            bin: Some(parent),
            group: drop.map(|g| g.id),
            errors: Vec::new(),
            bins: HashMap::new(),
            key,
        });
        Ok(json!({"importing": 1}))
    }
}

/// Import synchronously through `file.import`, as one undoable "Import"; returns its result, item
/// ids and reported errors.
fn sync_import(app: &mut FilmcraftApp, paths: Vec<String>, bin: BinId) -> (Result<Value, String>, Vec<ItemId>, Vec<String>) {
    let r = app.session.edit_group("Import", |s| s.execute("file.import", json!({"paths": paths, "bin": bin.0}))).map_err(|e| e.to_string());
    let ids = r.as_ref().ok().map(items_of).unwrap_or_default();
    let mut errors: Vec<String> =
        r.as_ref().ok().and_then(|v| v["errors"].as_array()).map(|errs| errs.iter().filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default();
    if let Err(e) = &r {
        errors.push(e.clone());
    }
    (r, ids, errors)
}

/// `file.import` of `paths` into `bin`, with image sequences off as the engine's folder import
/// runs it (each numbered still is its own item); returns the items and the reported errors.
fn import_files(s: &mut Session, paths: &[String], bin: Option<BinId>) -> (Vec<ItemId>, Vec<String>) {
    let mut q = json!({"paths": paths, "imageSequence": false});
    if let Some(b) = bin {
        q["bin"] = json!(b.0);
    }
    match s.execute("file.import", q) {
        Ok(v) => {
            let errors = v["errors"].as_array().map(|errs| errs.iter().filter_map(Value::as_str).map(str::to_owned).collect()).unwrap_or_default();
            (items_of(&v), errors)
        }
        Err(e) => (Vec::new(), vec![e.to_string()]),
    }
}

/// The items an import result created.
fn items_of(v: &Value) -> Vec<ItemId> {
    v["items"].as_array().map(|a| a.iter().filter_map(|i| i.as_u64()).map(ItemId).collect()).unwrap_or_default()
}

/// Queue a file drop's imported clips for the timeline to place this frame: a finished drop
/// hands its held items over here (see [`land_drop`]), and two drops finishing the same frame at
/// the same point place together.
fn place_drop(app: &mut FilmcraftApp, ids: Vec<ItemId>, drop_at: Option<Pos2>) {
    if ids.is_empty() {
        return;
    }
    let Some(pos) = drop_at else { return };
    match &mut app.pending_timeline_drop {
        Some((placed, at)) if *at == pos => placed.extend(ids),
        placed => *placed = Some((ids, pos)),
    }
}

/// Whether `path`'s import is the slow streaming open a background worker should do: media the
/// host's reader can open (video and audio containers, stills) that is not an image sequence —
/// those import whole, on their own synchronous path — and large enough that opening it is worth
/// a thread hop.
fn backgroundable(services: &dyn Services, path: &str) -> bool {
    if !filmcraft_media::is_importable(std::path::Path::new(path)) {
        return false;
    }
    let sequence = filmcraft_media::sequence::Numbered::parse(path).is_some()
        && filmcraft_engine::media_pool::image_sequence_frames(path, services).map(|f| f.len() > 1).unwrap_or(false);
    !sequence && services.reader(path).is_some() && services.file_size(path).map(|n| n >= BACKGROUND_MIN_BYTES).unwrap_or(false)
}

/// Ask for a frame now that a worker has something to hand over. `None` (the caller that started
/// the batch holds no repaint context): the frame loop polls the batches meanwhile.
fn repaint(ctx: &Option<egui::Context>) {
    if let Some(ctx) = ctx {
        ctx.request_repaint();
    }
}

/// Open one file for importing, catching a panic (a dead worker leaves the rest of the batch
/// running, rule §0.6).
fn open_one(pool: &MediaPool, services: &dyn Services, path: &str) -> Msg {
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let identity = filmcraft_engine::relink::identity_of(services, path).map_err(|e| e.to_string())?;
        let src = pool.open_file(path, services).map_err(|e| e.to_string())?;
        Ok::<_, String>((src, identity))
    }));
    match r {
        Ok(Ok((src, identity))) => Msg::Opened { path: path.to_string(), src, identity },
        Ok(Err(e)) => Msg::Failed { path: path.to_string(), error: e },
        Err(_) => Msg::Failed { path: path.to_string(), error: "the import panicked".into() },
    }
}

/// Open every file of the batch on the calling thread, sending each result as it is ready.
fn open_files(payload: &Arc<(Arc<MediaPool>, Arc<dyn Services>, Vec<String>)>, tx: &Sender<Msg>, ctx: &Option<egui::Context>) {
    let (pool, services, files) = (&payload.0, &payload.1, &payload.2);
    for path in files {
        // one unreadable file must not stop the ones behind it, nor a panic take the thread
        if tx.send(open_one(pool, &**services, path)).is_err() {
            break;
        }
        repaint(ctx);
    }
}

/// Walk a dropped folder like the engine's folder import does (`media_browser::walk_folder`):
/// the files the browser shows, sub-folders that hold any, within the same budget and depth.
/// Returns the index of the folder in `out`, parents before children; a folder that holds nothing
/// is left out. Each file goes where an individual drop of it would: the large streaming opens
/// to the worker, the rest to the synchronous path.
fn walk_folder(services: &dyn Services, dir: &str, depth: usize, budget: &mut usize, parent: Option<usize>, out: &mut Vec<WalkedFolder>) -> Option<usize> {
    *budget = budget.checked_sub(1)?;
    let entries = filmcraft_engine::media_browser::list(services, dir, "all").ok()?;
    let at = out.len();
    out.push(WalkedFolder { name: filmcraft_engine::media_browser::base_name(dir), parent, sync: Vec::new(), opening: Vec::new() });
    let mut files = Vec::new();
    let mut any = false;
    for e in entries {
        if e.is_dir {
            if depth < FOLDER_IMPORT_MAX_DEPTH && walk_folder(services, &e.path, depth + 1, budget, Some(at), out).is_some() {
                any = true;
            }
        } else {
            files.push(e.path);
            any = true;
        }
    }
    if !any {
        out.truncate(at);
        return None;
    }
    let (sync, opening) = files.into_iter().partition(|p| !backgroundable(services, p));
    out[at].sync = sync;
    out[at].opening = opening;
    Some(at)
}

/// Walk a dropped folder and open its background-worthy files: the tree is sent first (the bins
/// to make, the synchronous files, how many opens follow), then each opened source as it lands.
fn walk_and_open(payload: &Arc<(Arc<MediaPool>, Arc<dyn Services>, String)>, tx: &Sender<Msg>, ctx: &Option<egui::Context>) {
    let (pool, services, root) = (&payload.0, &payload.1, &payload.2);
    // a panic in the walk must not leave the batch waiting forever (rule §0.6)
    let walked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut folders = Vec::new();
        let mut budget = FOLDER_IMPORT_MAX_FOLDERS;
        walk_folder(&**services, root, 0, &mut budget, None, &mut folders);
        folders
    }));
    let folders = match walked {
        Ok(folders) => folders,
        Err(_) => {
            let _ = tx.send(Msg::Failed { path: root.clone(), error: "the import panicked".into() });
            repaint(ctx);
            return;
        }
    };
    let opening = opening_of(&folders);
    if tx.send(Msg::Folders { root: root.clone(), folders }).is_err() {
        return;
    }
    repaint(ctx);
    for path in &opening {
        if tx.send(open_one(pool, &**services, path)).is_err() {
            break;
        }
        repaint(ctx);
    }
}

/// The files a walk leaves to the worker, in walk order.
fn opening_of(folders: &[WalkedFolder]) -> Vec<String> {
    folders.iter().flat_map(|f| f.opening.iter().cloned()).collect()
}

/// Apply what the background import workers have finished: import each opened source (a cheap
/// edit), make a dropped folder's bins when its walk lands, and when a batch is complete run the
/// import's after-work and place a file drop's clips on the timeline. Worker messages request
/// repaints as they arrive.
pub(crate) fn pump(app: &mut FilmcraftApp) {
    let mut msgs: Vec<(usize, Msg)> = Vec::new();
    for (i, b) in app.imports.iter_mut().enumerate() {
        while let Ok(m) = b.rx.try_recv() {
            // a folder's walk announces the opens it started before they are counted off
            if let Msg::Folders { folders, .. } = &m {
                let opening = opening_of(folders).len();
                b.remaining += opening;
                b.files += opening;
            }
            b.remaining = b.remaining.saturating_sub(1);
            msgs.push((i, m));
        }
    }
    for (i, msg) in msgs {
        match msg {
            Msg::Folders { root, folders } => land_walked(app, i, root, folders),
            Msg::Opened { path, src, identity } => {
                let b = &mut app.imports[i];
                let bin = b.bins.get(&path).copied().or(b.bin);
                let key = b.key.clone();
                b.files = b.files.saturating_sub(1);
                match app.session.import_opened_source(&path, src, identity, bin, &key) {
                    Ok(id) => b.ids.push(id),
                    Err(e) => b.errors.push(format!("{path}: {e}")),
                }
            }
            Msg::Failed { path, error } => {
                let b = &mut app.imports[i];
                b.errors.push(format!("{path}: {error}"));
                b.files = b.files.saturating_sub(1);
            }
        }
    }
    // progress in the status line; every open that lands restarts the message's timer
    let in_flight: usize = app.imports.iter().map(|b| b.files).sum();
    if in_flight > 0 {
        let status = tlf!("Importing {n} media…", n = in_flight);
        if app.ui.status != status {
            app.ui.status = status;
        }
    }
    let ready: Vec<usize> = app.imports.iter().enumerate().filter(|(_, b)| b.remaining == 0).map(|(i, _)| i).collect();
    for i in ready.into_iter().rev() {
        let b = app.imports.remove(i);
        let landed = b.ids.len() + b.sync_ids.len();
        let had_landed = !b.ids.is_empty();
        after_import(app, &b.ids);
        let mut ids = b.sync_ids;
        ids.extend(b.ids);
        // the drop's batches place together once every one of them has landed
        if let Some(g) = b.group {
            land_drop(app, g, ids);
        }
        if !b.errors.is_empty() {
            app.ui.status = b.errors.join("; ");
        } else if had_landed {
            // the background import promised "Importing…"; say it landed
            app.ui.status = tlf!("Imported {n} media", n = landed);
        }
    }
}

/// A dropped folder's walk landed: make its bins and import its synchronous files — one undoable
/// "Import" the files still opening merge into — and remember each opening file's bin. A bin that
/// cannot be added does not stop the rest: its files import into the drop target instead.
fn land_walked(app: &mut FilmcraftApp, i: usize, root: String, folders: Vec<WalkedFolder>) {
    if folders.is_empty() {
        // the engine's folder import says the same of a folder with nothing in it
        app.imports[i].errors.push(format!("{root}: no media files"));
        return;
    }
    let target = app.imports[i].bin;
    let key = app.imports[i].key.clone();
    let group = app.session.edit_group("Import", |s| {
        let mut bins: Vec<Option<BinId>> = Vec::with_capacity(folders.len());
        let mut ids = Vec::new();
        let mut errors: Vec<String> = Vec::new();
        for f in &folders {
            // parents come before children in the walk, so a parent's bin exists by now
            let parent = f.parent.and_then(|p| bins.get(p).copied().flatten()).or(target);
            match s.edit("New Bin", |pr, _| Ok(pr.add_bin(&f.name, parent))) {
                Ok(bin) => bins.push(Some(bin)),
                Err(e) => {
                    errors.push(format!("{}: {e}", f.name));
                    bins.push(None);
                }
            }
        }
        for (f, bin) in folders.iter().zip(bins.iter().copied()) {
            if f.sync.is_empty() {
                continue;
            }
            let (made, errs) = import_files(s, &f.sync, bin);
            ids.extend(made);
            errors.extend(errs);
        }
        Ok((bins, ids, errors))
    });
    // the files the worker is still opening merge into the same undo step
    app.session.history.merge_key = Some(key);
    let b = &mut app.imports[i];
    match group {
        Ok((bins, ids, errors)) => {
            b.sync_ids.extend(ids);
            b.errors.extend(errors);
            for (f, bin) in folders.iter().zip(bins) {
                if let Some(bin) = bin {
                    for path in &f.opening {
                        b.bins.insert(path.clone(), bin);
                    }
                }
            }
        }
        Err(e) => b.errors.push(e.to_string()),
    }
}

/// The parts of `file.import` that follow the imports themselves, run for one finished batch:
/// Settings ▸ Media Analysis' automatic transcription, Project Settings ▸ Ingest, and relinking
/// (a newly imported file may resolve paths the project listed as offline, #110).
fn after_import(app: &mut FilmcraftApp, ids: &[ItemId]) {
    let (auto, scope) = {
        let ma = &app.session.prefs.media_analysis;
        (ma.auto_transcribe, ma.auto_transcribe_scope.clone())
    };

    if auto && scope == "allImported" && !ids.is_empty() {
        let audio: Vec<u64> = ids.iter().filter(|i| app.session.project.item(**i).is_some_and(|it| it.has_audio())).map(|i| i.0).collect();

        if !audio.is_empty()
            && let Err(e) = app.session.execute("transcript.generate", serde_json::json!({"items": audio}))
        {
            app.ui.status = format!("transcription: {e}");
        }
    }

    if let Err(e) = filmcraft_engine::proxies::ingest(&mut app.session, ids) {
        app.ui.status = format!("ingest: {e}");
    }

    let was_missing = app.session.offline.missing.clone();

    filmcraft_engine::relink::refresh(&mut app.session);

    for item in was_missing.iter().filter(|i| !app.session.offline.missing.contains(i)) {
        app.session.media.remove(*item);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use filmcraft_engine::media_browser::DirEntry;
    use filmcraft_media::generators::GeneratorSource;

    /// A fake filesystem: a directory's entries by path, and file sizes. `readers` decides
    /// whether the host can open media without reading it whole (what makes a file
    /// background-worthy besides its size).
    struct FakeFs {
        dirs: HashMap<String, Vec<(String, bool)>>,
        sizes: HashMap<String, u64>,
        readers: bool,
    }

    impl FakeFs {
        fn dir(mut self, path: &str, entries: &[(&str, bool)]) -> Self {
            self.dirs.insert(path.to_string(), entries.iter().map(|(n, d)| (n.to_string(), *d)).collect());
            self
        }
        fn file(mut self, path: &str, bytes: u64) -> Self {
            self.sizes.insert(path.to_string(), bytes);
            self
        }
        fn with_readers(mut self) -> Self {
            self.readers = true;
            self
        }
    }

    impl Default for FakeFs {
        fn default() -> Self {
            Self { dirs: HashMap::new(), sizes: HashMap::new(), readers: false }
        }
    }

    /// Bytes for the fake host's random-access readers; never read by these tests.
    struct NoBytes;
    impl filmcraft_media::ByteReader for NoBytes {
        fn len(&self) -> u64 {
            0
        }
        fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> std::io::Result<()> {
            Err(std::io::Error::other("the test reader has no bytes"))
        }
    }

    impl Services for FakeFs {
        fn read_file(&self, _path: &str) -> std::io::Result<Vec<u8>> {
            Err(std::io::Error::other("the fake filesystem has no bytes"))
        }
        fn write_file(&self, _path: &str, _data: &[u8]) -> std::io::Result<()> {
            Ok(())
        }
        fn file_size(&self, path: &str) -> std::io::Result<u64> {
            self.sizes.get(path).copied().ok_or_else(|| std::io::Error::other("no such file"))
        }
        fn reader(&self, _path: &str) -> Option<std::io::Result<filmcraft_media::SharedReader>> {
            self.readers.then(|| {
                let reader: filmcraft_media::SharedReader = Arc::new(NoBytes);
                Ok(reader)
            })
        }
        fn list_entries(&self, dir: &str) -> Option<std::io::Result<Vec<DirEntry>>> {
            Some(Ok(self
                .dirs
                .get(dir)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .map(|(name, is_dir)| DirEntry { name, is_dir, size: None, modified: None })
                .collect()))
        }
    }

    /// The walk mirrors the engine's folder import: a bin per folder that holds anything,
    /// sub-folders inside it, files split the way an individual drop splits them, and empty
    /// folders, hidden files and files the browser does not show left out.
    #[test]
    fn walk_mirrors_the_folder_import() {
        let fs = FakeFs::default()
            .dir("/m/My Folder", &[("big.mp4", false), ("notes.txt", false), ("small.wav", false), ("keep", true), ("empty", true), (".hidden", false)])
            .dir("/m/My Folder/keep", &[("c.mp4", false), ("deep", true)])
            .dir("/m/My Folder/keep/deep", &[("d.wav", false)])
            .dir("/m/My Folder/empty", &[])
            .file("/m/My Folder/big.mp4", 64 << 20)
            .file("/m/My Folder/small.wav", 1024)
            .file("/m/My Folder/keep/c.mp4", 1024)
            .file("/m/My Folder/keep/deep/d.wav", 64 << 20)
            .with_readers();
        let mut out = Vec::new();
        let mut budget = FOLDER_IMPORT_MAX_FOLDERS;
        let root = walk_folder(&fs, "/m/My Folder", 0, &mut budget, None, &mut out);
        assert_eq!(root, Some(0));
        let shape: Vec<(&str, Option<usize>)> = out.iter().map(|f| (f.name.as_str(), f.parent)).collect();
        assert_eq!(shape, [("My Folder", None), ("keep", Some(0)), ("deep", Some(1))]);
        assert_eq!(out[0].opening, ["/m/My Folder/big.mp4"]);
        assert_eq!(out[0].sync, ["/m/My Folder/small.wav"]);
        assert_eq!(out[1].opening, Vec::<String>::new());
        assert_eq!(out[1].sync, ["/m/My Folder/keep/c.mp4"]);
        assert_eq!(out[2].opening, ["/m/My Folder/keep/deep/d.wav"]);
    }

    /// The walk stops at the same budget as the engine's: folders past it are left out, and a
    /// folder whose whole tree was pruned is left out itself.
    #[test]
    fn walk_stops_at_the_folder_budget() {
        let fs = FakeFs::default()
            .dir("/m/root", &[("a.mp4", false), ("sub", true)])
            .dir("/m/root/sub", &[("b.mp4", false)])
            .file("/m/root/a.mp4", 8)
            .file("/m/root/sub/b.mp4", 8);
        let mut out = Vec::new();
        let mut budget = 1;
        assert_eq!(walk_folder(&fs, "/m/root", 0, &mut budget, None, &mut out), Some(0));
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].sync, ["/m/root/a.mp4"]);
    }

    /// Without a host reader nothing is worth a worker: every file of the walk stays synchronous.
    #[test]
    fn walk_without_readers_stays_synchronous() {
        let fs = FakeFs::default().dir("/m/root", &[("big.mp4", false)]).file("/m/root/big.mp4", 64 << 20);
        let mut out = Vec::new();
        let mut budget = FOLDER_IMPORT_MAX_FOLDERS;
        walk_folder(&fs, "/m/root", 0, &mut budget, None, &mut out);
        assert_eq!(out.len(), 1);
        assert!(out[0].opening.is_empty());
        assert_eq!(out[0].sync, ["/m/root/big.mp4"]);
    }

    /// Two drops that hand their clips to the timeline the same frame at the same point place
    /// together; a drop at another point replaces what was placed.
    #[test]
    fn drops_finishing_at_the_same_point_place_together() {
        let mut app = FilmcraftApp::new(Session::default());
        let at = egui::pos2(10.0, 20.0);
        place_drop(&mut app, vec![ItemId(1)], Some(at));
        place_drop(&mut app, vec![ItemId(2), ItemId(3)], Some(at));
        assert_eq!(app.pending_timeline_drop, Some((vec![ItemId(1), ItemId(2), ItemId(3)], at)));
        place_drop(&mut app, vec![ItemId(4)], Some(egui::pos2(1.0, 2.0)));
        assert_eq!(app.pending_timeline_drop, Some((vec![ItemId(4)], egui::pos2(1.0, 2.0))));
        // nothing to place, or no point to place it at: the queue is untouched
        place_drop(&mut app, Vec::new(), Some(egui::pos2(9.0, 9.0)));
        place_drop(&mut app, vec![ItemId(5)], None);
        assert_eq!(app.pending_timeline_drop, Some((vec![ItemId(4)], egui::pos2(1.0, 2.0))));
    }

    /// The batches of one drop — its files, each dropped folder — hold what they import until
    /// every one of them has landed, then place together at the point of the drop: placing each
    /// batch as it lands would lay the later ones over the earlier (the Timeline places from the
    /// drop point, end to end).
    #[test]
    fn one_drop_places_when_its_last_batch_lands() {
        let mut app = FilmcraftApp::new(Session::default());
        let at = egui::pos2(10.0, 20.0);
        let drop = begin_drop(&mut app, Some(at)).unwrap();
        // a batch of the drop that lands at once (the synchronous path)
        owe_drop(&mut app, &Some(drop));
        land_drop(&mut app, drop.id, vec![ItemId(1)]);
        // the drop's registration is still owed: its folder batches are starting
        assert_eq!(app.pending_timeline_drop, None);
        end_drop(&mut app, Some(drop));
        assert_eq!(app.pending_timeline_drop, Some((vec![ItemId(1)], at)));
        // a late landing after the drop placed has nothing left to place
        land_drop(&mut app, drop.id, vec![ItemId(9)]);
        assert_eq!(app.pending_timeline_drop, Some((vec![ItemId(1)], at)));
    }

    /// Batches of one drop that land out of order (a small folder's walk against a long recording
    /// still opening) still place together, in landing order, once the last of them lands.
    #[test]
    fn batches_of_one_drop_place_together_in_landing_order() {
        let mut app = FilmcraftApp::new(Session::default());
        let at = egui::pos2(30.0, 40.0);
        let drop = begin_drop(&mut app, Some(at)).unwrap();
        owe_drop(&mut app, &Some(drop));
        owe_drop(&mut app, &Some(drop));
        end_drop(&mut app, Some(drop));
        land_drop(&mut app, drop.id, vec![ItemId(2)]);
        assert_eq!(app.pending_timeline_drop, None);
        land_drop(&mut app, drop.id, vec![ItemId(1), ItemId(3)]);
        assert_eq!(app.pending_timeline_drop, Some((vec![ItemId(2), ItemId(1), ItemId(3)], at)));
    }

    /// A drop that does not place on the timeline (the preference off, or a drop onto the Project
    /// panel) imports into the bin only: no group, nothing queued, nothing held.
    #[test]
    fn a_drop_without_a_point_imports_without_placing() {
        let mut app = FilmcraftApp::new(Session::default());
        assert!(begin_drop(&mut app, None).is_none());
        // batches started without a group never place
        land_batch(&mut app, &None, vec![ItemId(1)]);
        assert_eq!(app.pending_timeline_drop, None);
        assert!(app.import_drops.is_empty());
    }

    /// The engine bridge behind a landed batch: imports that share a batch's key are one undo
    /// step, one Undo takes the whole batch back, and another batch is a step of its own.
    #[test]
    fn landed_imports_share_one_undo_step() {
        let mut app = FilmcraftApp::new(Session::default());
        let src: SharedSource = Arc::new(GeneratorSource::demo(filmcraft_media::DemoScene::Plasma));
        let identity = MediaIdentity { size: 1, fingerprint: 1 };
        let key = batch_key();
        let a = app.session.import_opened_source("/m/a.mp4", src.clone(), identity, None, &key).unwrap();
        let b = app.session.import_opened_source("/m/b.mp4", src.clone(), identity, None, &key).unwrap();
        assert_ne!(a, b);
        assert_eq!(app.session.history.undo.len(), 1);
        app.session.undo();
        assert!(app.session.project.item(a).is_none());
        assert!(app.session.project.item(b).is_none());
        // a second batch never merges into the first, even with nothing edited in between
        let other = batch_key();
        assert_ne!(key, other);
        let _ = app.session.import_opened_source("/m/a.mp4", src.clone(), identity, None, &other).unwrap();
        let _ = app.session.import_opened_source("/m/b.mp4", src, identity, None, &other).unwrap();
        assert_eq!(app.session.history.undo.len(), 2);
    }

    /// Two batch keys are never the same, whatever lands in between.
    #[test]
    fn batch_keys_are_distinct() {
        let a = batch_key();
        let b = batch_key();
        assert_ne!(a, b);
    }

    /// The dance `start` and `land_walked` run: the synchronous part of a batch is one grouped
    /// "Import" step, the batch's merge key is armed onto it, and the files the workers open
    /// merge into that same step — one Undo takes the whole batch back, however it was split
    /// between the two paths.
    #[test]
    fn landed_files_merge_into_the_synchronous_imports_step() {
        let mut app = FilmcraftApp::new(Session::default());
        let src: SharedSource = Arc::new(GeneratorSource::demo(filmcraft_media::DemoScene::Plasma));
        let identity = MediaIdentity { size: 1, fingerprint: 1 };
        let key = batch_key();
        // the synchronous part: one undoable Import (as `sync_import`/`land_walked` group it)
        let bin = app.session.edit_group("Import", |s| s.edit("New Bin", |pr, _| Ok(pr.add_bin("Dropped", None)))).unwrap();
        // the batch arms its key onto that step (as `start`/`land_walked` do)
        app.session.history.merge_key = Some(key.clone());
        let a = app.session.import_opened_source("/m/a.mp4", src.clone(), identity, Some(bin), &key).unwrap();
        let b = app.session.import_opened_source("/m/b.mp4", src, identity, Some(bin), &key).unwrap();
        assert_ne!(a, b);
        assert_eq!(app.session.history.undo.len(), 1);
        app.session.undo();
        assert!(app.session.project.item(a).is_none());
        assert!(app.session.project.item(b).is_none());
    }
}
