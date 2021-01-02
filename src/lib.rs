use std::{cell::RefCell, rc::Rc};

use sys::{system::{Nes}};
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, ImageData, console::log_1};

pub mod sys;

fn log(s: &String) {
    unsafe{
        log_1(&JsValue::from(s));
    }
}

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

enum AppState{
    UNINITIALIZED,
    READY,
    RUN
}

struct StateTest{
    system: Option<Nes>,
    app_state: AppState
}

impl StateTest {
    pub fn new() -> StateTest {
        StateTest{system: None, app_state: AppState::UNINITIALIZED}
    }

    pub fn set_system(&mut self, sys: Nes){
        self.system = Some(sys);
    }

    pub fn set_state(&mut self, value: AppState){
        self.app_state = value;
    }
}

static mut testtest: Option<StateTest> = None;

#[wasm_bindgen(start)]
pub fn run() -> Result<(), JsValue>  {
    unsafe{
        testtest = Some(StateTest::new());
    }
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut i = 0;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        if i > 5000 {

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
    
        draw(&context, i);

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());
    Ok(())
}

#[wasm_bindgen]
pub fn draw(ctx: &CanvasRenderingContext2d, step: i32) -> Result<(), JsValue> {
    // let width = 256;
    // let height = 240;
    // let mut data = get_image_data_demo();
    // let data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut data), width as u32, height as u32)?;
    // ctx.put_image_data(&data, 0.0, 0.0);
    ctx.set_fill_style(&JsValue::from("#000000"));
    // let mut a:u8 = 0x27;
    // a |=  1 << (0x01 as u8);
    // log(&format!("{}", a));
    unsafe {
        let test = & testtest;

        &mut testtest.as_mut().map(|state|{
            let sys = &mut state.system;
            match state.app_state {
                AppState::READY => {
                    (*sys).as_mut().map(|sys|{
                        sys.reset();
                    });
                    state.set_state(AppState::RUN)
                },
                AppState::RUN => {
                    (*sys).as_mut().map(|sys|{
                        for i in 0..10{ // TODO: タイミング・サイクル数換算は後で実装
                            sys.execute();
                        }
                        log(&format!("{:04x} {:02x} {:02x} {:02x} {:02x}", sys.cpu.program_counter, sys.cpu.reg_a, sys.cpu.reg_x, sys.cpu.reg_y, sys.cpu.reg_p));
                    });
                },
                _ => {}
            };
        });

        let mut buf_tmp = (*test).as_ref().map(|state|{
            let sys = &state.system;
            let buf_ = (*sys).as_ref().map(|sys|{
                let buf = sys.frame_buffer.to_vec();
                buf
            });
            match buf_ {
                Some(b) => b,
                None => get_image_data_demo().to_vec()
            }
        });
        let mut buf = match buf_tmp {
            Some(b) => b,
            None => get_image_data_demo().to_vec()
        };
        let width = 256;
        let height = 240;
        let data = ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut buf), width as u32, height as u32)?;
        // log(&format!("{:?}", buf));
        ctx.put_image_data(&data, 0.0, 0.0);

        let size = (*test).as_ref().map(|state|{
            let sys = &state.system;
            let size_ = (*sys).as_ref().map(|sys|{
                sys.memory_map.rom.prg_rom.len()
            });
            match size_ {
                Some(s) => s,
                None => 0
            }
        });
        let size = match size {
            Some(s) => s,
            None => 0
        };
        ctx.fill_text(&size.to_string(), 40.0, 245.0);
    }
    ctx.fill_text(&step.to_string(), 10.0, 245.0)
}

#[wasm_bindgen]
pub fn set_rom(buf: &mut [u8]) -> String {
    // TODO
    // dbg!(buf);
    let rom = load_cartridge(buf);
    let str = format!("prg_rom {}bytes\nchr_rom {}bytes\n", rom.prg_rom.len(), rom.chr_rom.len());


    unsafe{
        let mut option = &mut testtest;
        let mut test1 = (*option).as_mut().unwrap();
        test1.set_state(AppState::UNINITIALIZED);
        test1.set_system(Nes::new(rom));
        test1.set_state(AppState::READY);
    }
    // sys::system::Nes::new(rom);

    str
}

pub fn load_cartridge(buf: &[u8]) -> sys::rom::Rom{
    let cartridge = sys::rom::from_array(buf);
    cartridge
}

fn get_image_data_demo() -> Vec<u8> {
    let mut data = Vec::new();

    for x in 0..240 {
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

#[cfg(test)]
mod tests {
    use std::{fs, io};
    use std::fs::File;
    use std::io::Read;
    use regex::Regex;

    use crate::sys::{self, system::Nes};
    
    #[test]
    fn nestest() {
        let buf = fs::read("./nestest.nes").expect("Unable to read file");
        let log_file = fs::read_to_string("./nestest.log").expect("Unable to read file");
        let log_line = log_file.lines();
        let rom = sys::rom::from_array(&buf);
        let mut sys = Nes::new(rom);
        sys.reset();
        sys.cpu.program_counter = 0xC000;
        sys.cpu.reg_p = 0x24;

        for (index, line) in log_line.enumerate() {
            let line_split1: Vec<&str> = line.split_whitespace().collect();
            let pc_expect = line_split1[0].to_string();
            
            let regex_a = Regex::new(r"(A:..)").unwrap();
            let caps = regex_a.captures(line).unwrap();
            let reg_a_expect = caps.get(0).unwrap().as_str().to_string();

            let regex_p = Regex::new(r"(P:..)").unwrap();
            let caps = regex_p.captures(line).unwrap();
            let reg_p_expect = caps.get(0).unwrap().as_str().to_string();

            let expect = format!("{} {} {}", pc_expect, reg_a_expect, reg_p_expect);
            let actual = format!("{:04X} A:{:02X} P:{:02X}", sys.cpu.program_counter, sys.cpu.reg_a, sys.cpu.reg_p);
            assert_eq!(expect, actual);
            if index > 3350{
                break;
            }
            sys.execute();
        }
    }
}