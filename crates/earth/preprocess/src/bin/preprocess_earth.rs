use gdal::raster::GdalDataType;
use waw_earth_render::prelude::*;
use waw_earth_preprocess::prelude::*;

const ATTACHMENTS: [(&str, AttachmentLabel, GdalDataType, AttachmentFormat); 5] = [
    (
        "resources/earth/topography.tif",
        AttachmentLabel::Topography,
        GdalDataType::UInt8,
        AttachmentFormat::R8Unorm,
    ),
    (
        "resources/earth/bathyometry.tif",
        AttachmentLabel::Bathyometry,
        GdalDataType::UInt8,
        AttachmentFormat::R8Unorm
    ),
    (
        "resources/earth/daytime.tif",
        AttachmentLabel::DayTime,
        GdalDataType::UInt8,
        AttachmentFormat::Rgb8U,
    ),
    (
        "resources/earth/nighttime.tif",
        AttachmentLabel::NightTime,
        GdalDataType::UInt8,
        AttachmentFormat::Rgb8U
    ),
    (
        "resources/earth/ocean.tif",
        AttachmentLabel::OceanMask,
        GdalDataType::Float32,
        AttachmentFormat::R32F
    ),
];

fn main() {
    for (path, attachment_label, data_type, format) in ATTACHMENTS {
        println!("Processing: {path:?}");
        let args = EarthCli {
            data_type: PreprocessDataType::DataType(data_type),
            src_path: vec![path.into()],
            overwrite: true,
            attachment_label,
            format,
            ..Default::default()
        };
        let (src_dataset, mut context) = PreprocessContext::from_cli(args).unwrap();
        // Use original working version (reproject → split → downsample)
        preprocess(src_dataset, &mut context);
    }
}
