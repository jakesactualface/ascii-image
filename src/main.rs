use ascii_image::RectSize;
use std::borrow::Cow;

use arboard::Clipboard;
use ascii_image::{scale, ImageData};
use clap::Parser;
use image::io::Reader as ImageReader;

const DEFAULT_WIDTH: usize = 128;
const DEFAULT_HEIGHT: usize = 64;

#[derive(Parser, Debug)]
struct Cli {
    #[clap(
        short,
        long,
        value_parser,
        conflicts_with("clipboard"),
        required_unless_present("clipboard")
    )]
    file_path: Option<String>,

    #[clap(short, long, value_parser, default_value = "false")]
    clipboard: bool,

    #[clap(long, value_parser, requires("height"))]
    width: Option<usize>,

    #[clap(long, value_parser, requires("width"))]
    height: Option<usize>,
}

fn main() {
    let args = Cli::parse();

    let image_data;
    if args.clipboard {
        image_data = get_clipboard_image();
    } else {
        image_data = get_image_from_file(args.file_path.unwrap());
    }

    if image_data.is_none() {
        return;
    }

    let image_data = image_data.unwrap();

    let output_size = match (args.width, args.height) {
        (Some(width), Some(height)) => RectSize { width, height },
        (None, None) => {
            if let Some(terminal_size) = get_terminal_size() {
                terminal_size
            } else {
                RectSize {
                    width: DEFAULT_WIDTH,
                    height: DEFAULT_HEIGHT,
                }
            }
        }
        _ => panic!("Width and height must both be set!"),
    };

    let scaled_image = scale(&image_data, output_size);
    println!("{}", scaled_image);
}

fn get_terminal_size() -> Option<RectSize> {
    return match term_size::dimensions() {
        Some((width, height)) => Some(RectSize { width, height }),
        None => None,
    };
}

fn get_clipboard_image() -> Option<ImageData<'static>> {
    let mut clipboard = Clipboard::new().unwrap();
    return match clipboard.get_image() {
        Ok(img) => Some(ImageData {
            width: img.width,
            height: img.height,
            data: img.bytes,
        }),
        Err(_e) => {
            eprintln!("No image found in clipboard!");
            None
        }
    };
}

fn get_image_from_file(file_path: String) -> Option<ImageData<'static>> {
    let image = match ImageReader::open(file_path) {
        Ok(image) => image.decode(),
        Err(e) => {
            eprintln!("Error getting image from file: {}", e);
            return None;
        }
    };
    return match image {
        Ok(decoded) => Some(ImageData {
            width: decoded.width() as usize,
            height: decoded.height() as usize,
            data: Cow::from(decoded.into_bytes()),
        }),
        Err(e) => {
            eprintln!("Error decoding image file contents: {}", e);
            None
        }
    };
}
