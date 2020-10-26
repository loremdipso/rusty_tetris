use crate::game::game::{NUM_COLS, NUM_ROWS};
use rand::{rngs::ThreadRng, Rng};
use std::collections::BTreeSet;
use std::convert::TryInto;
use std::{collections::VecDeque, f64, rc::Rc};
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

pub const FPS: i32 = (0.025 * 1000.0) as i32; // 0.025 sec -> 40 fps
const MIN_SPEED: u32 = 5; // number of frames between updates
const MAX_KEY_BUFF_LEN: usize = 3; // how many keys we'll keep track of before ignoring inputs
const FRAMES_BEFORE_WE_SEAL_MOVE: u32 = 8;
const FRAMES_TO_SHOW_PURGATORY: u32 = 2;

const COLOR_LINE: &str = "blue"; // how many keys we'll keep track of before ignoring inputs
const COLOR_PYRAMID: &str = "white"; // how many keys we'll keep track of before ignoring inputs
const COLOR_SQUIGGLE: &str = "green"; // how many keys we'll keep track of before ignoring inputs
const COLOR_REVERSE_SQUIGGLE: &str = "red"; // how many keys we'll keep track of before ignoring inputs
const COLOR_SQUARE: &str = "orange"; // how many keys we'll keep track of before ignoring inputs
const COLOR_PURGATORY: &str = "#242424";

#[derive(Debug, Clone, Copy, Default)]
struct Vector2D {
	x: i32,
	y: i32,
}

#[derive(Debug, Default)]
struct Square {
	position: Vector2D,
	color: String,
	purgatory: bool,
}

#[derive(Debug, Clone, Copy)]
struct FVector2D {
	x: f64,
	y: f64,
}

