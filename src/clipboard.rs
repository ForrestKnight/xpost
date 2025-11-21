use anyhow::{Context, Result};
use arboard::Clipboard;
use arboard::ImageData;

pub fn get_image_from_clipboard() -> Result<Vec<u8>> {
    let mut clipboard = Clipboard::new()
        .context("Failed to access clipboard")?;
    
    let img = clipboard.get_image()
        .context("No image in clipboard. Try Ctrl+U to upload from file instead (clipboard may not work on Wayland)")?;
    
    let png_data = rgba_to_png(&img)?;
    
    Ok(png_data)
}

fn rgba_to_png(img: &ImageData) -> Result<Vec<u8>> {
    use image::{ImageBuffer, RgbaImage};
    use std::io::Cursor;
    
    let width = img.width as u32;
    let height = img.height as u32;
    
    let img_buffer: RgbaImage = ImageBuffer::from_raw(width, height, img.bytes.to_vec())
        .context("Failed to create image buffer from clipboard data")?;
    
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    
    img_buffer.write_to(&mut cursor, image::ImageFormat::Png)
        .context("Failed to encode image as PNG")?;
    
    Ok(png_bytes)
}

pub fn validate_image_file(path: &str) -> Result<Vec<u8>> {
    use std::io::Cursor;
    
    let img = image::open(path)
        .context("Failed to open image file")?;
    
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    
    img.write_to(&mut cursor, image::ImageFormat::Png)
        .context("Failed to encode image as PNG")?;
    
    Ok(png_bytes)
}
