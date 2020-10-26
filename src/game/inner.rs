use crate::game::game::NUM_COLS;
use crate::game::game::NUM_ROWS;
use rand::rngs::ThreadRng;
use rand::seq::SliceRandom;
use rand::Rng;
use std::{
	cmp::{max, min},
	collections::VecDeque,
	f64,
	rc::Rc,
};
use tau::TAU;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

pub const FPS: i32 = (0.025 * 1000.0) as i32; // 0.025 sec -> 40 fps
const MIN_SPEED: u32 = 3; // number of frames between updates
const MAX_SPEED: u32 = 1; // number of frames between updates

const MAX_KEY_BUFF_LEN: usize = 3; // how many keys we'll keep track of before ignoring inputs
const COLOR_LINE: &str = "blue"; // how many keys we'll keep track of before ignoring inputs
const COLOR_PYRAMID: &str = "white"; // how many keys we'll keep track of before ignoring inputs
const COLOR_SQUIGGLE: &str = "green"; // how many keys we'll keep track of before ignoring inputs
const COLOR_REVERSE_SQUIGGLE: &str = "red"; // how many keys we'll keep track of before ignoring inputs
const COLOR_SQUARE: &str = "orange"; // how many keys we'll keep track of before ignoring inputs

#[derive(Debug, Clone, Copy)]
struct Vector2D {
	x: i32,
	y: i32,
}

#[derive(Debug)]
struct Square {
	position: Vector2D,
	color: String,
}

#[derive(Debug, Clone, Copy)]
struct FVector2D {
	x: f64,
	y: f64,
}

struct Piece {
	top_left: Vector2D,
	size: i32,
	squares: Vec<Vector2D>, // square offsets from top_left
	color: String,
}

pub struct Inner {
	pub canvas: web_sys::HtmlCanvasElement,
	pub context: Rc<CanvasRenderingContext2d>,

	width: f64,
	height: f64,
	rect_size: f64,

	should_show_focus_banner: bool,
	is_paused: bool,
	is_game_over: bool,
	did_win: bool,
	score: u32,
	key_buff: VecDeque<String>,

	current_piece: Option<Piece>,
	swapped_piece: Option<Piece>,
	board_pieces: Vec<Vec<Square>>,

	frames_between_updates: u32,
	frames_until_update: u32,

	should_send_to_bottom: bool,
	should_swap_piece: bool,

	rotate: i32,
	move_y_delta: i32,
	move_x_delta: i32,

	rng: ThreadRng,
}

impl Inner {
	pub fn new(
		width: f64,
		height: f64,
		rect_size: f64,
		canvas: web_sys::HtmlCanvasElement,
		context: Rc<CanvasRenderingContext2d>,
	) -> Inner {
		let inner = Inner {
			canvas: canvas,
			context: context,

			width: width,
			height: height,
			rect_size: rect_size,

			should_show_focus_banner: false,
			is_paused: false,
			is_game_over: false,
			did_win: false,
			score: 0,
			key_buff: VecDeque::with_capacity(MAX_KEY_BUFF_LEN),

			current_piece: None,
			swapped_piece: None,
			board_pieces: Vec::new(),

			frames_between_updates: MIN_SPEED,
			frames_until_update: 0,

			should_send_to_bottom: false,
			should_swap_piece: false,

			rotate: 0,
			move_y_delta: 0,
			move_x_delta: 0,

			rng: rand::thread_rng(),
		};

		return inner;
	}

	fn reset(&mut self) {
		self.is_game_over = false;
		self.did_win = false;
		self.score = 0;
		self.frames_between_updates = MIN_SPEED;
		self.frames_until_update = MIN_SPEED;
	}

	pub fn focus(&self) -> Result<(), JsValue> {
		self.canvas.focus()
	}

	pub fn show_focus_banner(&mut self) -> Result<(), JsValue> {
		log::info!("Show focus banner");
		self.should_show_focus_banner = true;
		Ok(())
	}

	pub fn hide_focus_banner(&mut self) -> Result<(), JsValue> {
		log::info!("Hide focus banner");
		self.should_show_focus_banner = false;
		Ok(())
	}

	pub fn tick(&mut self) -> Result<(), JsValue> {
		self.pre_process_keys();
		if !self.effectively_paused() {
			if self.frames_until_update == 0 {
				self.process_key();
				self.update()?;
				self.frames_until_update = self.frames_between_updates;
			}
			self.frames_until_update -= 1;
		}
		self.draw().expect("Something's gone wrong with draw");
		Ok(())
	}

	pub fn handle_key(&mut self, key: String) -> Result<(), JsValue> {
		log::info!("Received {}", key);
		if self.key_buff.len() < MAX_KEY_BUFF_LEN {
			self.key_buff.push_back(key);
		}
		Ok(())
	}

	fn effectively_paused(&self) -> bool {
		self.should_show_focus_banner || self.is_paused || self.is_game_over
	}

