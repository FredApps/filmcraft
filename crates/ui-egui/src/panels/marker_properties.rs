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
    focus_name: bool,
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
        focus_name: true,
    });
}

enum Action {
    Done,
    Cancel,
    Delete,
    Navigate(Marker),
}

pub fn show(app: &mut FilmcraftApp, ctx: &egui::Context) {
    let Some(mut draft) = app.ui.marker_properties.take() else { return };
    if !Arc::ptr_eq(&draft.project, &app.session.project_instance) {
        return;
    }
    let Some((title, _, _, mut markers)) = draft.owner.details(app) else {
        app.ui.status = tl!("Marker owner is unavailable").into();
        return;
    };
    // Commands can edit or remove this marker while its dialog (or native picker) is open.
    // Unrelated edits do not invalidate the draft. Ties use IDs so coincident markers remain reachable.
    markers.sort_by_key(|m| (m.start, m.id.0));
    let index = markers.iter().position(|m| m.id.0 == draft.marker);
    if index.and_then(|i| markers.get(i)) != Some(&draft.original) {
        app.ui.status = tl!("Marker changed or was removed. Reopen it to continue.").into();
        return;
    }
    if draft.color_editor.is_some() {
        color_dialog(app, ctx, &mut draft);
        app.ui.marker_properties = Some(draft);
        return;
    }
    let previous = index.and_then(|i| i.checked_sub(1)).and_then(|i| markers.get(i));
    let next = index.and_then(|i| i.checked_add(1)).and_then(|i| markers.get(i));
    let mut action = None;
    let modal = egui::Modal::new(egui::Id::new("marker-properties")).show(ctx, |ui| {
        let width = (ctx.content_rect().width() - 40.0).clamp(240.0, 880.0);
        let max_height = (ctx.content_rect().height() - 110.0).max(100.0);
        ui.set_width(width);
        ui.heading(format!("{} @ {}", tl!("Edit Marker"), draft.time));
        ui.label(&title);
        ui.add_space(6.0);
        if width >= 560.0 {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 16.0;
                ui.vertical(|ui| {
                    ui.set_width(width - 112.0);
                    field_area(app, ui, &mut draft, width - 112.0, max_height);
                });
                ui.vertical(|ui| {
                    ui.set_width(96.0);
                    action = actions(app, ui, previous, next, true);
                });
            });
        } else {
            field_area(app, ui, &mut draft, width, (max_height - if width < 400.0 { 190.0 } else { 115.0 }).max(80.0));
            ui.separator();
            action = actions(app, ui, previous, next, false);
        }
        if !draft.error.is_empty() {
            ui.colored_label(ui.visuals().error_fg_color, &draft.error);
        }
    });
    if matches!(action, Some(Action::Cancel)) || modal.should_close() {
        return;
    }
    match action {
        Some(Action::Delete) => {
            let mut p = draft.owner.params();
            p["marker"] = json!(draft.marker);
            match app.session.execute("markers.delete", p) {
                Ok(_) => return,
                Err(e) => draft.error = e.to_string(),
            }
        }
        Some(Action::Done | Action::Navigate(_)) => match save(app, &draft) {
            Ok(()) => {
                if let Some(Action::Navigate(marker)) = action {
                    open_owned(app, &marker, draft.owner);
                }
                return;
            }
            Err(e) => draft.error = e,
        },
        _ => {}
    }
    app.ui.marker_properties = Some(draft);
}

fn field_area(app: &mut FilmcraftApp, ui: &mut egui::Ui, draft: &mut Draft, width: f32, height: f32) {
    ui.allocate_ui_with_layout(vec2(width, height), egui::Layout::top_down(egui::Align::Min), |ui| {
        egui::ScrollArea::vertical().id_salt("marker-fields").max_height(height).show(ui, |ui| {
            ui.set_width(width - 12.0);
            fields(app, ui, draft);
        });
    });
}

fn actions(app: &mut FilmcraftApp, ui: &mut egui::Ui, previous: Option<&Marker>, next: Option<&Marker>, vertical: bool) -> Option<Action> {
    let mut action = None;
    let width = if vertical { 96.0 } else { 128.0 };
    let mut buttons = |ui: &mut egui::Ui| {
        for (id, label, value) in [("apply", tl!("Done"), Action::Done), ("cancel", tl!("Cancel"), Action::Cancel)] {
            let response = ui.add_sized(vec2(width, 28.0), egui::Button::new(label));
            app.auto.add_response(&format!("markerProperties.{id}"), &response, label);
            if response.clicked() {
                action = Some(value);
            }
            if vertical {
                ui.add_space(if id == "apply" { 8.0 } else { 20.0 });
            }
        }
        for (id, label, marker) in [("previous", tl!("Previous"), previous), ("next", tl!("Next"), next)] {
            let response = ui.add_enabled(marker.is_some(), egui::Button::new(label).min_size(vec2(width, 28.0)));
            app.auto.add_response(&format!("markerProperties.{id}"), &response, label);
            if response.clicked()
                && let Some(marker) = marker
            {
                action = Some(Action::Navigate(marker.clone()));
            }
            if vertical {
                ui.add_space(8.0);
            }
        }
        if vertical {
            ui.add_space(8.0);
        }
        let response = ui.add_sized(vec2(width, 28.0), egui::Button::new(tl!("Delete Marker")));
        app.auto.add_response("markerProperties.delete", &response, tl!("Delete Marker"));
        if response.clicked() {
            action = Some(Action::Delete);
        }
    };
    if vertical {
        ui.vertical(&mut buttons);
    } else {
        ui.horizontal_wrapped(&mut buttons);
    }
    ui.add_space(8.0);
    ui.small(tl!("Previous and Next save this marker."));
    action
}

