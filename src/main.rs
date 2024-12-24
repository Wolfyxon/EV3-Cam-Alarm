use std::io::Cursor;
use image::{ImageBuffer, ImageReader, Rgb};
use rscam::{Camera, IntervalInfo, ResolutionInfo};
use ev3dev_lang_rust::Ev3Result;

const FORMAT: &[u8] = b"MJPG";
const CHANNEL_THRESHOLD: u8 = 64;
const DIFF_THRESHOLD: u32 = 1024;

fn main() -> Ev3Result<()> {
    println!("Initializing...");

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

    println!("Camera started");

    let mut last_img = get_image(&cam);

    println!("Scanning for movement...");

    loop {
        let img = get_image(&cam);
        let mut diff: u32 = 0;

        let pixels: Vec<&Rgb<u8>> = img.pixels().collect();
        let last_pixels: Vec<&Rgb<u8>> = last_img.pixels().collect();

        for i in 0..pixels.len() {
            let mut detected = false;
            let pix = pixels[i];
            let last_pix = last_pixels[i];

            for chi in 0..pix.0.len() {
                let ch = pix.0[chi];
                let lch = last_pix.0[chi];
                
                if ch.abs_diff(lch) > CHANNEL_THRESHOLD {
                    diff += 1;

                    if diff > DIFF_THRESHOLD {
                        println!("Motion detected {}", diff);

                        detected = true;
                        break;
                    }
                }
            }

            if detected {
                break;
            }
        }

        last_img = img;
    }
}

fn get_image(cam: &Camera) -> ImageBuffer<Rgb<u8>, Vec<u8>> {
    let buf = &cam.capture().unwrap()[..];
    
    ImageReader::new(Cursor::new(buf))
        .with_guessed_format().unwrap()
        .decode().unwrap()
        .into_rgb8()
}