#[derive(Clone)]
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

	frames_since_last_successful_move: u32,
	frames_to_wait: u32,

	should_send_to_bottom: bool,
	should_swap_piece: bool,

	rotations_to_perform: i32,
	x_to_move: i32,

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

			frames_since_last_successful_move: 0,
			frames_to_wait: 0,

			should_send_to_bottom: false,
			should_swap_piece: false,

			rotations_to_perform: 0,
			x_to_move: 0,

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
		self.current_piece = None;
		self.board_pieces.clear();
		self.frames_to_wait = 0;
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
				"ArrowUp" => self.rotations_to_perform += 1,
				"ArrowDown" => self.rotations_to_perform -= 1,

				"ArrowRight" => self.x_to_move = 1,
				"ArrowLeft" => self.x_to_move = -1,

				// reverse head
				" " => self.should_send_to_bottom = true,
				"s" => self.should_swap_piece = true,

				_ => {}
			}
		}
	}

	fn update(&mut self) -> Result<(), JsValue> {
		if self.frames_to_wait > 0 {
			self.frames_to_wait -= 1;
			return Ok(());
		}

		if self.should_swap_piece {
			self.should_swap_piece = false;
			let previously_swapped_piece = self.swapped_piece.take();
			self.swapped_piece = self.current_piece.take();

			if let Some(mut current_piece) = previously_swapped_piece {
				current_piece.top_left.y = 0;
				self.current_piece = Some(current_piece);
			}
		}

		match &self.current_piece {
			None => {
				self.current_piece = Some(self.get_random_piece());
				self.frames_since_last_successful_move = 0;

				// fix the grid
				// let's loop through the relevant rows, backwards, removing any that are full up

				for row_index in (0..self.board_pieces.len()).rev() {
					if let Some(row) = self.board_pieces.get(row_index) {
						if row.len() == NUM_COLS.try_into().unwrap() {
							self.board_pieces.remove(row_index);

							// shift all rows above down one
							for row_index in row_index..self.board_pieces.len() {
								log::info!("Row index: {}", &row_index);
								if let Some(row) = self.board_pieces.get_mut(row_index) {
									for cell in row.iter_mut() {
										cell.position.y += 1;
									}
								}
							}
						}
					}
				}
			}

			Some(current_piece) => {
				if self.frames_since_last_successful_move > FRAMES_BEFORE_WE_SEAL_MOVE {
					let mut rows_to_check: BTreeSet<usize> = BTreeSet::new();

					// add to board
					for square in current_piece.squares.iter() {
						let x = current_piece.top_left.x + square.x;
						let y = current_piece.top_left.y + square.y;

						let ty = (NUM_ROWS - y).try_into().unwrap(); // TODO: refactor
						rows_to_check.insert(ty);

						// make sure we have enough rows before we push to them
						while self.board_pieces.len() <= ty {
							self.board_pieces.push(vec![]);
						}

						self.board_pieces.get_mut(ty).unwrap().push(Square {
							position: Vector2D { x: x, y: y },
							color: current_piece.color.clone(),
							..Default::default()
						});
					}

					// let's loop through the relevant rows, backwards, removing any that are full up
					let mut should_redraw = false;
					for row_index in rows_to_check.iter().rev() {
						let row = self.board_pieces.get_mut(*row_index).unwrap();
						if row.len() == NUM_COLS.try_into().unwrap() {
							for cell in row.iter_mut() {
								cell.purgatory = true;
								should_redraw = true;
							}
						}
					}

					self.current_piece = None;

					if should_redraw {
						self.frames_to_wait = FRAMES_TO_SHOW_PURGATORY;
					}
					return Ok(());
				}
			}
		};

		if let Some(current_piece) = &mut self.current_piece {
			let mut did_move = false;
			// move down
			{
				let mut y_to_move = 1;
				let mut did_send_to_bottom = false;

				if self.should_send_to_bottom {
					y_to_move = NUM_ROWS;
					self.should_send_to_bottom = false;
					did_send_to_bottom = true;
					self.frames_since_last_successful_move = FRAMES_BEFORE_WE_SEAL_MOVE;
				}

				while y_to_move > 0 {
					y_to_move -= 1;
					current_piece.top_left.y += 1;
					if Inner::does_collide(&current_piece, &self.board_pieces) {
						// undo last move
						current_piece.top_left.y -= 1;
						break;
					} else {
						if !did_send_to_bottom {
							did_move = true;
						}
					}
				}
			}

			// move left/right
			{
				let x_delta = if self.x_to_move > 0 { 1 } else { -1 };
				while self.x_to_move != 0 {
					self.x_to_move -= x_delta;
					current_piece.top_left.x += x_delta;
					if Inner::does_collide(&current_piece, &self.board_pieces) {
						current_piece.top_left.x -= x_delta;
						break;
					} else {
						did_move = true;
					}
				}
			}

			// rotate
			{
				let rotate_delta = if self.rotations_to_perform > 0 { 1 } else { -1 };
				while self.rotations_to_perform != 0 {
					self.rotations_to_perform -= rotate_delta;
					let backup = current_piece.squares.clone();
					if rotate_delta > 0 {
						Inner::rotate_clockwise(current_piece);
					} else {
						Inner::rotate_counter_clockwise(current_piece);
					}
					if Inner::does_collide(&current_piece, &self.board_pieces) {
						// rotating counter-clockwise seemed like a lot of work, so we're just copying memory instead
						current_piece.squares = backup;
					} else {
						did_move = true;
					}
				}
			}

			if did_move {
				self.frames_since_last_successful_move = 0;
			} else {
				self.frames_since_last_successful_move += 1;
			}
		}

		Ok(())
	}

	fn rotate_counter_clockwise(current_piece: &mut Piece) {
		for square in current_piece.squares.iter_mut() {
			// flip about the y-axis
			square.x = current_piece.size - 1 - square.x;

			// translate about the origin
			let temp = square.x;
			square.x = square.y;
			square.y = temp;
		}
	}

	fn rotate_clockwise(current_piece: &mut Piece) {
		for square in current_piece.squares.iter_mut() {
			// flip about the x-axis
			square.y = current_piece.size - 1 - square.y;

			// translate about the origin
			let temp = square.x;
			square.x = square.y;
			square.y = temp;
		}
	}

	fn get_interception_point(current_piece: &Piece, board: &Vec<Vec<Square>>) -> i32 {
		let mut extra_y = 0;
		let mut temp_piece = current_piece.clone(); // clone to get mutable version
		loop {
			temp_piece.top_left.y += 1;
			extra_y += 1;
			if Inner::does_collide(&temp_piece, &board) {
				return extra_y - 1;
			}
		}
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
			for row in board.iter() {
				for board_piece in row.iter() {
					if x == board_piece.position.x && y == board_piece.position.y {
						return true;
					}
				}
			}
		}

		return false;
	}

	fn get_random_piece(&mut self) -> Piece {
		match self.rng.gen_range(0, 5) {
			0 => Piece {
				color: COLOR_LINE.to_string(),
				top_left: Vector2D {
					x: NUM_COLS / 2 - 2,
					y: 0,
				},
				size: 4,
				squares: vec![
					Vector2D { x: 0, y: 1 },
					Vector2D { x: 1, y: 1 },
					Vector2D { x: 2, y: 1 },
					Vector2D { x: 3, y: 1 },
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
					Vector2D { x: 1, y: 1 },
					Vector2D { x: 2, y: 1 },
					Vector2D { x: 0, y: 2 },
					Vector2D { x: 1, y: 2 },
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
					Vector2D { x: 1, y: 1 },
					Vector2D { x: 0, y: 1 },
					Vector2D { x: 2, y: 2 },
					Vector2D { x: 1, y: 2 },
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

		// draw background
		self.start_context("white", "black", 0.6, 3.);
		for x in 0..NUM_COLS {
			for y in 0..NUM_ROWS {
				self.draw_rect(&Vector2D { x: x, y: y });
			}
		}
		self.end_context();

		for row in self.board_pieces.iter() {
			for piece in row.iter() {
				let color = if piece.purgatory {
					COLOR_PURGATORY
				} else {
					&piece.color
				};

				self.start_context(color, "black", 1.0, 3.);
				self.draw_rect(&piece.position);
				self.end_context();
			}
		}

		if let Some(current_piece) = &self.current_piece {
			// draw ghost first in case real piece steps in
			let extra_y = Inner::get_interception_point(&current_piece, &self.board_pieces);

			self.start_context(&current_piece.color, "black", 0.2, 3.);
			for piece in current_piece.squares.iter() {
				let x = current_piece.top_left.x + piece.x;
				let y = current_piece.top_left.y + piece.y + extra_y;
				self.draw_rect(&Vector2D { x: x, y: y });
			}
			self.end_context();

			self.start_context(&current_piece.color, "black", 1.0, 3.);
			for piece in current_piece.squares.iter() {
				let x = current_piece.top_left.x + piece.x;
				let y = current_piece.top_left.y + piece.y;
				self.draw_rect(&Vector2D { x: x, y: y });
			}
			self.end_context();
		}

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

	fn start_context(&self, fill_color: &str, stroke_color: &str, opacity: f64, line_width: f64) {
		let context = &self.context;
		context.save();
		context.set_fill_style(&JsValue::from(fill_color));
		context.set_stroke_style(&JsValue::from(stroke_color));
		context.set_global_alpha(opacity);
		context.set_line_width(line_width);
	}

	fn draw_rect(&self, rect: &Vector2D) {
		&self.context.begin_path();
		&self.context.rect(
			self.rect_size * rect.x as f64,
			self.rect_size * rect.y as f64,
			self.rect_size,
			self.rect_size,
		);
		&self.context.fill();
		&self.context.stroke();
	}

	fn end_context(&self) {
		&self.context.restore();
	}

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
