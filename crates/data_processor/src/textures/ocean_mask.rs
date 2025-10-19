use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::args::{CliArgs, EarthCommand};
use crate::processing::generate_mask;

pub fn handle_ocean_mask(args: &CliArgs) {
    let EarthCommand::OceanMask {
        input,
        output,
        resolutions,
    } = &args.command
    else {
        unreachable!()
    };

    print!("Processing ocean mask...");

    resolutions.par_iter().for_each(|resolution| {
        let mut out_file_path = output.clone();
        out_file_path.extend([&resolution.to_string(), "ocean_mask.png"]);
        if out_file_path.exists() {
            std::fs::remove_file(&out_file_path).unwrap();
        }
        match generate_mask(input, *resolution, &out_file_path, false) {
            Ok(_) => print!("✔"),
            Err(err) => eprintln!("  Error scaling ocean_mask to {}: {}", resolution, err),
        }
    });

    println!("");
}
