use crate::error::Error;
use axum::body::Bytes;
use image::{ImageReader, imageops};
use std::io::Cursor;

/// Recompresses an image to the specified resolution in the AVIF format.
pub fn recompress(data: Bytes, expected_resolution: (u32, u32)) -> Result<Bytes, Error> {
    let mut buffer = Cursor::new(Vec::with_capacity(data.len()));

    ImageReader::new(Cursor::new(data))
        .with_guessed_format()
        .map_err(Error::bad_request)?
        .decode()
        .map_err(Error::bad_request)?
        .resize(
            expected_resolution.0,
            expected_resolution.1,
            imageops::FilterType::Gaussian,
        )
        .write_to(&mut buffer, image::ImageFormat::Avif)
        .map_err(Error::internal)?;

    Ok(Bytes::from(buffer.into_inner()))
}