	pub fn pre_process_keys(&mut self) {
		let mut should_reset = false;
		if let Some(key) = self.key_buff.front() {
			match key.as_str() {
				"r" => {
					log::info!("resetting");
					should_reset = true;
					self.key_buff.pop_front();
				}

				"Enter" => {
					if self.is_game_over {
						should_reset = true;
					} else {
						self.is_paused = !self.is_paused;
					}
					self.key_buff.pop_front();
				}
				_ => {}
			}
		}

		// some things we need to do after our immutable borrows up top
		if should_reset {
			self.reset();
		}

		// eats up any keys that would otherwise clog the buffer.
		// Also prevents pause-buffering
		if self.effectively_paused() {
			self.key_buff.clear();
		}
	}

	pub fn process_key(&mut self) {
		if let Some(key) = self.key_buff.pop_front() {
			if self.effectively_paused() {
				return;
			}

			match key.as_str() {
				// NOTE: y is flipped here since that's the default for rendering, and it's easier
				// to flip it just here than anytime we draw
				"ArrowUp" => self.rotate += 1,
				"ArrowDown" => self.move_y_delta = 1,

				"ArrowRight" => self.move_x_delta = 1,
				"ArrowLeft" => self.move_x_delta = -1,

				// reverse head
				" " => self.should_send_to_bottom = true,
				"s" => self.should_swap_piece = true,

				_ => {}
			}
		}
	}

	fn update(&mut self) -> Result<(), JsValue> {
		if let None = self.current_piece {
			self.current_piece = Some(self.get_random_piece());
		}

		if let Some(current_piece) = &mut self.current_piece {
			let mut y_to_move = 1 + self.move_y_delta;
			self.move_y_delta = 0;

			if self.should_send_to_bottom {
				y_to_move = NUM_ROWS;
				self.should_send_to_bottom = false;
			}

			while y_to_move > 0 {
				y_to_move -= 1;
				current_piece.top_left.y += 1;
				if Inner::does_collide(&current_piece, &self.board_pieces) {
					// undo last move
					current_piece.top_left.y -= 1;
					break;
				}
			}

			let mut x_to_move = self.move_x_delta;
			let x_delta = if x_to_move > 0 { 1 } else { -1 };
			self.move_x_delta = 0;
			while x_to_move != 0 {
				x_to_move -= x_delta;
				current_piece.top_left.x += x_delta;
				log::info!("Moving");
				if Inner::does_collide(&current_piece, &self.board_pieces) {
					log::info!("Actually...");
					current_piece.top_left.x -= x_delta;
					break;
				} else {
					log::info!("Success...");
				}
			}
		}
		// TODO
		Ok(())
	}

	fn does_collide(current_piece: &Piece, board: &Vec<Vec<Square>>) -> bool {
		for square in current_piece.squares.iter() {
			let x = current_piece.top_left.x + square.x;
			let y = current_piece.top_left.y + square.y;

			if x < 0 || x >= NUM_COLS {
				return true;
			}
			if y >= NUM_ROWS {
				return true;
			}

			// TODO: make more efficient
			for board_row in board.iter() {
				for board_piece in board_row.iter() {
					if x == board_piece.position.x && y == board_piece.position.y {
						return false;
					}
				}
			}
		}

		return false;
	}

	fn get_random_piece(&mut self) -> Piece {
		match self.rng.gen_range(0, 5) {
			// match 1 { // TODO: take out
			0 => Piece {
				color: COLOR_LINE.to_string(),
				top_left: Vector2D {
					x: NUM_COLS / 2 - 2,
					y: 0,
				},
				size: 4,
				squares: vec![
					Vector2D { x: 0, y: 0 },
					Vector2D { x: 1, y: 0 },
					Vector2D { x: 2, y: 0 },
					Vector2D { x: 3, y: 0 },
				],
			},

			1 => Piece {
				color: COLOR_PYRAMID.to_string(),
				top_left: Vector2D {
					x: NUM_COLS / 2 - 2,
					y: 0,
				},
				size: 3,
				squares: vec![
					Vector2D { x: 1, y: 0 },
					Vector2D { x: 0, y: 1 },
					Vector2D { x: 1, y: 1 },
					Vector2D { x: 2, y: 1 },
				],
			},

			2 => Piece {
				color: COLOR_SQUIGGLE.to_string(),
				top_left: Vector2D {
					x: NUM_COLS / 2 - 2,
					y: 0,
				},
				size: 3,
				squares: vec![
					Vector2D { x: 1, y: 0 },
					Vector2D { x: 2, y: 0 },
					Vector2D { x: 0, y: 1 },
					Vector2D { x: 1, y: 1 },
				],
			},

			3 => Piece {
				color: COLOR_REVERSE_SQUIGGLE.to_string(),
				top_left: Vector2D {
					x: NUM_COLS / 2 - 2,
					y: 0,
				},
				size: 3,
				squares: vec![
					Vector2D { x: 1, y: 0 },
					Vector2D { x: 0, y: 0 },
					Vector2D { x: 2, y: 1 },
					Vector2D { x: 1, y: 1 },
				],
			},

			4 => Piece {
				color: COLOR_SQUARE.to_string(),
				top_left: Vector2D {
					x: NUM_COLS / 2 - 1,
					y: 0,
				},
				size: 2,
				squares: vec![
					Vector2D { x: 0, y: 0 },
					Vector2D { x: 0, y: 1 },
					Vector2D { x: 1, y: 0 },
					Vector2D { x: 1, y: 1 },
				],
			},

			_ => panic!("Oopsie doodles"),
		}
	}

