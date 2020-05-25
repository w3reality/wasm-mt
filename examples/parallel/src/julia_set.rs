// credits -- https://github.com/rustwasm/wasm-bindgen/blob/master/examples/julia_set/src/lib.rs

use std::ops::Add;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use rand::Rng;

pub fn compute(
    width: u32,
    height: u32,
    scale: f64,
    real: f64,
    imaginary: f64,
    as_arraybuffer: bool,
) -> Result<JsValue, JsValue> {
    let c = Complex { real, imaginary };

    let mut data = get_julia_set(width, height, scale, c);

    let data = if as_arraybuffer {
        wasm_mt::utils::u8arr_from_vec(&data).buffer().into()
    } else {
        web_sys::ImageData::new_with_u8_clamped_array_and_sh(Clamped(&mut data), width, height)?.into()
    };

    Ok(data)
}

fn get_julia_set(width: u32, height: u32, scale: f64, c: Complex) -> Vec<u8> {
    let mut data = Vec::new();

    let param_i = 1.5;
    let param_r = 1.5;
    let offset = (rand::thread_rng().gen_range(0.0, 1.0) * 255.0) as u32;

    for x in 0..width {
        for y in 0..height {
            let z = Complex {
                real: y as f64 * scale - param_r,
                imaginary: x as f64 * scale - param_i,
            };
            let iter_index = get_iter_index(z, c);

            data.push((iter_index / 4 + offset) as u8);
            data.push((iter_index / 2 + offset) as u8);
            data.push((iter_index + offset) as u8);
            data.push(255);
        }
    }

    data
}

fn get_iter_index(z: Complex, c: Complex) -> u32 {
    let mut iter_index: u32 = 0;
    let mut z = z;
    while iter_index < 900 {
        if z.norm() > 2.0 {
            break;
        }
        z = z.square() + c;
        iter_index += 1;
    }
    iter_index
}

#[derive(Clone, Copy, Debug)]
struct Complex {
    real: f64,
    imaginary: f64,
}

impl Complex {
    fn square(self) -> Complex {
        let real = (self.real * self.real) - (self.imaginary * self.imaginary);
        let imaginary = 2.0 * self.real * self.imaginary;
        Complex { real, imaginary }
    }

    fn norm(&self) -> f64 {
        (self.real * self.real) + (self.imaginary * self.imaginary)
    }
}

impl Add<Complex> for Complex {
    type Output = Complex;

    fn add(self, rhs: Complex) -> Complex {
        Complex {
            real: self.real + rhs.real,
            imaginary: self.imaginary + rhs.imaginary,
        }
    }
}
