//! Mapa del mundo con zoom (rueda), desplazamiento (arrastrar) y selección (clic).

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, TextureHandle, Vec2};
use worldengine_core::{LocalityKind, Pos, World};

use crate::render::render;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapMode {
    Physical,
    Political,
    Elevation,
    Temperature,
    Rainfall,
}

impl MapMode {
    pub const ALL: [MapMode; 5] = [
        MapMode::Physical,
        MapMode::Political,
        MapMode::Elevation,
        MapMode::Temperature,
        MapMode::Rainfall,
    ];

    pub fn label(self) -> &'static str {
        match self {
            MapMode::Physical => "Físico",
            MapMode::Political => "Político",
            MapMode::Elevation => "Elevación",
            MapMode::Temperature => "Temperatura",
            MapMode::Rainfall => "Lluvia",
        }
    }
}

/// Caudal mínimo para dibujar un río, según el zoom: de lejos solo se ven los
/// grandes; al acercarse aparecen los afluentes.
fn min_visible_flow(px: f32) -> f32 {
    (600.0 / px.max(1.0)).clamp(30.0, 300.0)
}

pub struct MapView {
    pub mode: MapMode,
    /// Multiplicador sobre el tamaño que hace caber el mapa completo.
    zoom: f32,
    /// Coordenada (en casillas) que queda en el centro de la vista.
    center: Option<Vec2>,
    texture: Option<TextureHandle>,
    /// Qué mundo y año está pintado en la textura; si cambia, se repinta.
    texture_key: Option<(u64, i32, usize)>,
}

impl Default for MapView {
    fn default() -> Self {
        Self {
            mode: MapMode::Physical,
            zoom: 1.0,
            center: None,
            texture: None,
            texture_key: None,
        }
    }
}

impl MapView {
    pub fn invalidate(&mut self) {
        self.texture = None;
        self.center = None;
        self.zoom = 1.0;
    }

    pub fn set_mode(&mut self, mode: MapMode) {
        if self.mode != mode {
            self.mode = mode;
            self.texture = None;
        }
    }

    pub fn center_on(&mut self, pos: Pos) {
        self.center = Some(Vec2::new(pos.x as f32 + 0.5, pos.y as f32 + 0.5));
        self.zoom = self.zoom.max(4.0);
    }

