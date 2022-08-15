extern crate quantizr;
extern crate png;

use std::env;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

fn load_image(path: String) -> Result<(Vec<u8>, usize, usize), Box<dyn std::error::Error>> {
    let decoder = png::Decoder::new(File::open(path)?);
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;
    // let bytes = &buf[..info.buffer_size()];

    let mut bytes = vec![];
    bytes.extend_from_slice(&buf[..info.buffer_size()]);

    return Ok((bytes, info.width as usize, info.height as usize))
}

fn save_image(path: String, palette: &quantizr::Palette, indexes: &Vec<u8>,
    width: usize, height: usize) -> Result<(), Box<dyn std::error::Error>> {

    let mut rgb_palette = Vec::with_capacity((palette.count * 3) as usize);
    let mut trans = Vec::with_capacity(palette.count as usize);

    for e in palette.entries.iter() {
        rgb_palette.push(e.r);
        rgb_palette.push(e.g);
        rgb_palette.push(e.b);
        trans.push(e.a);
    }

    let file = File::create(Path::new(&path))?;
    let ref mut w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width as u32, height as u32);
    encoder.set_color(png::ColorType::Indexed);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_palette(rgb_palette);
    encoder.set_trns(trans);
    let mut writer = encoder.write_header()?;

    writer.write_image_data(&indexes)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 4 {
        println!("Usage: quantizr_demo <colors> <src_path> <dst_path>");
        std::process::exit(1)
    }

    let colors = args[1].parse::<i32>()?;
    let src_path = args[2].clone();
    let dst_path = args[3].clone();

    let (bytes, width, height) = load_image(src_path)?;

    let image = quantizr::Image::new(bytes.as_slice(), width, height)?;

    let mut opts = quantizr::Options::default();
    opts.set_max_colors(colors)?;

    let mut result = quantizr::QuantizeResult::quantize(&image, &opts);
    result.set_dithering_level(1.0)?;

    let mut indexes = vec![0u8; width*height];

    result.remap_image(&image, indexes.as_mut_slice())?;

    let palette = result.get_palette();

    save_image(dst_path, palette, &indexes, width, height)
}
