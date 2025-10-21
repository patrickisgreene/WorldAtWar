use crate::args::{CliArgs, EarthCommand};
use crate::processing::scale_image;
use rayon::prelude::*;

pub fn handle_topography(args: &CliArgs) {
    let EarthCommand::Topography {
        input,
        output,
        resolutions,
    } = &args.command
    else {
        unreachable!()
    };

    print!("Processing topography...");

    resolutions.par_iter().for_each(|resolution| {
        let mut out_file_path = output.clone();
        out_file_path.extend([&resolution.to_string(), "topography.png"]);
        if out_file_path.exists() {
            std::fs::remove_file(&out_file_path).unwrap();
        }
        match scale_image(input, *resolution, &out_file_path) {
            Ok(_) => print!("✔"),
            Err(err) => eprintln!("  Error scaling topography to {}: {}", resolution, err),
        }
    });

    println!("");
}
