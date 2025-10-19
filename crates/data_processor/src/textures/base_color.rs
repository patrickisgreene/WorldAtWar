use crate::processing::generate_scaled_image;
use crate::args::{CliArgs, EarthCommand};

pub fn handle_base_color(args: &CliArgs) {
    let EarthCommand::BaseColor { input, output, resolutions } = &args.command else {
        unreachable!()
    };

    print!("Processing Base Color...");

    // Process sequentially to avoid loading multiple copies of the large source image into memory
    for resolution in resolutions.iter() {
        let mut out_file_path = output.clone();
        out_file_path.extend([&resolution.to_string(), "base_color.png"]);
        if out_file_path.exists() {
            std::fs::remove_file(&out_file_path).unwrap();
        }
        match generate_scaled_image(input, *resolution, &out_file_path) {
            Ok(_) => print!("✔"),
            Err(err) => eprintln!("  Error scaling base_color to {}: {}", resolution, err),
        }
    }

    println!("");
}