//! Mapa del mundo con zoom (rueda), desplazamiento (arrastrar) y selección (clic).

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Rect, Sense, Stroke, TextureHandle, Vec2};
use worldengine_core::{Biome, LocalityKind, Pos, World};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapMode {
    Physical,
    Political,
}

impl MapMode {
    pub const ALL: [MapMode; 2] = [MapMode::Physical, MapMode::Political];

    pub fn label(self) -> &'static str {
        match self {
            MapMode::Physical => "Físico",
            MapMode::Political => "Político",
        }
    }
}

pub struct MapView {
    pub mode: MapMode,
    /// Multiplicador sobre el tamaño que hace caber el mapa completo.
    zoom: f32,
    /// Coordenada (en casillas) que queda en el centro de la vista.
    center: Option<Vec2>,
    texture: Option<TextureHandle>,
}

impl Default for MapView {
    fn default() -> Self {
        Self {
            mode: MapMode::Physical,
            zoom: 1.0,
            center: None,
            texture: None,
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

    /// Dibuja el mapa y devuelve la casilla clicada, si la hay.
    pub fn show(&mut self, ui: &mut egui::Ui, world: &World, selected: Option<Pos>) -> Option<Pos> {
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
                self.zoom = (self.zoom * (scroll * 0.003).exp()).clamp(1.0, 48.0);
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
        let texture = self.texture.get_or_insert_with(|| {
            ui.ctx().load_texture(
                "world-map",
                render(world, self.mode),
                egui::TextureOptions::NEAREST,
            )
        });
        let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
        painter.image(
            texture.id(),
            Rect::from_min_max(to_screen(0.0, 0.0), to_screen(w, h)),
            uv,
            Color32::WHITE,
        );

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

        if self.mode == MapMode::Physical && px >= 5.0 {
            for region in world.regions.iter().filter(|r| r.area >= 40) {
                let p = to_screen(region.center.x as f32 + 0.5, region.center.y as f32 + 0.5);
                label(p, &region.name, 12.0);
            }
        }

        for locality in &world.localities {
            let p = to_screen(locality.pos.x as f32 + 0.5, locality.pos.y as f32 + 0.5);
            let radius = match locality.kind {
                LocalityKind::Village => 2.0,
                LocalityKind::Town => 3.0,
                LocalityKind::City => 4.5,
            } * (px / 6.0).clamp(1.0, 2.0);
            painter.circle(
                p,
                radius,
                Color32::from_rgb(240, 230, 200),
                Stroke::new(1.0_f32, Color32::BLACK),
            );
            let show_name = px >= 8.0 || (locality.kind == LocalityKind::City && px >= 3.0);
            if show_name {
                label(p - Vec2::new(0.0, radius + 1.0), &locality.name, 13.0);
            }
        }

        if self.mode == MapMode::Political {
            for country in &world.countries {
                let capital = world.locality(country.capital).pos;
                let p = to_screen(capital.x as f32 + 0.5, capital.y as f32 + 0.5);
                label(p + Vec2::new(0.0, 22.0), &country.name.to_uppercase(), 15.0);
            }
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

fn biome_color(biome: Biome) -> [u8; 3] {
    match biome {
        Biome::Ocean => [28, 58, 110],
        Biome::Coast => [52, 100, 160],
        Biome::Desert => [214, 190, 130],
        Biome::Grassland => [130, 175, 85],
        Biome::Forest => [52, 120, 60],
        Biome::Jungle => [30, 95, 45],
        Biome::Tundra => [160, 165, 140],
        Biome::Mountain => [125, 110, 100],
        Biome::Glacier => [230, 238, 245],
    }
}

fn render(world: &World, mode: MapMode) -> egui::ColorImage {
    let map = &world.map;
    let mut pixels = Vec::with_capacity(map.tiles.len());
    for (i, tile) in map.tiles.iter().enumerate() {
        let base = biome_color(tile.biome);
        // Sombreado por elevación para dar relieve.
        let shade = if tile.biome.is_water() {
            0.75 + tile.elevation * 0.6
        } else {
            0.8 + (tile.elevation - 0.4) * 0.6
        };
        let mut rgb = base.map(|c| (c as f32 * shade).min(255.0));
        if mode == MapMode::Political {
            let gray = (rgb[0] + rgb[1] + rgb[2]) / 3.0;
            rgb = [gray, gray, gray];
            if let Some(owner) = tile.owner {
                let color = world.country(owner).color;
                let border = map
                    .neighbors(map.pos(i))
                    .any(|n| map.tile(n).unwrap().owner != tile.owner);
                let mix = if border { 0.95 } else { 0.55 };
                for k in 0..3 {
                    rgb[k] = rgb[k] * (1.0 - mix) + color[k] as f32 * mix;
                }
            }
        }
        pixels.push(Color32::from_rgb(rgb[0] as u8, rgb[1] as u8, rgb[2] as u8));
    }
    egui::ColorImage {
        size: [map.width as usize, map.height as usize],
        pixels,
    }
}
