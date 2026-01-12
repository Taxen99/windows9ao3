use std::{env::args, path::PathBuf};

use image::{ImageReader, imageops::FilterType};

pub fn main() {
    let args: Vec<_> = args().skip(1).collect();
    assert!(args.len() >= 2);
    let remove_white = args.get(2).unwrap_or(&"".into()) == "--rw";
    let mut image = ImageReader::open(&args[0]).unwrap().decode().unwrap();
    if remove_white {
        // TODO: this is shitty shit
        image
            .as_mut_rgba8()
            .unwrap()
            .chunks_exact_mut(4)
            .for_each(|x| {
                if x[0] == 255 && x[1] == 255 && x[2] == 255 {
                    x[0] = 0;
                    x[1] = 0;
                    x[2] = 0;
                    x[3] = 0;
                }
            });
    }
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
