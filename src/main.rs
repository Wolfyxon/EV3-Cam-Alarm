use std::{io::Cursor, thread, time::Duration};
use image::{ImageBuffer, ImageReader, Rgb};
use rscam::{Camera, IntervalInfo, ResolutionInfo};
use ev3dev_lang_rust::{sound, Ev3Result, Led};

const FORMAT: &[u8] = b"MJPG";
const CHANNEL_THRESHOLD: u8 = 64;
const DIFF_THRESHOLD: u32 = 2000;

fn main() -> Ev3Result<()> {
    println!("Initializing...");

    let mut cam = Camera::new("/dev/video0").expect("Camera not connected or not supported");
    let led = Led::new()?;

    let resolutions = cam.resolutions(FORMAT).expect("Failed to get available resolutions");
    println!("Available resolutions: {:?}", resolutions);

    let resolution = match resolutions {
        ResolutionInfo::Discretes(res) => res.first().unwrap().to_owned(),
        ResolutionInfo::Stepwise { min, .. } => min
    };

    let intervals = cam.intervals(FORMAT, resolution).expect("Failed to get available intervals");    
    println!("Available intervals: {:?}", intervals);

    let interval = match intervals {
        IntervalInfo::Discretes(res) => res.first().unwrap().to_owned(),
        IntervalInfo::Stepwise { max, ..} => max
    };

    println!("Using:");
    println!("  Resolution: {:?}", resolution);
    println!("  Interval: {:?}", interval);
    
    cam.start(&rscam::Config { 
        format: FORMAT,
        resolution: resolution,
        interval: interval,
        ..Default::default() }
    ).expect("Failed to start camera");

    println!("Camera started");

    let colors = vec![
        Led::COLOR_YELLOW,
        Led::COLOR_ORANGE,
        Led::COLOR_RED
    ];
    
    println!("Arming in");
    
    for i in 0..colors.len() {
        println!("{}...", colors.len() - i);
        led.set_color(colors[i])?;
        thread::sleep(Duration::from_secs(1));
    }

    led.set_color(Led::COLOR_OFF)?;

    let mut last_img = get_image(&cam);

    println!("Scanning for movement...");

    loop {
        let mut img = get_image(&cam);
        let mut diff: u32 = 0;

        for (pix, last_pix) in img.pixels().zip(last_img.pixels()) {
            let mut detected = false;

            for (&ch, &lch) in pix.0.iter().zip(last_pix.0.iter()) {
                if ch.abs_diff(lch) > CHANNEL_THRESHOLD {
                    diff += 1;

                    if diff > DIFF_THRESHOLD {
                        println!("Motion detected");

                        for i in 0..20 {
                            let dur = 100;

                            if i % 2 == 0 {
                                led.set_color(Led::COLOR_RED)?;
                                sound::tone(800.0, dur)?.wait()?;
                            } else {
                                led.set_color(Led::COLOR_OFF)?;
                                sound::tone(1000.0, dur)?.wait()?;
                            }
                        }

                        detected = true;
                        break;
                    }
                }
            }

            if detected {
                img = get_image(&cam);
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