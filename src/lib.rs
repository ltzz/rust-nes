use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, ImageData};

#[wasm_bindgen(start)]
pub fn start() {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    draw(&context);
}

#[wasm_bindgen]
pub fn draw(ctx: &CanvasRenderingContext2d) -> Result<(), JsValue> {
    let width = 256;
    let height = 256;
    let mut data = get_image_data();
    let data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut data), width as u32, height as u32)?;
    ctx.put_image_data(&data, 0.0, 0.0)
}

fn get_image_data() -> Vec<u8> {
    let mut data = Vec::new();

    for x in 0..256 {
        for y in 0..256 {
            if x + y < 128 {
                data.push((255) as u8);
                data.push((127) as u8);
                data.push((127) as u8);
            }
            else {
                data.push((255) as u8);
                data.push((255) as u8);
                data.push((255) as u8);
            }
            data.push(255);
        }
    }
    data
}
