extern crate std;
extern crate piston;
extern crate opengl_graphics;
extern crate graphics;
#[cfg(feature = "include_sdl2")]
extern crate sdl2_window;
#[cfg(feature = "include_glfw")]
extern crate glfw_window;
#[cfg(feature = "include_glutin")]
extern crate glutin_window;

#[cfg(feature = "include_sdl2")]
use self::sdl2_window::Sdl2Window as Window;
#[cfg(feature = "include_glfw")]
use self::glfw_window::GlfwWindow as Window;
#[cfg(feature = "include_glutin")]
use self::glutin_window::GlutinWindow as Window;

use self::piston::input::*;
use self::piston::event_loop::*;

use display::mouse::Mouse;
use display::draw;

use board::GoBoard;
use board::Team;
use board::Tile;

use ia::Decision;
use ia::heuristic;

pub const GO_WIDTH : usize = 19;
pub const CASE_WIDTH: graphics::types::Resolution = 40;

#[derive(Debug, PartialEq, Clone)]
pub enum Player {
    Human,
    Ia,
}

impl Player {
	pub fn from_str(s: &str) -> Player {
	    match s {
	        "ia"	=> Player::Ia,
	        "human"	=> Player::Human,
	        _		=> panic!("Player cli option must be either ia, solo or multi")
	    }
	}
}

const ORANGE: graphics::types::Color = [0.97647065f32, 0.9450981f32, 0.854902f32, 1f32];

#[derive(Debug, Clone)]
pub struct Console {
    board: GoBoard,
    event: Mouse,
    player: (Team, Player),
    friend: (Team, Player),
    layer: u32,
    turn: bool, // Player one = true, player two = false.
    win: bool,
    help: bool,
}

impl Console {

	/// The `new` constructor function returns the interface console.

    pub fn new (
        board: GoBoard,
        layer: u32,
        (player, friend): (Player, Player),
        help: bool,
    ) -> Self {
        let size: u32 = board.get_size() as u32;
        let (team_player, team_friend) = Team::new_teams();

		Console {
			board: board,
            event: Mouse::new((CASE_WIDTH * size, CASE_WIDTH * size)),
            player: (team_player, player),
            friend: (team_friend, friend),
            turn: true,
            layer: layer,
            win: false,
            help: help,
		}
    }

    fn get_size (
        &self
    ) -> piston::window::Size {
        let size: graphics::types::Resolution = self.board.get_size (
        ) as graphics::types::Resolution;
        let dimension = self.event.get_dimension();

        piston::window::Size::from([
            dimension.0 / size,
            dimension.1 / size,
        ])
    }

    fn set_raw (&mut self, (x, y): (u32, u32), team: &Team) -> (u32, u32) {
        self.board.set_raw (
            (x as usize, y as usize),
            team.get_tile()
        );
        self.turn = !self.turn;
        (x, y)
    }

    fn set (&mut self, event: &Event, team: &mut Team) -> (u32, u32) {
        let (x, y) = self.event.get_coordinate();

        if x < self.board.get_size() as u32 && y < self.board.get_size() as u32 {
            if let Some(Button::Mouse(_)) = event.press_args() {
                if self.board.set((x as usize, y as usize), team) {
                    self.turn = !self.turn;
                }
            };
        }
        (x, y)
    }

    fn play (&mut self, event: &Event) -> Option<Tile> {
        let (x, y):(u32, u32) = match (self.turn, self.player.clone(), self.friend.clone()) {
            (true, (ref player, Player::Ia), (friend, _)) => {
                let (x, y) = Decision::get_optimal_move (
                    &mut self.board,
                    &(*player, friend),
                    friend,
                    self.layer,
                    heuristic
                );
                self.set_raw((x as u32, y as u32), player)
            },
            (false, (player, _), (ref friend, Player::Ia)) => {
                let (x, y) = Decision::get_optimal_move (
                    &mut self.board,
                    &(player, *friend),
                    *friend,
                    self.layer,
                    heuristic
                );
                self.set_raw((x as u32, y as u32), friend)
            },
            (true, (_, Player::Human), (_, _)) => {
                let mut team = self.player.0;

                self.set(event, &mut team)
            },
            (false, (_, _), (_, Player::Human)) => {
                let mut team = self.friend.0;

                self.set(event, &mut team)
            },
        };
        self.board.is_win(x as usize, y as usize)
    }

    pub fn start (
        &mut self,
    ) {
        let opengl = opengl_graphics::OpenGL::V3_2;
        let window: Window = piston::window::WindowSettings::new (
            "Gomoku",
            self.event.get_dimension(),
        ).exit_on_esc(true).opengl(opengl).build().unwrap();
        let window = std::rc::Rc::new(std::cell::RefCell::new(window));
        let ref mut gl = opengl_graphics::GlGraphics::new(opengl);
        let limit: u32 = self.board.get_size() as u32;

        for event in window.clone().events() {
            let dimension = self.get_size();

            if let Some(resize) = event.resize(|w, h| (w as u32, h as u32)) {
                self.event.set_dimension(resize);
            }
            if self.win == false {
                if let Some(coordinate) = event.mouse_cursor(|x, y| {
                    (x as u32, y as u32)
                }) {
                    if let Some(coordinate) = self.event.check_inside_window (
                        coordinate,
                        limit,
                    ) {
                        self.event.set_coordinate(coordinate);
                    }
                }
                if let Some(team) = self.play(&event) {
                    println!("{} win! Give a cookie to him!", team);
                    self.win = true;
                }
            }
            if let Some(args) = event.render_args() {
                gl.draw(args.viewport(), |context, g| {
                    graphics::clear(ORANGE, g);
                    draw::draw_render(&self.board, dimension, limit, (&context, g));
                    if self.help {
                        draw::draw_help(&self.board, dimension, (0, 0), (&context, g));
                    }
                    if self.event.get_over() {
                        draw::draw_over(&self.board, dimension, self.event.get_coordinate(), (&context, g));
                    }
                });
            }
            event.update(|_| {});
        }
    }
}

impl Default for Console {

	/// The `new` constructor function returns the interface console.

    fn default () -> Self {
        let board: GoBoard = Default::default();
        let (team_player, team_friend) = Team::new_teams();
        let size: u32 = board.get_size() as u32;

		Console {
			board: board,
            event: Mouse::new((CASE_WIDTH * size, CASE_WIDTH * size)),
            player: (team_player, Player::Human),
            friend: (team_friend, Player::Ia),
            layer: 3,
            turn: true,
            win: false,
            help: false,
		}
    }
}
