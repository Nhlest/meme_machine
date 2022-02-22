use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::mem::take;
use image::{Delay, Frame, GenericImage, ImageDecoder};
use pango::*;
use pangocairo::cairo::{Format};
use image::AnimationDecoder;
use image::gif::Repeat;

fn main() {
  let args = std::env::args().collect::<Vec<_>>();
  if args.len() != 3 {
    eprintln!("Usage: \n\t {} meme.gif \"Caption\"", args[0]);
    std::process::exit(-1);
  }

  let gif = args[1].as_str();
  let text = args[2].as_str();

  let f = File::open(gif).unwrap();
  let i = image::codecs::gif::GifDecoder::new(BufReader::new(f)).unwrap();
  let dim = i.dimensions();
  let font_desc = pango::FontDescription::from_string(format!("Futura Extra Black Condensed {}", (dim.0 as f32 / 13.0).floor() as u32).as_str());
  let image_surface = pangocairo::cairo::ImageSurface::create(Format::ARgb32, dim.0 as i32, 10000 as i32).unwrap();
  let pixel_size;
  {
    let cairo_ctx: pangocairo::cairo::Context = pangocairo::cairo::Context::new(&image_surface).unwrap();
    let pango_ctx = pangocairo::create_context(&cairo_ctx).unwrap();
    pango_ctx.set_font_description(&font_desc);
    let layout = pango::Layout::new(&pango_ctx);

    cairo_ctx.set_source_rgba(1.0, 1.0, 1.0, 1.0);
    cairo_ctx.paint().unwrap();
    cairo_ctx.set_source_rgba(0.0, 0.0, 0.0, 1.0);

    cairo_ctx.move_to(0.0, 10.0);

    let scale = pangocairo::pango::SCALE;

    layout.set_font_description(Some(&font_desc));
    layout.set_width(dim.0 as i32 * scale);
    layout.set_wrap(WrapMode::Word);
    layout.set_alignment(Alignment::Center);
    layout.set_markup(format!("<span font_family=\"futura\" weight=\"bold\">{}</span>", text).as_str());
    pixel_size = layout.pixel_size().1;

    pangocairo::update_layout(&cairo_ctx, &layout);
    pangocairo::show_layout(&cairo_ctx, &layout);
  }
  let data = image_surface.take_data().unwrap();

  let mut data = data.to_vec();
  data.truncate((dim.0 * 4 * (pixel_size as u32 + dim.1 + 20) as u32) as usize);

  let mut image = image::RgbaImage::from_raw(dim.0 as u32, pixel_size as u32 + dim.1 + 20, data).unwrap();

  let mut encoder = image::codecs::gif::GifEncoder::new(BufWriter::new(File::create("output_gif.gif").unwrap()));
  encoder.set_repeat(Repeat::Infinite);

  encoder.encode_frames(
    i.into_frames().map(|frame| {
      let mut o_frame = Frame::from_parts(image.clone(), 0, 0, frame.as_ref().unwrap().delay().clone());
      o_frame.buffer_mut().copy_from(frame.unwrap().buffer(), 0, (pixel_size + 20) as u32).unwrap();
      o_frame
    })
  ).unwrap();
}
