//! Marker editing and project palette creation are separate, undoable operations.
use crate::{
    FilmcraftApp,
    widgets::color_picker::{self, ColorDraft},
};
use egui::{Color32, Sense, Stroke, vec2};
use filmcraft_project::{HexColor, Label, Marker, MarkerKind};
use serde_json::json;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Draft {
    pub project: Arc<()>,
    pub owner: Owner,
    pub time: String,
    pub duration: String,
    pub range: bool,
    pub original_duration: filmcraft_time::Tick,
    pub initial_time: String,
    pub initial_duration: String,
    pub marker: u64,
    pub original: Marker,
    pub kind: MarkerKind,
    pub url: String,
    pub name: String,
    pub comment: String,
    pub author: String,
    pub hex: String,
    pub custom: bool,
    pub preset: Label,
    pub color_editor: Option<ColorDraft>,
    pub error: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Owner {
    Sequence(u64),
    Source(u64),
}
impl Owner {
    pub fn params(self) -> serde_json::Value {
        match self {
            Self::Sequence(id) => json!({"target":"program","sequence":id}),
            Self::Source(id) => json!({"target":"source","item":id}),
        }
    }
    pub fn current(app: &FilmcraftApp, source: bool) -> Option<Self> {
        if source { app.session.state.source_item.map(|id| Self::Source(id.0)) } else { app.session.state.active_sequence.map(|id| Self::Sequence(id.0)) }
    }
    pub fn details(self, app: &FilmcraftApp) -> Option<(String, filmcraft_time::FrameRate, bool, Vec<Marker>)> {
        let id = filmcraft_project::ItemId(match self {
            Self::Sequence(id) | Self::Source(id) => id,
        });
        let item = app.session.project.item(id)?;
        match self {
            Self::Sequence(_) => {
                let q = app.session.project.sequence(id)?;
                Some((tlf!("Sequence: {name}", name = &item.name), q.settings.frame_rate, q.settings.drop_frame, q.markers.clone()))
            }
            Self::Source(_) => {
                let v = filmcraft_engine::clip_ops::source_view(&app.session, id)?;
                Some((
                    if v.media != id { tlf!("Source: {name} (shared source markers)", name = &item.name) } else { tlf!("Source: {name}", name = &item.name) },
                    v.rate,
                    app.session.project.sequence(v.media).is_some_and(|q| q.settings.drop_frame),
                    v.markers,
                ))
            }
        }
    }
}
pub fn open(app: &mut FilmcraftApp, marker: &Marker) {
    if let Some(owner) = Owner::current(app, false) {
        open_owned(app, marker, owner);
    }
}
pub fn open_owned(app: &mut FilmcraftApp, marker: &Marker, owner: Owner) {
    let Some((_, rate, drop, _)) = owner.details(app) else { return };
    let format = |ticks| filmcraft_time::format_time(ticks, rate, drop, filmcraft_time::TimeDisplay::Timecode, 48000);
    let time = format(marker.start);
    let duration = format(marker.duration);
    app.ui.marker_properties = Some(Draft {
        project: app.session.project_instance.clone(),
        owner,
        initial_time: time.clone(),
        initial_duration: duration.clone(),
        time,
        duration,
        range: marker.duration > filmcraft_time::Tick::ZERO,
        original_duration: marker.duration,
        marker: marker.id.0,
        original: marker.clone(),
        kind: marker.kind,
        url: marker.url.clone(),
        name: marker.name.clone(),
        comment: marker.comment.clone(),
        author: marker.author.clone(),
        hex: HexColor(marker.custom_color.map_or_else(|| app.session.prefs.labels.rgb(marker.color), |c| c.0)).hex(),
        custom: marker.custom_color.is_some(),
        preset: marker.color,
        color_editor: None,
        error: String::new(),
    });
}

pub fn show(app: &mut FilmcraftApp, ctx: &egui::Context) {
    let Some(mut draft) = app.ui.marker_properties.take() else { return };
    if !Arc::ptr_eq(&draft.project, &app.session.project_instance) {
        return;
    }
    // Commands can edit or remove this marker while its dialog (or native picker) is open.
    // Check the marker itself, not unrelated project history, before allowing stale writes.
    let current = draft.owner.details(app).and_then(|(_, _, _, markers)| markers.into_iter().find(|marker| marker.id.0 == draft.marker));
    if current.as_ref() != Some(&draft.original) {
        app.ui.status = tl!("Marker changed or was removed. Reopen it to continue.").into();
        return;
    }
    if draft.color_editor.is_some() {
        color_dialog(app, ctx, &mut draft);
        app.ui.marker_properties = Some(draft);
        return;
    }
    let mut apply = false;
    let mut cancel = false;
    let mut delete = false;
    let modal = egui::Modal::new(egui::Id::new("marker-properties")).show(ctx, |ui| {
        ui.set_width(440.0);
        ui.heading(tl!("Edit Marker"));
        if let Some((title, _, _, _)) = draft.owner.details(app) {
            ui.label(title);
        }
        ui.horizontal(|ui| {
            ui.label(tl!("Time"));
            let response = ui.add(egui::TextEdit::singleline(&mut draft.time).desired_width(100.0));
            app.auto.add_response("markerProperties.time", &response, tl!("Time"));
            let response = ui.checkbox(&mut draft.range, tl!("Range"));
            app.auto.add_response("markerProperties.range", &response, tl!("Range marker"));
            if draft.range && draft.original_duration == filmcraft_time::Tick::ZERO && draft.duration == draft.initial_duration {
                draft.duration = "00:00:00:01".into();
            }
            ui.label(tl!("Duration"));
            let response = ui.add_enabled(draft.range, egui::TextEdit::singleline(&mut draft.duration).desired_width(100.0));
            app.auto.add_response("markerProperties.duration", &response, tl!("Duration"));
        });
        ui.add_space(6.0);
        ui.columns(2, |columns| {
            for ((id, label, value), column) in [("name", tl!("Name"), &mut draft.name), ("author", tl!("Author"), &mut draft.author)].into_iter().zip(columns)
            {
                column.label(label);
                let response = column.add(egui::TextEdit::singleline(value).desired_width(f32::INFINITY));
                app.auto.add_response(&format!("markerProperties.{id}"), &response, label);
            }
        });
        ui.label(tl!("Notes"));
        let response = ui.add(egui::TextEdit::multiline(&mut draft.comment).desired_width(f32::INFINITY).desired_rows(3));
        app.auto.add_response("markerProperties.comment", &response, tl!("Notes"));
        ui.horizontal(|ui| {
            ui.label(tl!("Type"));
            let menu = egui::ComboBox::from_id_salt("marker-properties-kind").selected_text(kind_label(draft.kind)).show_ui(ui, |ui| {
                for kind in [MarkerKind::Comment, MarkerKind::Chapter, MarkerKind::Segmentation, MarkerKind::WebLink, MarkerKind::FlashCue] {
                    let label = kind_label(kind);
                    let response = ui.selectable_value(&mut draft.kind, kind, label);
                    app.auto.add_response(&format!("markerProperties.kind.{}", kind_id(kind)), &response, label);
                }
            });
            app.auto.add_response("markerProperties.kind", &menu.response, tl!("Type"));
        });
        if draft.kind == MarkerKind::WebLink {
            ui.label(tl!("URL"));
            ui.horizontal(|ui| {
                let response = ui.add(egui::TextEdit::singleline(&mut draft.url).desired_width(320.0).char_limit(8192).hint_text("https://"));
                app.auto.add_response("markerProperties.url", &response, tl!("URL"));
                let address = filmcraft_project::marker_web_url(&draft.url);
                let response = ui.add_enabled(address.is_some(), egui::Button::new(tl!("Open Link")));
                let response = response.on_hover_text(tl!("Opens this address in your web browser"));
                app.auto.add_response("markerProperties.openLink", &response, tl!("Open Link"));
                if response.clicked()
                    && let Some(address) = address
                {
                    crate::links::open(app, ctx, &address);
                    if let Some(failure) = app.ui.failed_link.as_ref().filter(|failure| failure.url == address) {
                        draft.error.clone_from(&failure.error);
                    }
                }
            });
            if !draft.url.trim().is_empty() && filmcraft_project::marker_web_url(&draft.url).is_none() {
                ui.colored_label(ui.visuals().error_fg_color, tl!("Enter a valid HTTP or HTTPS URL without credentials"));
            }
        }
        ui.add_space(8.0);
        ui.label(tl!("Color"));
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 5.0;
            for preset in Label::ALL {
                let response =
                    swatch(ui, app.session.prefs.labels.rgb(preset), !draft.custom && draft.preset == preset).on_hover_text(crate::i18n::t(preset.name()));
                app.auto.add_response(&format!("markerProperties.preset.{}", preset.name()), &response, crate::i18n::t(preset.name()));
                if response.clicked() {
                    draft.preset = preset;
                    draft.custom = false;
                    draft.hex = HexColor(app.session.prefs.labels.rgb(preset)).hex();
                }
            }
        });
        let mut colors = super::custom_colors::available(app);
        if draft.custom
            && let Some(color) = HexColor::parse(&draft.hex)
            && !colors.contains(&color)
        {
            colors.push(color);
        }
        egui::ScrollArea::vertical().id_salt("marker-custom-colors").max_height(100.0).show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing.x = 5.0;
                for color in colors {
                    let hex = color.hex();
                    let response =
                        swatch(ui, color.0, draft.custom && draft.hex.eq_ignore_ascii_case(&hex)).on_hover_text(super::custom_colors::name(app, color));
                    app.auto.add_response(&format!("markerProperties.custom.{hex}"), &response, &hex);
                    if response.clicked() {
                        draft.hex = hex;
                        draft.custom = true;
                    }
                }
                let response = add_color_swatch(ui).on_hover_text(tl!("Add custom color"));
                app.auto.add_response("markerProperties.addColor", &response, tl!("Add custom color"));
                if response.clicked() {
                    draft.error.clear();
                    draft.color_editor = Some(ColorDraft::new(HexColor::parse(&draft.hex).map_or(app.session.prefs.labels.rgb(draft.preset), |c| c.0)));
                }
            });
        });
        if !draft.error.is_empty() {
            ui.colored_label(ui.visuals().error_fg_color, &draft.error);
        }
        ui.separator();
        ui.horizontal(|ui| {
            let response = ui.button(tl!("Done"));
            app.auto.add_response("markerProperties.apply", &response, tl!("Done"));
            apply = response.clicked();
            let response = ui.button(tl!("Cancel"));
            app.auto.add_response("markerProperties.cancel", &response, tl!("Cancel"));
            cancel = response.clicked();
            let response = ui.button(tl!("Delete Marker"));
            app.auto.add_response("markerProperties.delete", &response, tl!("Delete Marker"));
            delete = response.clicked();
        });
    });
    if cancel || modal.should_close() {
        return;
    }
    if delete {
        let mut p = draft.owner.params();
        p["marker"] = json!(draft.marker);
        match app.session.execute("markers.delete", p) {
            Ok(_) => return,
            Err(e) => draft.error = e.to_string(),
        }
    } else if apply {
        match edit_params(app, &draft).and_then(|p| app.session.execute("markers.edit", p).map_err(|e| e.to_string())) {
            Ok(_) => return,
            Err(e) => draft.error = e,
        }
    }
    app.ui.marker_properties = Some(draft);
}

