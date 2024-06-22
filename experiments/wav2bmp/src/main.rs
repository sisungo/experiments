use std::panic::AssertUnwindSafe;
use bmp::{Image, Pixel};
use rand::Rng;
use wavers::Wav;

fn main() {
    let mut args = std::env::args().skip(1);
    let src = args.next().expect("no src specified");
    let dst = args.next().expect("no dst specified");
    let size = args.next().expect("no size specified");
    let size = {
        let mut splited = size.split('x');
        let width = splited.next().expect("bad size format");
        let height = splited.next().expect("bad size format");
        let width: u32 = width.parse().expect("bad size format");
        let height: u32 = height.parse().expect("bad size format");
        (width, height)
    };
    let mut src: Wav<i16> = Wav::from_path(src).expect("read wave error");
    let mut bmp = Image::new(size.0, size.1);
    let samples = src.read().expect("bad wave file");
    for (n, sample) in samples.iter().copied().enumerate() {
        let sample = sample.to_le_bytes();
        let n = n as u32;
        if std::panic::catch_unwind(AssertUnwindSafe(|| {
            bmp.set_pixel(n % size.0, n / size.0, Pixel::new(sample[0], sample[1], rand::thread_rng().gen()));
        })).is_err() {
            break;
        }
    }
    bmp.save(dst).expect("save bmp failed");

}
