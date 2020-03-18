extern crate oxipng;
extern crate serde_json;

use image::GenericImageView;
use oxipng::{optimize_from_memory, Options};
use std::collections::HashSet;
// use std::time::{Duration, SystemTime, UNIX_EPOCH};
use wasm_bindgen::prelude::*;

#[macro_use]
pub mod log;

#[macro_use]
extern crate serde_derive;

#[wasm_bindgen]
extern "C" {
  fn alert(s: &str);
  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);
}

#[wasm_bindgen]
pub fn handle_effect(image_buffer: &[u8], effect: &str) -> Option<Box<[u8]>> {
  match effect {
    "monochrome" => Some(monochrome(image_buffer)),
    "half-monochrome" => {
      // here, use a dialogbox that mentions that we dont support that yet
      // alert("Oops! We dont support that yet. Please try again sooon 😉");
      console_log!("Oops! This is a work in progress");
      Some(half_monochrome(image_buffer))
    }
    "sepia" => {
      let eff = sepia(image_buffer);
      console_log!("This is a work in progress!!");
      Some(eff)
    }
    "crop" => {
      console_log!("This is a WIP. If there is anything wrong, pls open an issue on the repi");
      None
    }
    _ => {
      console_log!("Oops! Unknowm format");
      None
    }
  }
}

#[wasm_bindgen]
pub fn monochrome(image_buff: &[u8]) -> Box<[u8]> {
  // TODO: remove rayon jpeg feature to support wasm jpeg decode
  let img = image::load_from_memory(image_buff).unwrap().grayscale();
  let mut wr = Vec::new();
  img
    .write_to(&mut wr, image::ImageOutputFormat::PNG)
    .unwrap();

  wr.into_boxed_slice()
}

#[wasm_bindgen]
pub fn sepia(image_buff: &[u8]) -> Box<[u8]> {
  let img = image::load_from_memory(image_buff).unwrap().to_rgba();
  let (width, height) = img.dimensions();

  let mut output_img = img.clone();
  for x in 0..width {
    for y in 0..height {
      // this is a rather naive implementation. but works now
      // 💡 an idea would be to use ```map_with_alpha so the effect is applied to all channels on image
      // without necessarily doing most of below. BUT THIS WORKS NOW!!!🎊🎉
      let pixel = img.get_pixel(x, y);
      let mut pixel_cp = *pixel;
      let r = (0.393 * pixel[0] as f64) + (0.769 * pixel[1] as f64) + (0.189 * pixel[0] as f64);
      let g = (0.349 * pixel[0] as f64) + (0.686 * pixel[1] as f64) + (0.168 * pixel[0] as f64);
      let b = (0.272 * pixel[0] as f64) + (0.53 * pixel[1] as f64) + (0.131 * pixel[0] as f64);

      if r > 255.0 {
        pixel_cp[0] = 255;
      } else {
        pixel_cp[0] = r as u8;
      }

      if g > 255.0 {
        pixel_cp[1] = 255
      } else {
        pixel_cp[1] = g as u8;
      }

      if b > 255.0 {
        pixel_cp[2] = 255
      } else {
        pixel_cp[2] = b as u8;
      }

      pixel_cp[3] = pixel[3];
      output_img.put_pixel(x, y, pixel_cp);
    }
  }

  let mut out_writer: Vec<u8> = Vec::new();
  let md = image::DynamicImage::ImageRgba8(output_img);
  md.write_to(&mut out_writer, image::ImageOutputFormat::PNG)
    .unwrap();
  out_writer.into_boxed_slice()
}

#[wasm_bindgen]
pub fn half_monochrome(image_buffer: &[u8]) -> Box<[u8]> {
  let img = image::load_from_memory(image_buffer).unwrap().to_rgba();
  let (width, height) = img.dimensions();
  let mut output_img = image::ImageBuffer::new(width, height);

  for x in 0..width {
    for y in 0..height {
      let pixel = img.get_pixel(x, y);
      let mut pixel_cp = *pixel;

      if x >= width / 2 {
        let r = (pixel_cp[0] as f64 * 0.92126) as f64;
        let g = (pixel_cp[1] as f64 * 0.97152) as f64;
        let b = (pixel_cp[2] as f64 * 0.90722) as f64;

        let grey = ((r + g + b) / 3.0) as u8;
        pixel_cp[0] = grey;
        pixel_cp[1] = grey;
        pixel_cp[2] = grey;
        output_img.put_pixel(x, y, pixel_cp);
      } else {
        output_img.put_pixel(x, y, *pixel);
      }
    }
  }
  let mut out_writer: Vec<u8> = Vec::new();
  let md = image::DynamicImage::ImageRgba8(output_img);
  md.write_to(&mut out_writer, image::ImageOutputFormat::PNG)
    .unwrap();

  out_writer.into_boxed_slice()
}

#[wasm_bindgen]
pub fn crop_image(image_buffer: &[u8]) -> Box<[u8]> {
  let mut img = image::load_from_memory(image_buffer).unwrap();
  let (width, height) = img.dimensions();
  // then crop image
  let crop = img.crop(100, 110, width, height);
  let mut out_writer: Vec<u8> = Vec::new();
  crop
    .write_to(&mut out_writer, image::ImageOutputFormat::PNG)
    .unwrap();
  out_writer.into_boxed_slice()
}

