//! The text tool keeps the keyboard only while it truly owns it (#683): an edit session takes
//! the keyboard when it begins, a click on another field (the Effects panel's search) gives that
//! field the keyboard, a keyboard that nobody owns is not re-grabbed (the stuck tool: typing
//! went into the video text layer instead of where the user had gone), and a click back on the
//! text re-takes it. Driven headless through the control channel like `graphics_ui.rs`.

use std::sync::mpsc::{Sender, channel};

use egui_kittest::Harness;
use filmcraft_engine::Session;
use filmcraft_ui_egui::FilmcraftApp;
use filmcraft_ui_egui::control::ControlRequest;
use serde_json::{Value, json};

/// `crate::panels::graphics::TEXT_EDIT_ID`: the monitor's text layer, a private constant there.
const TEXT_EDIT_ID: &str = "gfx-text-edit";

struct Driver {
    harness: Harness<'static, FilmcraftApp>,
    tx: Sender<ControlRequest>,
}

impl Driver {
    fn demo() -> Self {
        let mut session = Session::default();
        session.execute("file.openDemoProject", json!({})).expect("demo project");
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

    fn exec(&mut self, command: &str, params: Value) -> Value {
        self.ok("engine.execute", json!({"command": command, "params": params}))
    }

    fn rect(&mut self, id: &str) -> [f64; 4] {
        let v = self.ok("ui.elements", json!({"prefix": id}));
        let e = v.as_array().unwrap().iter().find(|e| e["id"] == id).unwrap_or_else(|| panic!("no element {id}: {v}")).clone();
        let r: Vec<f64> = e["rect"].as_array().unwrap().iter().map(|x| x.as_f64().unwrap()).collect();
        [r[0], r[1], r[2], r[3]]
    }

    fn focused(&self) -> Option<egui::Id> {
        self.harness.ctx.memory(|m| m.focused())
    }

    /// The text of the sequence's first graphic layer, with its clip id.
    fn text(&mut self) -> (u64, String) {
        let list = self.exec("graphics.list", json!({}));
        let clip = list["clip"].as_u64().unwrap_or_else(|| panic!("no graphic clip: {list}"));
        let text = list["layers"][0]["text"].as_str().unwrap_or_default().to_string();
        (clip, text)
    }
}

#[test]
fn the_text_layer_takes_the_keyboard_only_when_activated() {
    let mut d = Driver::demo();
    // the Effects workspace shows the Program monitor (for the text) and the Effects panel with
    // its search field; the panel draws after the monitor, so a click on the field wins the
    // keyboard in the same frame
    d.ok("ui.set", json!({"workspace": "Effects"}));
    d.exec("playhead.set", json!({"seconds": 2}));
    d.ok("ui.set", json!({"tool": "Type"}));
    d.frames(3);

    // a click with the Type tool starts the edit session, which takes the keyboard
    let pic = d.rect("program.picture");
    d.ok("ui.click", json!({"x": pic[0] + pic[2] * 0.2, "y": pic[1] + pic[3] * 0.35}));
    d.frames(3);
    let text_id = egui::Id::new(TEXT_EDIT_ID);
    assert!(d.harness.state().ui.gfx_edit.is_some(), "the edit session began");
    assert_eq!(d.focused(), Some(text_id), "the session that just began takes the keyboard");
    d.ok("ui.type", json!({"text": "abc"}));
    let (clip, text) = d.text();
    assert_eq!(text, "abc");

    // a click on the Program panel's own tab (not on any text field): egui frees the keyboard,
    // and the panel focus stays on Program, so the session lives on. The stuck tool re-grabbed
    // the keyboard here, and the next keys went into the video text layer.
    d.ok("ui.click", json!({"id": "panel.tab.Program"}));
    d.frames(3);
    assert_eq!(d.focused(), None, "a click on a non-field frees the keyboard");
    assert!(d.harness.state().ui.gfx_edit.is_some(), "the session lives on (the panel focus did not move)");
    d.ok("ui.type", json!({"text": "xyz"}));
    let (_, text) = d.text();
    assert_eq!(text, "abc", "typing with nobody focused goes nowhere, not into the layer");
    assert_eq!(d.harness.state().ui.effects_search, "", "and not into another field either");

    // a click back on the text re-takes the keyboard, and typing continues in the layer
    d.ok("ui.click", json!({"id": format!("program.layer.{clip}.0")}));
    d.frames(2);
    assert_eq!(d.focused(), Some(text_id), "a click on the text re-takes the keyboard");
    d.ok("ui.type", json!({"text": "def"}));
    let (_, text) = d.text();
    assert_eq!(text, "abcdef");

    // a click on the Effects search field gives that field the keyboard; moving the panel
    // focus ends the session a frame later
    let panel = d.rect("panel.effects");
    d.ok("ui.click", json!({"x": panel[0] + 60.0, "y": panel[1] + 17.0}));
    d.frames(3);
    assert_ne!(d.focused(), Some(text_id), "the search field has the keyboard");
    assert!(d.harness.state().ui.gfx_edit.is_none(), "the session ended with the panel focus");
    d.ok("ui.type", json!({"text": "efg"}));
    assert_eq!(d.harness.state().ui.effects_search, "efg", "typing goes to the search field");
    let (_, text) = d.text();
    assert_eq!(text, "abcdef", "not into the video text layer");

    // Escape leaves no phantom focus on the text layer
    d.ok("ui.key", json!({"key": "Escape"}));
    d.frames(2);
    assert_ne!(d.focused(), Some(text_id), "no phantom focus on the text layer");
}