fn edit_params(app: &FilmcraftApp, draft: &Draft) -> Result<serde_json::Value, String> {
    let Some((_, rate, drop, _)) = draft.owner.details(app) else {
        return Err(tl!("Marker owner is unavailable").into());
    };
    let parse = |value: &str| -> Result<i64, String> {
        let f = filmcraft_time::parse_timecode(value, rate, drop, 0).map_err(|e| e.to_string())?;
        if f < 0 || f > rate.frame_at(filmcraft_time::Tick::MAX) {
            return Err(tl!("Time is outside the supported range").into());
        }
        Ok(f)
    };
    let mut p = draft.owner.params();
    p["marker"] = json!(draft.marker);
    p["name"] = json!(draft.name);
    p["comment"] = json!(draft.comment);
    p["author"] = json!(draft.author);
    p["kind"] = json!(kind_id(draft.kind));
    if draft.url != draft.original.url {
        p["url"] = json!(draft.url);
    }
    p["color"] = json!(if draft.custom { draft.hex.clone() } else { draft.preset.name().to_string() });
    // Do not quantize imported subframe positions when only editing text/color.
    if draft.time != draft.initial_time {
        p["frame"] = json!(parse(&draft.time)?);
    }
    if !draft.range && draft.original_duration != filmcraft_time::Tick::ZERO {
        p["durationFrames"] = json!(0);
    } else if draft.range && (draft.duration != draft.initial_duration || draft.original_duration == filmcraft_time::Tick::ZERO) {
        let duration = parse(&draft.duration)?;
        if duration == 0 {
            return Err(tl!("A range marker needs a positive duration").into());
        }
        p["durationFrames"] = json!(duration);
    }
    Ok(p)
}