	pub fn draw(&mut self) -> Result<(), JsValue> {
		let context = &self.context;
		context.clear_rect(0., 0., self.width, self.height);

		if let Some(current_piece) = &self.current_piece {
			for piece in current_piece.squares.iter() {
				let x = current_piece.top_left.x + piece.x;
				let y = current_piece.top_left.y + piece.y;
				self.draw_rect(&Vector2D { x: x, y: y }, &current_piece.color);
			}
		}

		for row in self.board_pieces.iter() {
			for piece in row.iter() {
				self.draw_rect(&piece.position, &piece.color);
			}
		}

		// self.draw_rect(self.path.back().unwrap(), TAIL_COLOR);

		if self.is_paused {
			self.draw_banner("PAUSED");
		} else if self.is_game_over {
			if self.did_win {
				self.draw_banner("YOU WON!!!");
			} else {
				self.draw_banner("GAME OVER");
			}
		} else if self.should_show_focus_banner {
			self.draw_banner("LOST FOCUS");
		}
		Ok(())
	}

	fn draw_line(context: &Rc<CanvasRenderingContext2d>, p1: &FVector2D, p2: &FVector2D) {
		context.begin_path();
		context.move_to(p1.x, p1.y);
		context.line_to(p2.x, p2.y);
		context.stroke();
	}

	fn draw_rect(&self, rect: &Vector2D, color: &str) {
		let context = &self.context;
		context.save();
		context.set_fill_style(&JsValue::from(color));
		context.set_stroke_style(&JsValue::from("black"));
		context.set_line_width(1.);
		context.begin_path();
		context.rect(
			self.rect_size * rect.x as f64,
			self.rect_size * rect.y as f64,
			self.rect_size,
			self.rect_size,
		);
		context.fill();
		context.stroke();
		context.restore();
	}

	// fn draw_circles<'a, I>(&self, circles: I, color: &str)
	// where
	// 	I: Iterator<Item = &'a Vector2D>,
	// {
	// 	let context = &self.context;
	// 	let radius = self.rect_size / 2.;
	// 	let border = 2.;
	// 	context.save();
	// 	context.set_fill_style(&JsValue::from(color));
	// 	context.set_stroke_style(&JsValue::from("black"));
	// 	context.set_line_width(1.);
	// 	for pos in circles {
	// 		context.begin_path();
	// 		context
	// 			.arc(
	// 				self.rect_size * pos.x as f64 + radius,
	// 				self.rect_size * pos.y as f64 + radius,
	// 				radius - border,
	// 				0.,
	// 				TAU,
	// 			)
	// 			.unwrap();
	// 		context.fill();
	// 		context.stroke();
	// 	}
	// 	context.restore();
	// }

	fn draw_banner(&self, text: &str) {
		let context = &self.context;
		context.save();
		context.set_fill_style(&JsValue::from("white"));
		context.set_global_alpha(0.5);
		let quarter_height = self.height / 4.;
		context.fill_rect(
			0.,
			quarter_height,
			self.width,
			self.height - quarter_height * 2.,
		);
		context.restore();

		context.save();
		context.begin_path();
		context.set_font("60px Arial");
		context.set_stroke_style(&JsValue::from("white"));
		context.set_font("60px Arial");
		context.set_text_align("center");
		context.set_text_baseline("middle");
		context.set_fill_style(&JsValue::from("white"));
		context
			.fill_text_with_max_width(text, self.width / 2., self.height / 2., self.width)
			.expect("Something's gone wrong here");
		context.restore();
	}

	// fn get_random_empty_space(&mut self) -> Option<Vector2D> {
	// 	let empty_squares = self.get_empty_squares();
	// 	if let Some(space) = empty_squares.choose(&mut self.rng) {
	// 		return Some(Vector2D {
	// 			x: space.x,
	// 			y: space.y,
	// 		});
	// 	}
	// 	return None;
	// }

	// fn get_empty_squares(&mut self) -> Vec<Vector2D> {
	// 	let mut rv = vec![];
	// 	for x in 0..self.num_squares_x {
	// 		for y in 0..self.num_squares_y {
	// 			if let ICellContents::Empty = self.contents_of_square(x, y) {
	// 				rv.push(Vector2D { x: x, y: y });
	// 			}
	// 		}
	// 	}
	// 	return rv;
	// }

	// fn contents_of_square(&self, x: i32, y: i32) -> ICellContents {
	// 	for pos in self.path.iter() {
	// 		if pos.x == x && pos.y == y {
	// 			return ICellContents::Snake;
	// 		}
	// 	}

	// 	for pos in self.apples.iter() {
	// 		if pos.x == x && pos.y == y {
	// 			return ICellContents::Apple;
	// 		}
	// 	}

	// 	return ICellContents::Empty;
	// }
}
