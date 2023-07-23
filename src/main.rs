use arboard::Clipboard;
use ascii_image::scale;

fn main() {
    let mut clipboard = Clipboard::new().unwrap();
    let image = match clipboard.get_image() {
        Ok(img) => img,
        Err(e) => {
            eprintln!("error getting image: {}", e);
            return;
        }
    };

    let scaled_image = scale(&image, 200, 200);
    println!("{}", scaled_image);
}
