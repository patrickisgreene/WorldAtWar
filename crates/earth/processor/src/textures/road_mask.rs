use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::args::{CliArgs, EarthCommand};
use crate::processing::generate_mask;

// TODO: Invert the road mask
pub fn handle_road_mask(args: &CliArgs) {
    let EarthCommand::RoadMask {
        input,
        output,
        resolutions,
    } = &args.command
    else {
        unreachable!()
    };

    print!("Processing road mask...");

    resolutions.par_iter().for_each(|resolution| {
        let mut out_file_path = output.clone();
        out_file_path.extend([&resolution.to_string(), "road_mask.png"]);
        if out_file_path.exists() {
            std::fs::remove_file(&out_file_path).unwrap();
        }
        match generate_mask(input, *resolution, &out_file_path, true) {
            Ok(_) => print!("✔"),
            Err(err) => eprintln!("  Error scaling road mask to {}: {}", resolution, err),
        }
    });

    println!("");
}