    /// Dibuja el mapa y devuelve la casilla clicada, si la hay. `markers` son
    /// lugares a destacar, como los hechos del año que se está viendo.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        world: &World,
        selected: Option<Pos>,
        markers: &[Pos],
    ) -> Option<Pos> {
        let (w, h) = (world.map.width as f32, world.map.height as f32);
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        let rect = response.rect;
        let fit = (rect.width() / w).min(rect.height() / h);
        let center = self.center.get_or_insert(Vec2::new(w / 2.0, h / 2.0));

        let mut px = fit * self.zoom;
        if response.dragged() {
            *center -= response.drag_delta() / px;
        }
        if let (true, Some(cursor)) = (response.hovered(), response.hover_pos()) {
            let scroll = ui.input(|i| i.smooth_scroll_delta.y);
            if scroll != 0.0 {
                // Zoom manteniendo fija la casilla bajo el cursor.
                let under_cursor = *center + (cursor - rect.center()) / px;
                self.zoom = (self.zoom * (scroll * 0.003).exp()).clamp(1.0, 24.0);
                px = fit * self.zoom;
                *center = under_cursor - (cursor - rect.center()) / px;
            }
        }
        center.x = center.x.clamp(0.0, w);
        center.y = center.y.clamp(0.0, h);
        let origin = rect.center() - *center * px;
        let to_screen = |x: f32, y: f32| origin + Vec2::new(x, y) * px;
        let to_tile = |p: Pos2| {
            let t = (p - origin) / px;
            (t.x >= 0.0 && t.y >= 0.0 && t.x < w && t.y < h).then_some(Pos {
                x: t.x as u32,
                y: t.y as u32,
            })
        };

        let painter = painter.with_clip_rect(rect);
        painter.rect_filled(rect, 0.0, Color32::from_rgb(10, 20, 40));
        // Solo el mapa político cambia con los años.
        let key = match self.mode {
            MapMode::Political => (world.config.seed, world.year, world.events.len()),
            _ => (world.config.seed, 0, 0),
        };
        if self.texture_key != Some(key) {
            self.texture = None;
            self.texture_key = Some(key);
        }
        let texture = self.texture.get_or_insert_with(|| {
            ui.ctx().load_texture(
                "world-map",
                render(world, self.mode),
                egui::TextureOptions::LINEAR,
            )
        });
        let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
        painter.image(
            texture.id(),
            Rect::from_min_max(to_screen(0.0, 0.0), to_screen(w, h)),
            uv,
            Color32::WHITE,
        );

        // Ríos: un trazo por tramo, más ancho cuanto más caudal lleva.
        let view = rect.expand(px * 2.0);
        let min_flow = min_visible_flow(px);
        let river_color = Color32::from_rgb(70, 130, 200);
        let river_text = Color32::from_rgb(185, 220, 255);
        let mut river_labels = Vec::new();
        for river in &world.rivers {
            if river.discharge < min_flow {
                continue;
            }
            let center = |p: Pos| to_screen(p.x as f32 + 0.5, p.y as f32 + 0.5);
            let points: Vec<(Pos2, f32)> = river
                .path
                .iter()
                .map(|&p| (center(p), world.map.tile(p).unwrap().flow))
                .chain(std::iter::once((center(river.mouth), river.discharge)))
                .collect();
            let points = smooth(&smooth(&points));
            for pair in points.windows(2) {
                let ((a, _), (b, flow)) = (pair[0], pair[1]);
                if flow < min_flow || !(view.contains(a) || view.contains(b)) {
                    continue;
                }
                let width = ((flow / 30.0).ln() * 0.6 + 0.8) * (px / 8.0).clamp(0.6, 3.0);
                painter.line_segment([a, b], Stroke::new(width, river_color));
            }
            if px >= 10.0 && river.path.len() >= 6 {
                river_labels.push((center(river.path[river.path.len() / 2]), &river.name));
            }
        }

        let text_color = ui.visuals().strong_text_color();
        let shadow = Color32::from_black_alpha(180);
        let label = |pos: Pos2, text: &str, size: f32| {
            let font = FontId::proportional(size);
            painter.text(
                pos + Vec2::splat(1.0),
                Align2::CENTER_BOTTOM,
                text,
                font.clone(),
                shadow,
            );
            painter.text(pos, Align2::CENTER_BOTTOM, text, font, text_color);
        };

        for (p, name) in river_labels {
            let font = FontId::proportional(11.0);
            painter.text(
                p + Vec2::splat(1.0),
                Align2::CENTER_CENTER,
                name,
                font.clone(),
                shadow,
            );
            painter.text(p, Align2::CENTER_CENTER, name, font, river_text);
        }

        if self.mode == MapMode::Physical && px >= 3.0 {
            // Al acercarse se nombran también las regiones pequeñas.
            let min_area = (4000.0 / (px * px)).clamp(12.0, 200.0) as u32;
            for region in world.regions.iter().filter(|r| r.area >= min_area) {
                let p = to_screen(region.center.x as f32 + 0.5, region.center.y as f32 + 0.5);
                label(p, &region.name, 12.0);
            }
        }

        let scale = (px / 6.0).clamp(1.0, 2.0);
        for locality in &world.localities {
            let p = to_screen(locality.pos.x as f32 + 0.5, locality.pos.y as f32 + 0.5);
            if locality.ruined.is_some() {
                // Ruinas: una cruz gris, visible solo de cerca.
                if px >= 6.0 {
                    let d = Vec2::splat(2.5 * scale);
                    let stroke = Stroke::new(1.5_f32, Color32::from_gray(90));
                    painter.line_segment([p - d, p + d], stroke);
                    painter
                        .line_segment([p + Vec2::new(d.x, -d.y), p + Vec2::new(-d.x, d.y)], stroke);
                }
                continue;
            }
            let is_capital = locality
                .country
                .is_some_and(|c| world.country(c).capital == locality.id);
            let radius = match locality.kind {
                LocalityKind::Village => 2.0,
                LocalityKind::Town => 3.0,
                LocalityKind::City => 4.5,
            } * scale;
            if is_capital {
                painter.circle_stroke(
                    p,
                    radius + 2.5,
                    Stroke::new(1.5_f32, Color32::from_rgb(250, 210, 90)),
                );
            }
            painter.circle(
                p,
                radius,
                Color32::from_rgb(240, 230, 200),
                Stroke::new(1.0_f32, Color32::BLACK),
            );
            let show_name = match locality.kind {
                LocalityKind::City => px >= 3.0,
                LocalityKind::Town => px >= 6.0,
                LocalityKind::Village => px >= 10.0,
            };
            if show_name || is_capital {
                label(p - Vec2::new(0.0, radius + 1.0), &locality.name, 13.0);
            }
        }

        if self.mode == MapMode::Political {
            for country in world.countries.iter().filter(|c| c.exists()) {
                let capital = world.locality(country.capital).pos;
                let p = to_screen(capital.x as f32 + 0.5, capital.y as f32 + 0.5);
                label(p + Vec2::new(0.0, 22.0), &country.name.to_uppercase(), 15.0);
            }
        }

        let pulse = Color32::from_rgb(235, 70, 60);
        for &m in markers {
            let p = to_screen(m.x as f32 + 0.5, m.y as f32 + 0.5);
            painter.circle_stroke(p, (px * 1.2).max(9.0), Stroke::new(2.5_f32, pulse));
        }

        if let Some(sel) = selected {
            let r = Rect::from_min_max(
                to_screen(sel.x as f32, sel.y as f32),
                to_screen(sel.x as f32 + 1.0, sel.y as f32 + 1.0),
            );
            painter.rect_stroke(r.expand(1.0), 0.0, Stroke::new(2.0_f32, Color32::YELLOW));
        }

        if let Some(tile_pos) = response.hover_pos().and_then(to_tile) {
            let tile = world.map.tile(tile_pos).unwrap();
            let mut text = format!("({}, {})  {}", tile_pos.x, tile_pos.y, tile.biome.name());
            if let Some(region) = tile.region {
                text += &format!(" · {}", world.region(region).name);
            }
            if let Some(river) = tile.river {
                text += &format!(" · {}", world.river(river).name);
            }
            if let Some(owner) = tile.owner {
                text += &format!(" · {}", world.country(owner).name);
            }
            let font = FontId::proportional(13.0);
            painter.text(
                rect.left_bottom() + Vec2::new(9.0, -7.0),
                Align2::LEFT_BOTTOM,
                &text,
                font.clone(),
                shadow,
            );
            painter.text(
                rect.left_bottom() + Vec2::new(8.0, -8.0),
                Align2::LEFT_BOTTOM,
                &text,
                font,
                text_color,
            );
        }

        response
            .clicked()
            .then(|| response.interact_pointer_pos().and_then(to_tile))
            .flatten()
    }
}

/// Suavizado de Chaikin: recorta las esquinas del trazo de un río. Conserva los
/// extremos para que el río siga naciendo y desembocando en el mismo sitio.
fn smooth(points: &[(Pos2, f32)]) -> Vec<(Pos2, f32)> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut out = vec![points[0]];
    for pair in points.windows(2) {
        let ((a, fa), (b, fb)) = (pair[0], pair[1]);
        out.push((a.lerp(b, 0.25), fa * 0.75 + fb * 0.25));
        out.push((a.lerp(b, 0.75), fa * 0.25 + fb * 0.75));
    }
    out.push(points[points.len() - 1]);
    out
}