#[wasm_bindgen]
pub fn rotate(image_buffer: &[u8], degree: i16) -> Option<Box<[u8]>> {
  let img: Option<image::DynamicImage> = match degree {
    // we want to reset when the user selects 0
    90 => Some(image::load_from_memory(image_buffer).unwrap().rotate90()),
    180 => Some(image::load_from_memory(image_buffer).unwrap().rotate180()),
    270 => Some(image::load_from_memory(image_buffer).unwrap().rotate270()),
    360 => Some(image::load_from_memory(image_buffer).unwrap()),
    _ => {
      console_log!("Cannot rotate to the degree {:?}", degree);
      None
    }
  };
  let mut out_writer = Vec::new();
  img
    .unwrap()
    .write_to(&mut out_writer, image::ImageOutputFormat::PNG)
    .unwrap();
  Some(out_writer.into_boxed_slice())
}

#[derive(Serialize, Deserialize, Debug)]
struct FOption {
  backup: bool,
  filter: u8,
  compression: u8,
  verbose: u8,
}

#[wasm_bindgen]
pub fn compress_image(image_buffer: &[u8], compression_options: &JsValue) -> Option<Box<[u8]>> {
  let img = image::load_from_memory(image_buffer).unwrap();
  let mut output_img = Vec::new();

  img
    .write_to(&mut output_img, image::ImageFormat::PNG)
    .unwrap();

  let (width, height) = img.dimensions();
  console_log!("Width and heigth of the image is {} and {}", width, height);
  // let mut img_out = image::ImageBuffer::new(width, height);
  let opts: FOption = compression_options.into_serde().unwrap();
  console_log!("Passed options is: {:?}", opts);
  let mut filter_hashset = HashSet::new();
  filter_hashset.insert(opts.filter);

  let mut compression_hashset = HashSet::new();
  compression_hashset.insert(opts.compression);

  let options = Options {
    backup: false,
    filter: filter_hashset,
    compression: compression_hashset,
    verbosity: Some(opts.verbose),
    ..Default::default()
  };

  let optimize_img = optimize_from_memory(image_buffer, &options)
    .unwrap()
    .into_boxed_slice();
  Some(optimize_img)
}

// #[wasm_bindgen(start)]
// pub fn run_perf() {
//   let window = web_sys::window().expect("should have a window in this context");
//   let performance = window
//     .performance()
//     .expect("performance should be available");

//   console_log!("the current time (in ms) is {}", performance.now());

//   let start = perf_to_system(performance.timing().request_start());
//   let end = perf_to_system(performance.timing().response_end());

//   console_log!("request started at {}", humantime::format_rfc3339(start));
//   console_log!("request ended at {}", humantime::format_rfc3339(end));
// }

// fn perf_to_system(amt: f64) -> SystemTime {
//   let secs = (amt as u64) / 1_000;
//   let nanos = ((amt as u32) % 1_000) * 1_000_000;
//   UNIX_EPOCH + Duration::new(secs, nanos)
// }

// #[wasm_bindgen]
// pub fn get_time_taken(start: f64, end: f64) -> Option<(f64, f64)> {
//   let window = web_sys::window().expect("ERROR. WINDOW CONTEXT NOT FOUND");
//   let performance = window
//     .performance()
//     .expect("ERROR. PERFORMANCE NOT IN SCOPE");

//   let start = perf_to_sys(performance.timing().request_start());
//   let end = perf_to_sys(performance.timing().response_end());
//   console_log!("STARTED AT: {:?}", humantime::format_rfc3339(start));
//   console_log!("ENDED AT {:?}", humantime::format_rfc3339(end));

// }

// #[wasm_bindgen]
// pub fn program_start() -> Option<f64> {
//   let window = web_sys::window().expect("Cannot Find window in context");
//   let performance = window
//     .performance()
//     .expect("ERROR. PERFORMANCE NOT FOUND IN CONTEXT");
//   let start = perf_to_sys(performance.timing().request_start());
//   let to_mili = start
//     .duration_since(UNIX_EPOCH)
//     .expect("Time is in the past.")
//     .as_secs_f64();

//   return Some(to_mili);
// }

// #[wasm_bindgen]
// pub fn program_end() -> Option<f64> {
//   let window = web_sys::window().expect("Cannot Find window in context");
//   let performance = window
//     .performance()
//     .expect("Error. Could not find performance in context");
//   let end = perf_to_sys(performance.timing().response_end());
//   let to_mili = end
//     .duration_since(UNIX_EPOCH)
//     .expect("TIME IS IN THE PAST")
//     .as_secs_f64();

//   return Some(to_mili);
// }

// fn perf_to_sys(tm: f64) -> SystemTime {
//   let sec = (tm as u64) / 1_000;
//   let nano = ((tm as u32) % 1_000) * 1_000_000;
//   UNIX_EPOCH + Duration::new(sec, nano)
// }
