//! Render genesis layers to BMP: the pre-viewer eyeball loop.
//! `map-export --seed 42 --size 192 --layer terrain --out maps/terrain-42.bmp`

use map_export::{bmp, palette};
use sim_events::WorldSeed;
use std::path::PathBuf;

fn main() {
    let mut seed = 42u64;
    let mut size = 192u32;
    let mut layer = String::from("terrain");
    let mut out: Option<PathBuf> = None;

    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < args.len() {
        let value = args.get(i + 1).cloned();
        match (args[i].as_str(), value) {
            ("--seed", Some(v)) => seed = v.parse().unwrap_or_else(|_| usage()),
            ("--size", Some(v)) => size = v.parse().unwrap_or_else(|_| usage()),
            ("--layer", Some(v)) => layer = v,
            ("--out", Some(v)) => out = Some(PathBuf::from(v)),
            _ => usage(),
        }
        i += 2;
    }

    let color: fn(&sim_server::Genesis, usize) -> palette::Rgb = match layer.as_str() {
        "terrain" => palette::terrain,
        "height" => palette::height,
        "flora" => palette::flora_layer,
        _ => usage(),
    };

    let genesis = sim_server::genesis(WorldSeed(seed), size, 24, &tuning::Deep::default());
    let cells = genesis.fields.grid().cells();
    let rgb: Vec<palette::Rgb> = (0..cells).map(|i| color(&genesis, i)).collect();

    let path = out.unwrap_or_else(|| PathBuf::from(format!("maps/{layer}-{seed}.bmp")));
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).expect("create output directory");
    }
    bmp::write(&path, size, size, &rgb).expect("write bmp");

    let habitable = (0..cells)
        .filter(|&t| world_map::tiles::habitable(&genesis.fields, t))
        .count();
    println!(
        "{} · seed {seed} · {size}² tiles · land {} · habitable {habitable} · {} flora species",
        path.display(),
        genesis.fields.land_cells(),
        genesis.flora.species.len(),
    );
}

fn usage() -> ! {
    eprintln!(
        "usage: map-export [--seed N] [--size N] \
         [--layer terrain|height|flora] [--out path.bmp]"
    );
    std::process::exit(2)
}
