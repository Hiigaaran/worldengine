//! Guarda las imágenes de un mundo: un SVG por cada dibujo de evento y por
//! cada retrato, y una hoja de contactos en PNG con todos juntos.
//! Uso: cargo run -p worldengine-core --example art -- [semilla] [realista|fantasia|cyberpunk] [carpeta]

use resvg::usvg::{self, TreeParsing};
use worldengine_core::art::{self, Motif};
use worldengine_core::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let seed = args.next().map(|s| rng::seed_from_text(&s)).unwrap_or(1);
    let age = args
        .next()
        .and_then(|a| Age::from_text(&a))
        .unwrap_or_default();
    let dir = std::path::PathBuf::from(args.next().unwrap_or_else(|| "imagenes".into()));
    std::fs::create_dir_all(&dir)?;
    let world = generate(&WorldConfig {
        seed,
        age,
        ..Default::default()
    });

    let mut images = Vec::new();
    for motif in Motif::all() {
        images.push((format!("{motif:?}").to_lowercase(), motif.svg(age)));
    }
    // Un personaje de cada rol, vivo o muerto.
    let mut seen = Vec::new();
    for person in &world.people {
        let key = (person.role, person.died.is_some());
        if !seen.contains(&key) {
            seen.push(key);
            images.push((
                format!("retrato-{}", person.id.index()),
                art::portrait(&world, person.id),
            ));
        }
    }
    for (name, svg) in &images {
        std::fs::write(dir.join(format!("{name}.svg")), svg)?;
    }

    let columns = 8;
    let rows = images.len().div_ceil(columns);
    let (w, h) = (columns as u32 * 136, rows as u32 * 136);
    let mut sheet = format!(
        r##"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}"><rect width="{w}" height="{h}" fill="#777"/>"##
    );
    for (i, (_, svg)) in images.iter().enumerate() {
        let (x, y) = ((i % columns) * 136 + 4, (i / columns) * 136 + 4);
        sheet += &svg.replacen("<svg ", &format!(r#"<svg x="{x}" y="{y}" "#), 1);
    }
    sheet += "</svg>";
    let tree = resvg::Tree::from_usvg(&usvg::Tree::from_str(&sheet, &usvg::Options::default())?);
    let mut pixmap = resvg::tiny_skia::Pixmap::new(w, h).ok_or("hoja vacía")?;
    tree.render(resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());
    pixmap.save_png(dir.join("catalogo.png"))?;
    println!("{} imágenes en {}", images.len(), dir.display());
    Ok(())
}
