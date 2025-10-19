use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::args::{CliArgs, EarthCommand};
use crate::processing::generate_mask;

pub fn handle_river_mask(args: &CliArgs) {
    let EarthCommand::RiverMask {
        input,
        output,
        resolutions,
    } = &args.command
    else {
        unreachable!()
    };

    print!("Processing river mask...");

    resolutions.par_iter().for_each(|resolution| {
        let mut out_file_path = output.clone();
        out_file_path.extend([&resolution.to_string(), "river_mask.png"]);
        if out_file_path.exists() {
            std::fs::remove_file(&out_file_path).unwrap();
        }
        match generate_mask(input, *resolution, &out_file_path, true) {
            Ok(_) => print!("✔"),
            Err(err) => eprintln!("  Error scaling river mask to {}: {}", resolution, err),
        }
    });

    println!("");
}
