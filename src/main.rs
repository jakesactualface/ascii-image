use ascii_image::RectSize;
use std::{
    borrow::Cow,
    fs::File,
    io::{BufReader, Read},
    process::Command,
};

use arboard::Clipboard;
use ascii_image::{scale, ImageData};
use base64::{engine::general_purpose, Engine as _};
use clap::{builder::ArgPredicate, Parser};

const DEFAULT_WIDTH: usize = 128;
const DEFAULT_HEIGHT: usize = 64;

#[derive(clap::ValueEnum, Clone, Debug)]
enum ScalingBehavior {
    Scale,
    Stretch,
}

#[derive(Parser, Debug)]
struct Cli {
    #[clap(short, long, value_parser, conflicts_with("clipboard"))]
    filepath: Option<String>,

    #[clap(
        short,
        long,
        value_parser,
        default_value = "true",
        default_value_if("filepath", ArgPredicate::IsPresent, "false")
    )]
    clipboard: bool,

    #[clap(long, value_parser, requires("height"))]
    width: Option<usize>,

    #[clap(long, value_parser, requires("width"))]
    height: Option<usize>,

    #[clap(short, long, value_enum, default_value_t=ScalingBehavior::Scale)]
    scaling: ScalingBehavior,

    #[clap(short, long, value_parser, default_value = "false")]
    verbose: bool,
}

fn main() {
    let args = Cli::parse();

    let image_data = if args.clipboard {
        get_clipboard_image()
    } else {
        get_image_from_file(args.filepath.unwrap())
    };

    if image_data.is_none() {
        return;
    }

    let image_data = image_data.unwrap();

    let mut output_size = match (args.width, args.height) {
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

    if let ScalingBehavior::Scale = args.scaling {
        output_size = trim_to_aspect_ratio(
            image_data.width,
            image_data.height,
            output_size,
            args.verbose,
        )
    }

    let scaled_image = scale(&image_data, output_size);
    println!("{}", scaled_image);
}

fn get_terminal_size() -> Option<RectSize> {
    term_size::dimensions().map(|(width, height)| RectSize { width, height })
}

fn trim_to_aspect_ratio(
    start_width: usize,
    start_height: usize,
    output_size: RectSize,
    verbose: bool,
) -> RectSize {
    let buffer = 5;

    let desired_ratio = start_width as f32 / start_height as f32;

    // Skew ratio to account for text characters not being perfect squares
    let desired_ratio = desired_ratio * 2.0;

    let target_width = (desired_ratio * (output_size.height as f32).ceil()) as usize;
    let target_height = (output_size.width as f32 / desired_ratio).ceil() as usize;

    match (target_width, target_height) {
        (width, height) if width > output_size.width.saturating_add(buffer) => {
            // Ratio would cause width to be too high, decrease height to compensate
            if verbose {
                println!(
                    "Decreasing width to maintain aspect ratio. Aspect ratio wanted width: {width}"
                );
            }
            RectSize {
                width: output_size.width,
                height,
            }
        }
        (width, height) if height > output_size.height.saturating_add(buffer) => {
            // Ratio would cause height to be too high, decrease width to compensate
            if verbose {
                println!(
                    "Decreasing height to maintain aspect ratio. Aspect ratio wanted height: {height}"
                );
            }
            RectSize {
                width,
                height: output_size.height,
            }
        }
        _ => output_size,
    }
}

fn get_clipboard_image() -> Option<ImageData<'static>> {
    if wsl::is_wsl() {
        if let Some(windows_image) = get_clipboard_image_from_wsl() {
            return Some(windows_image);
        }
    }
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

fn get_clipboard_image_from_wsl() -> Option<ImageData<'static>> {
    let error_text = "Failed to transfer Windows clipboard image to WSL!";
    let mut child = Command::new("powershell.exe")
        .arg("-C")
        .arg(include_str!("scripts/image_from_clipboard.ps1"))
        .current_dir(".")
        .output()
        .expect(error_text);

    if !child.status.success() {
        let error_text = std::str::from_utf8(&child.stderr);
        eprintln!("{:#?}", error_text);
        return None;
    }

    if child.stdout.ends_with(&[13, 10]) {
        // Remove line endings after output
        child.stdout.pop();
        child.stdout.pop();
    }

    if child.stdout.is_empty() {
        // No image was found in Windows
        return None;
    }

    let decoded = general_purpose::STANDARD
        .decode(child.stdout)
        .expect("Error decoding contents from Windows clipboard");
    let converted_image = image::load_from_memory(decoded.as_ref()).ok().unwrap();

    Some(ImageData {
        width: converted_image.width() as usize,
        height: converted_image.height() as usize,
        data: Cow::from(converted_image.into_rgba8().into_vec()),
    })
}

fn get_image_from_file(filepath: String) -> Option<ImageData<'static>> {
    let file = File::open(filepath).expect("Could not open file");
    let mut reader = BufReader::new(file);
    let mut bytes: Vec<u8> = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .expect("Error extracting image bytes");
    let image = image::load_from_memory(&bytes).expect("Error decoding image file contents");
    Some(ImageData {
        width: image.width() as usize,
        height: image.height() as usize,
        data: Cow::from(image.into_rgba8().into_vec()),
    })
}