fn field_row(ui: &mut egui::Ui, label: &str, contents: impl FnOnce(&mut egui::Ui)) {
    ui.horizontal_top(|ui| {
        ui.allocate_ui_with_layout(vec2(76.0, 24.0), egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(label);
        });
        ui.vertical(contents);
    });
}

fn fields(app: &mut FilmcraftApp, ui: &mut egui::Ui, draft: &mut Draft) {
    field_row(ui, tl!("Name"), |ui| {
        let response = ui.add(egui::TextEdit::singleline(&mut draft.name).desired_width(f32::INFINITY));
        app.auto.add_response("markerProperties.name", &response, tl!("Name"));
        if draft.focus_name {
            response.request_focus();
            draft.focus_name = false;
        }
    });
    ui.add_space(10.0);
    field_row(ui, tl!("Time"), |ui| {
        ui.horizontal_wrapped(|ui| {
            let response = ui.add(egui::TextEdit::singleline(&mut draft.time).desired_width(110.0));
            app.auto.add_response("markerProperties.time", &response, tl!("Time"));
            ui.add_space(16.0);
            ui.label(tl!("Duration"));
            let response = ui.add_enabled(draft.range, egui::TextEdit::singleline(&mut draft.duration).desired_width(110.0));
            app.auto.add_response("markerProperties.duration", &response, tl!("Duration"));
            let response = ui.checkbox(&mut draft.range, tl!("Range"));
            app.auto.add_response("markerProperties.range", &response, tl!("Range marker"));
            if response.changed() && draft.range && draft.original_duration == filmcraft_time::Tick::ZERO && draft.duration == draft.initial_duration {
                draft.duration = "00:00:00:01".into();
            }
        });
    });
    ui.add_space(10.0);
    field_row(ui, tl!("Notes"), |ui| {
        let response = ui.add(egui::TextEdit::multiline(&mut draft.comment).desired_width(f32::INFINITY).desired_rows(8));
        app.auto.add_response("markerProperties.comment", &response, tl!("Notes"));
    });
    ui.add_space(6.0);
    field_row(ui, tl!("Author"), |ui| {
        let response = ui.add(egui::TextEdit::singleline(&mut draft.author).desired_width(f32::INFINITY));
        app.auto.add_response("markerProperties.author", &response, tl!("Author"));
    });
    ui.add_space(14.0);
    ui.group(|ui| {
        ui.set_width(ui.available_width().max(100.0));
        ui.label(tl!("Options"));
        field_row(ui, tl!("Color"), |ui| {
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
        });
        ui.add_space(10.0);
        let group = ui.vertical(|ui| {
            for kind in [MarkerKind::Comment, MarkerKind::Chapter, MarkerKind::Segmentation, MarkerKind::WebLink, MarkerKind::FlashCue] {
                let label = kind_label(kind);
                let response = ui.radio_value(&mut draft.kind, kind, label);
                app.auto.add_response(&format!("markerProperties.kind.{}", kind_id(kind)), &response, label);
                if kind == MarkerKind::WebLink && draft.kind == MarkerKind::WebLink {
                    field_row(ui, tl!("URL"), |ui| {
                        ui.horizontal_wrapped(|ui| {
                            let response = ui.add(
                                egui::TextEdit::singleline(&mut draft.url)
                                    .desired_width((ui.available_width() - 90.0).max(100.0))
                                    .char_limit(8192)
                                    .hint_text("https://"),
                            );
                            app.auto.add_response("markerProperties.url", &response, tl!("URL"));
                            let address = filmcraft_project::marker_web_url(&draft.url);
                            let response = ui.add_enabled(address.is_some(), egui::Button::new(tl!("Open Link")));
                            let response = response.on_hover_text(tl!("Opens this address in your web browser"));
                            app.auto.add_response("markerProperties.openLink", &response, tl!("Open Link"));
                            if response.clicked()
                                && let Some(address) = address
                            {
                                crate::links::open(app, ui.ctx(), &address);
                                if let Some(failure) = app.ui.failed_link.as_ref().filter(|failure| failure.url == address) {
                                    draft.error.clone_from(&failure.error);
                                }
                            }
                        });
                    });
                    if !draft.url.trim().is_empty() && filmcraft_project::marker_web_url(&draft.url).is_none() {
                        ui.colored_label(ui.visuals().error_fg_color, tl!("Enter a valid HTTP or HTTPS URL without credentials"));
                    }
                }
            }
        });
        app.auto.add_response("markerProperties.kind", &group.response, tl!("Type"));
    });
}

/// Navigation commits only this marker; unchanged navigation does not dirty the project or Undo.
fn save(app: &mut FilmcraftApp, draft: &Draft) -> Result<(), String> {
    let params = edit_params(app, draft)?;
    let original = &draft.original;
    let changed = draft.name != original.name
        || draft.author != original.author
        || draft.comment != original.comment
        || draft.kind != original.kind
        || draft.url != original.url
        || draft.time != draft.initial_time
        || draft.range != (original.duration > filmcraft_time::Tick::ZERO)
        || (draft.range && draft.duration != draft.initial_duration)
        || draft.custom != original.custom_color.is_some()
        || if draft.custom { HexColor::parse(&draft.hex) != original.custom_color } else { draft.preset != original.color };
    if changed {
        app.session.execute("markers.edit", params).map_err(|e| e.to_string())?;
    }
    Ok(())
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
