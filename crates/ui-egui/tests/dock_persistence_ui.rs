//! The panel arrangement auto-saves per workspace and reopens at launch and when switching back
//! (#493, [gaps.md G8](../../docs/gaps.md)): floating a panel, waiting for the debounce, then
//! restarting or switching away and back restores the arrangement as it was left, while Reset to
//! Saved Layout drops back to the workspace's saved (here: built-in) layout. Driven headless
//! through the control channel like `workspaces_ui.rs`.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{Sender, channel};

use egui_kittest::Harness;
use filmcraft_engine::Session;
use filmcraft_ui_egui::FilmcraftApp;
use filmcraft_ui_egui::control::ControlRequest;
use filmcraft_ui_egui::dock::{self, PanelKind};
use serde_json::{Value, json};

struct Driver {
    harness: Harness<'static, FilmcraftApp>,
    tx: Sender<ControlRequest>,
}

impl Driver {
    /// The demo project with its preferences (and workspaces) in `data`.
    fn launch(data: &Path) -> Self {
        let mut session = Session::default();
        session.execute("file.openDemoProject", json!({})).expect("demo project");
        session.prefs_path = Some(data.join("preferences.json"));
        let (tx, rx) = channel();
        let app = FilmcraftApp::new(session).with_control(rx);
        let harness = Harness::builder().with_size(egui::vec2(1600.0, 980.0)).with_max_steps(10_000).build_eframe(move |_cc| app);
        let mut d = Driver { harness, tx };
        d.frames(4);
        d
    }

    fn frames(&mut self, n: usize) {
        for _ in 0..n {
            let ctx = self.harness.ctx.clone();
            let mut raw = std::mem::take(self.harness.input_mut());
            eframe::App::raw_input_hook(self.harness.state_mut(), &ctx, &mut raw);
            *self.harness.input_mut() = raw;
            self.harness.step();
        }
    }

    fn call(&mut self, method: &str, params: Value) -> Value {
        let (req, reply) = ControlRequest::new(method, params.clone());
        self.tx.send(req).expect("control channel open");
        for _ in 0..600 {
            self.frames(1);
            if let Ok(v) = reply.try_recv() {
                return v;
            }
        }
        panic!("no reply to {method} {params}");
    }

    fn ok(&mut self, method: &str, params: Value) -> Value {
        let v = self.call(method, params.clone());
        assert_eq!(v["ok"], json!(true), "{method} {params} failed: {v}");
        v["result"].clone()
    }

    fn menu(&mut self, id: &str, params: Value) -> Value {
        self.ok("ui.menu.invoke", json!({"id": id, "params": params}))
    }

    fn app(&mut self) -> &mut FilmcraftApp {
        self.harness.state_mut()
    }
}

fn data_dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("filmcraft-dock-persistence-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).expect("temp dir");
    d
}

/// The workspaces file's auto-saved arrangements (`open`), by workspace name.
fn open_names(data: &Path) -> Vec<String> {
    let file: Value =
        serde_json::from_slice(&std::fs::read(data.join(dock::WORKSPACES_FILE)).expect("workspaces.json written")).expect("workspaces.json parses");
    file["open"].as_array().expect("auto-saved arrangements (`open`)").iter().map(|e| e["name"].as_str().unwrap_or_default().to_string()).collect()
}

#[test]
fn the_arrangement_as_left_reopens_at_launch_and_on_switching_back() {
    let data = data_dir("open-layout");
    let mut d = Driver::launch(&data);
    assert_eq!(d.app().ui.workspace, "Editing");
    assert!(d.app().ui.floating.is_empty(), "the built-in Editing layout has no floating panels: {:?}", d.app().ui.floating);

    // rearrange: float the Info panel — a change no saved layout holds
    d.menu("window.floatPanel", json!({"panel": "Info", "rect": [45, 65, 420, 320]}));
    assert!(d.app().ui.floating.iter().any(|f| f.panels.contains(&PanelKind::Info)), "Info floats");
    // the debounce settles (DOCK_SAVE_DELAY_FRAMES + margin): the arrangement is written under
    // the workspace in use
    d.frames(130);
    assert!(open_names(&data).contains(&"Editing".to_string()), "auto-saved under the workspace in use");
    let left = (d.app().ui.dock.clone(), d.app().ui.floating.clone());

    // switching away loads the other workspace's own arrangement; switching back reopens the
    // one as it was left, not the saved layout
    d.menu("window.workspace.color", json!({}));
    assert_eq!(d.app().ui.workspace, "Color");
    assert!(d.app().ui.floating.is_empty(), "the Color workspace has no floating panels");
    d.ok("ui.set", json!({"workspace": "Editing"}));
    assert_eq!((d.app().ui.dock.clone(), d.app().ui.floating.clone()), left, "switching back reopens the arrangement as it was left");

    // Reset to Saved Layout drops the auto-saved arrangement: the built-in layout, no floats
    d.menu("window.workspace.reset", json!({}));
    assert!(d.app().ui.floating.is_empty(), "the reset layout has no floating panels");
    assert_eq!(d.app().ui.dock, dock::workspace("Editing"));
    // the reset arrangement is auto-saved too, and a restart reopens it
    d.frames(130);
    drop(d);
    let mut d = Driver::launch(&data);
    assert_eq!(d.app().ui.workspace, "Editing");
    assert_eq!(d.app().ui.dock, dock::workspace("Editing"), "the arrangement as it was left (the reset one) reopens");
    assert!(d.app().ui.floating.is_empty());
    let _ = std::fs::remove_dir_all(&data);
}
