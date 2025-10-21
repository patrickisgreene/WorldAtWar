use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::processing::generate_scaled_image;
use crate::args::{CliArgs, EarthCommand};

pub fn handle_chlorophyll(args: &CliArgs) {
    let EarthCommand::Chlorophyll { input, output, resolutions } = &args.command else {
        unreachable!()
    };
    
    print!("Processing chlorophyll...");

    resolutions.par_iter().for_each(|resolution| {
        let mut out_file_path = output.clone();
        out_file_path.extend([&resolution.to_string(), "chlorophyll.png"]);
        if out_file_path.exists() {
            std::fs::remove_file(&out_file_path).unwrap();
        }
        match generate_scaled_image(input, *resolution, &out_file_path) {
            Ok(_) => print!("✔"),
            Err(err) => eprintln!("  Error scaling chlorophyll to {}: {}", resolution, err),
        }
    });

    println!("");
}