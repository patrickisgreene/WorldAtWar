use std::path::Path;

use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::args::{CliArgs, EarthCommand, ImageResolution};
use crate::processing::scale_image;

pub fn handle_light_mask(args: &CliArgs) {
    let EarthCommand::LightMask {
        input,
        output,
        resolutions,
    } = &args.command
    else {
        unreachable!()
    };

    print!("Processing light mask...");

    resolutions.par_iter().for_each(|resolution| {
        let mut out_file_path = output.clone();
        out_file_path.extend([&resolution.to_string(), "light_mask.png"]);
        if out_file_path.exists() {
            std::fs::remove_file(&out_file_path).unwrap();
        }
        match generate_light_mask(input, *resolution, &out_file_path) {
            Ok(_) => print!("✔"),
            Err(err) => eprintln!("  Error scaling light mask to {}: {}", resolution, err),
        }
    });

    println!("");
}

fn generate_light_mask(
    input_file: &Path,
    resolution: ImageResolution,
    output_file: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    scale_image(input_file, resolution, output_file)
}
