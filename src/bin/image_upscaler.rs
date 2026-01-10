use std::{env::args, path::PathBuf};

use image::{ImageReader, imageops::FilterType};

pub fn main() {
    let args: Vec<_> = args().skip(1).collect();
    assert!(args.len() == 2);
    let image = ImageReader::open(&args[0]).unwrap().decode().unwrap();
    let scale_factor: u32 = args[1].parse().unwrap();
    let image = image.resize(
        image.width() * scale_factor,
        image.height() * scale_factor,
        FilterType::Nearest,
    );
    let mut new_path = PathBuf::from(&args[0]);
    let ext = new_path.extension().unwrap().to_owned();
    new_path.set_extension("");
    new_path.set_file_name(
        new_path.file_name().unwrap().to_str().unwrap().to_owned() + &format!("-{}", scale_factor),
    );
    new_path.set_extension(ext);
    image.save(new_path).unwrap();
}
