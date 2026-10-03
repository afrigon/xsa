mod arguments;
mod cube_face;
mod cube_map_dds;
mod equirectangular_map;

use std::thread;

use anyhow::Context;
use glam::Vec3;

use arguments::Arguments;
use cube_face::{CUBE_FACE_COUNT, CubeFace};
use cube_map_dds::CubeMapDds;
use equirectangular_map::EquirectangularMap;

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse()?;
    let map = EquirectangularMap::open(&arguments.input)?;
    map.print_brightness_percentiles();

    let faces: Vec<CubeFace> = thread::scope(|scope| {
        let workers: Vec<_> = (0..CUBE_FACE_COUNT)
            .map(|face| {
                let map = &map;
                scope.spawn(move || CubeFace::project(map, face))
            })
            .collect();

        workers
            .into_iter()
            .map(|worker| worker.join().expect("face worker panicked"))
            .collect()
    });
    let mip_chains: Vec<Vec<CubeFace>> = faces.into_iter().map(CubeFace::mip_chain).collect();

    let average = mip_chains
        .iter()
        .map(|chain| chain[chain.len() - 1].texels[0])
        .sum::<Vec3>()
        / CUBE_FACE_COUNT as f32;
    println!("average linear color before exposure: {average}");
    println!(
        "average linear color after exposure: {}",
        (average * arguments.exposure).min(Vec3::ONE)
    );

    CubeMapDds { mip_chains }
        .write(&arguments.output, arguments.exposure)
        .with_context(|| format!("writing {}", arguments.output))
}
