use image::DynamicImage;
use image::ImageError;
use std::borrow::Cow;

use arboard::Clipboard;
use ascii_image::{scale, ImageData};
use clap::Parser;
use image::io::Reader as ImageReader;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(short, long, value_parser, conflicts_with("clipboard"))]
    file_path: Option<String>,

    #[clap(short, long, value_parser, default_value = "false")]
    clipboard: bool,

    #[clap(long, value_parser, default_value = "200")]
    width: usize,

    #[clap(long, value_parser, default_value = "200")]
    height: usize,
}

fn main() {
    let args = Cli::parse();

    let image_data;
    if args.clipboard {
        let mut clipboard = Clipboard::new().unwrap();
        image_data = match clipboard.get_image() {
            Ok(img) => ImageData {
                width: img.width,
                height: img.height,
                data: img.bytes,
            },
            Err(e) => {
                eprintln!("error getting image: {}", e);
                return;
            }
        };
    } else {
        match get_image_from_file(args.file_path.unwrap()) {
            Ok(image) => {
                image_data = ImageData {
                    width: image.width() as usize,
                    height: image.height() as usize,
                    data: Cow::from(image.into_bytes()),
                };
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                return;
            }
        }
    }

    let scaled_image = scale(&image_data, args.width, args.height);
    println!("{}", scaled_image);
}

fn get_image_from_file(file_path: String) -> Result<DynamicImage, ImageError> {
    ImageReader::open(file_path)?.decode()
}