fn kind_id(kind: MarkerKind) -> &'static str {
    match kind {
        MarkerKind::Comment => "comment",
        MarkerKind::Chapter => "chapter",
        MarkerKind::Segmentation => "segmentation",
        MarkerKind::WebLink => "webLink",
        MarkerKind::FlashCue => "flashCue",
    }
}

fn kind_label(kind: MarkerKind) -> &'static str {
    match kind {
        MarkerKind::Comment => tl!("Comment Marker"),
        MarkerKind::Chapter => tl!("Chapter Marker"),
        MarkerKind::Segmentation => tl!("Segmentation Marker"),
        MarkerKind::WebLink => tl!("Web Link"),
        MarkerKind::FlashCue => tl!("Flash Cue Point"),
    }
}

fn color_dialog(app: &mut FilmcraftApp, ctx: &egui::Context, draft: &mut Draft) {
    let Some(editor) = &mut draft.color_editor else { return };
    let mut add = false;
    let mut cancel = false;
    let modal = egui::Modal::new(egui::Id::new("marker-custom-color")).show(ctx, |ui| {
        ui.set_width(360.0);
        ui.heading(tl!("Add Custom Color"));
        color_picker::show(ui, &mut app.auto, &mut app.hooks, "markerColor", editor);
        if !draft.error.is_empty() {
            ui.colored_label(ui.visuals().error_fg_color, &draft.error);
        }
        ui.separator();
        ui.horizontal(|ui| {
            let response = ui.add_enabled(editor.value().is_some(), egui::Button::new(tl!("Add color")));
            app.auto.add_response("markerColor.add", &response, tl!("Add color"));
            add = response.clicked();
            let response = ui.button(tl!("Cancel"));
            app.auto.add_response("markerColor.cancel", &response, tl!("Cancel"));
            cancel = response.clicked();
        });
    });
    if cancel || modal.should_close() {
        draft.color_editor = None;
        draft.error.clear();
    } else if add && let Some(color) = editor.value() {
        match app.session.execute("colors.addCustom", json!({"color":color})) {
            Ok(_) => {
                draft.hex = color.hex();
                draft.custom = true;
                draft.color_editor = None;
                draft.error.clear();
            }
            Err(e) => draft.error = e.to_string(),
        }
    }
}

