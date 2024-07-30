use tinybmp::Bmp;
use wavers::write;
use embedded_graphics::{pixelcolor::Rgb888, prelude::*};

fn main() {
    let mut args = std::env::args().skip(1);
    let src = args.next().expect("no src specified");
    let dst = args.next().expect("no dst specified");
    let src = std::fs::read(src).expect("failed to read src");
    let src: Bmp<'_, Rgb888> = Bmp::from_slice(&src).expect("failed parsing bmp");
    let mut samples = vec![0i16; (src.size().width * src.size().height) as usize];
    for px in src.pixels() {
        let pos = px.0;
        samples[(pos.y * src.size().width as i32 + pos.x) as usize] = i16::from_le_bytes([px.1.r(), px.1.g()]);
    }
    write(dst, &samples, 48000, 2).expect("failed to write samples");
}
