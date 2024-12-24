use std::io::Cursor;

use image::{ImageBuffer, ImageReader, Rgb};
use rscam::{Camera, IntervalInfo, ResolutionInfo};
use ev3dev_lang_rust::Ev3Result;

const FORMAT: &[u8] = b"MJPG";

fn main() -> Ev3Result<()> {
    let mut cam = Camera::new("/dev/video0").expect("Camera not connected or not supported");

    let resolutions = cam.resolutions(FORMAT).expect("Failed to get available resolutions");

    let resolution = match resolutions {
        ResolutionInfo::Discretes(res) => res.last().unwrap().to_owned(),
        ResolutionInfo::Stepwise { min, .. } => min
    };

    let intervals = cam.intervals(FORMAT, resolution).expect("Failed to get available intervals");    

    let interval = match intervals {
        IntervalInfo::Discretes(res) => res.first().unwrap().to_owned(),
        IntervalInfo::Stepwise { max, ..} => max
    };

    cam.start(&rscam::Config { 
        format: FORMAT,
        resolution: resolution,
        interval: interval,
        ..Default::default() }
    ).expect("Failed to start camera");

    Ok(())
}

fn get_image(cam: &Camera) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let buf = &cam.capture().unwrap()[..];
    
    ImageReader::new(Cursor::new(buf))
        .with_guessed_format().unwrap()
        .decode().unwrap()
        .into_rgb8()
}