fn add_color_swatch(ui: &mut egui::Ui) -> egui::Response {
    let (chip, response) = allocate_swatch(ui);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, ui.is_enabled(), tl!("Add custom color")));
    let visuals = ui.style().interact(&response);
    ui.painter().rect_filled(chip, 3.0, visuals.bg_fill);
    ui.painter().rect_stroke(chip, 3.0, Stroke::new(1.0, ui.visuals().text_color().gamma_multiply(0.5)), egui::StrokeKind::Outside);
    let center = chip.center();
    let stroke = Stroke::new(1.25, visuals.text_color());
    ui.painter().line_segment([center - vec2(4.0, 0.0), center + vec2(4.0, 0.0)], stroke);
    ui.painter().line_segment([center - vec2(0.0, 4.0), center + vec2(0.0, 4.0)], stroke);
    response
}

pub(super) fn swatch(ui: &mut egui::Ui, rgb: [u8; 3], selected: bool) -> egui::Response {
    let (chip, response) = allocate_swatch(ui);
    let [r, g, b] = rgb;
    ui.painter().rect_filled(chip, 3.0, Color32::from_rgb(r, g, b));
    ui.painter().rect_stroke(chip, 3.0, Stroke::new(1.0, ui.visuals().text_color().gamma_multiply(0.5)), egui::StrokeKind::Outside);
    if selected {
        ui.painter().rect_stroke(chip.expand(3.0), 5.0, Stroke::new(2.0, ui.visuals().text_color()), egui::StrokeKind::Outside);
    }
    response
}

fn allocate_swatch(ui: &mut egui::Ui) -> (egui::Rect, egui::Response) {
    let (rect, response) = ui.allocate_exact_size(vec2(22.0, 22.0), Sense::click());
    (rect.shrink(3.0), response)
}
