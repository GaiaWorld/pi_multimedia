use std::mem::transmute;

use pi_slotmap::{SecondaryMap, DefaultKey};
use wasm_bindgen::JsCast;
use crate::{font::font::{FontId, Font, FontImage, Block, Await, DrawBlock}, measureText};
use web_sys::{window, CanvasRenderingContext2d, HtmlCanvasElement};

use super::{fillBackGround, setFont, drawCharWithStroke, drawChar, getGlobalMetricsHeight};

pub struct Brush {
	faces: SecondaryMap<DefaultKey, Font>,
	canvas: HtmlCanvasElement,
	ctx: CanvasRenderingContext2d,
}
impl Brush {
	pub fn new() -> Self {
		let window = window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let canvas = document.create_element("canvas").expect("create canvas fail");
        let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().expect("create canvas fail");
        let ctx = canvas
            .get_context("2d")
            .expect("")
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()
            .expect("create canvas fail");
		Brush {
			faces: SecondaryMap::default(),
			canvas,
			ctx
		}
	}

	pub fn check_or_create_face(&mut self, font_id: FontId, font: &Font) {
		self.faces.insert((*font_id).clone(), font.clone());
	}

	pub fn height(&mut self, font_id: FontId) -> f32 {
		let face = &mut self.faces[*font_id];
		getGlobalMetricsHeight(face.font_family.get_hash() as u32, face.font_size as f32) as f32
	}

    pub fn width(&mut self, font_id: FontId, char: char) -> f32 {
		let face = match self.faces.get_mut(*font_id) {
			Some(r) => r,
			None => return 0.0,
		};
		let ch_code: u32 = unsafe { transmute(char) };
		measureText(&self.ctx, ch_code, face.font_size as u32, face.font_family.get_hash() as u32)
    }

    pub fn draw<F: FnMut(Block, FontImage) + Clone + Send + Sync + 'static>(
		&mut self, 
		draw_list: Vec<DrawBlock>,
		mut update: F) {
		
		for draw_block in draw_list.into_iter() {
			let face = match self.faces.get_mut(*draw_block.font_id) {
				Some(r) => r,
				None => return ,
			};
			// 绘制
			// face.set_pixel_sizes(draw_block.font_size as u32);
			// face.set_stroker_width(*draw_block.font_stroke as f64);

			draw_sync(
				draw_block.chars, 
				&draw_block.block,
				face,
				*draw_block.font_stroke as f64,
				&self.canvas,
				&self.ctx
			);
			let (width, height) = (draw_block.block.width, draw_block.block.height);
			match self.ctx.get_image_data(0.0, 0.0, width as f64, height as f64) {
				Ok(r) => {
					update(draw_block.block, FontImage {buffer: (*r.data()).clone(), width: width as usize, height: height as usize});
				},
				Err(e) => log::error!("get_image_data fail, {:?}", e),
			}
		}
	}
}

fn draw_sync(list: Vec<Await>, block: &Block, font: &Font, stroke: f64, canvas: &HtmlCanvasElement, ctx: &CanvasRenderingContext2d) {
	fillBackGround(canvas, ctx, block.width as u32, block.height as u32);
	setFont(
		ctx,
		font.font_weight as u32,
		font.font_size as u32,
		font.font_family.get_hash() as u32,
		stroke as u8,
	);
	if stroke > 0.0 {
		for await_item in list.iter() {
			let ch_code: u32 = unsafe { transmute(await_item.char) };
			let x = (await_item.x_pos + stroke as f32/2.0) as u32;
			//fillText 和 strokeText 的顺序对最终效果会有影响， 为了与css text-stroke保持一致， 应该fillText在前
			drawCharWithStroke(ctx, ch_code, x, 0);
		}
	} else {
		for await_item in list.iter() {
			let ch_code: u32 = unsafe { transmute(await_item.char) };
			drawChar(ctx, ch_code, await_item.x_pos as u32, 0);
		}
	}
}

