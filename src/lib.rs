use std::{cell::RefCell, rc::Rc};

use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, ImageData};

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue>  {
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut i = 0;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        if i > 300 {

            // Drop our handle to this closure so that it will get cleaned
            // up once we return.
            let _ = f.borrow_mut().take();
            return;
        }

        // Set the body's text content to how many times this
        // requestAnimationFrame callback has fired.
        i += 1;
        let text = format!("requestAnimationFrame has been called {} times.", i);

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

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

#[wasm_bindgen]
pub fn draw(ctx: &CanvasRenderingContext2d) -> Result<(), JsValue> {
    let width = 256;
    let height = 256;
    let mut data = get_image_data();
    let data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut data), width as u32, height as u32)?;
    ctx.put_image_data(&data, 0.0, 0.0)
}

#[wasm_bindgen]
pub fn set_rom(x: &mut [u8]) -> String {
    // TODO
    dbg!(x);
    "test".to_string()
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
