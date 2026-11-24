//! Catch the Ball: a ball bounces around the board; click it as many times as you can in 30
//! seconds. It shows the three things every game is made of:
//!
//! - **State** - what the game remembers: the `MainWindow` struct below (where the ball is, how
//!   fast it goes, the score, the time left).
//! - **Events** - what the player does: a click on Start, a click on the board.
//! - **Time** - two Timers: `mover` ticks 50 times a second and moves the ball a little each
//!   time; `clock` ticks once a second and counts down.
//!
//! The ball is not a control: the board is a panel the program draws itself (its Paint event,
//! `board_paint`). Each tick changes the state, then asks the board to draw again (`invalidate`).

#![allow(unused_variables, unused_imports)] // an event's method need not use all it is given (ui, e), nor the file all the prelude brings

use crate::main_window_ui::{MainWindowEvents, MainWindowUi};
use phai_ui::prelude::*;

/// How long a game lasts, in seconds.
const GAME_SECONDS: u32 = 30;

/// Colours, in CSS: the board, its lines, the ball and its shine.
const BOARD: &str = "#f0f7f4";
const LINES: &str = "#dcebe5";
const BALL: &str = "#e8590c";
const SHINE: &str = "#ffa94d";

#[derive(Default)]
pub struct MainWindow {
    /// Is a game being played now?
    playing: bool,
    /// The ball's centre on the board, in pixels, and its radius.
    x: f32,
    y: f32,
    radius: f32,
    /// How far the ball moves each tick, across and down. A minus number goes left or up.
    dx: f32,
    dy: f32,
    score: u32,
    misses: u32,
    seconds_left: u32,
    /// The last game's score, to say "Time's up" over the board; None before the first game.
    last_score: Option<u32>,
    /// A number that changes every time `random` is called - see `random`.
    seed: u32,
}

impl MainWindow {
    /// A number from 0.0 up to (not including) 1.0 that looks random: each call multiplies and
    /// adds big numbers to the last one and keeps what overflows away. (Real games often use the
    /// `rand` crate; this is enough here and needs nothing else.)
    fn random(&mut self) -> f32 {
        self.seed = self.seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        (self.seed >> 8) as f32 / (1u32 << 24) as f32
    }

    /// Puts the ball somewhere new on the board and sends it off in a new direction.
    fn new_ball(&mut self, ui: &MainWindowUi) {
        let (w, h) = ui.board.client_size();
        let (w, h) = (w as f32, h as f32);
        // The ball's size and speed follow the board's size, so the game plays the same on a
        // small window and a big screen.
        self.radius = w.min(h) / 14.0;
        let speed = w.min(h) / 90.0 * (1.0 + self.score as f32 * 0.08); // faster with every catch
        self.x = self.radius + self.random() * (w - 2.0 * self.radius);
        self.y = self.radius + self.random() * (h - 2.0 * self.radius);
        self.dx = if self.random() < 0.5 { speed } else { -speed };
        self.dy = if self.random() < 0.5 { speed * 0.7 } else { -speed * 0.7 };
    }

    /// Shows the score, the misses and the time left in the labels at the top.
    fn show_numbers(&self, ui: &MainWindowUi) {
        ui.score.set_text(&self.score.to_string());
        ui.misses.set_text(&self.misses.to_string());
        ui.time_left.set_text(&self.seconds_left.to_string());
    }

    fn start_game(&mut self, ui: &MainWindowUi) {
        self.playing = true;
        self.score = 0;
        self.misses = 0;
        self.seconds_left = GAME_SECONDS;
        self.seed = now().millisecond + now().second * 1000; // a different game each time
        self.new_ball(ui);
        self.show_numbers(ui);
        ui.mover.start(); // the ball moves...
        ui.clock.start(); // ...and the seconds count down
        ui.start_stop.set_text(t("Stop"));
        ui.status.set_part_text(0, t("Click the ball!"));
        ui.board.invalidate();
    }

    fn stop_game(&mut self, ui: &MainWindowUi) {
        self.playing = false;
        ui.mover.stop();
        ui.clock.stop();
        self.last_score = Some(self.score);
        ui.start_stop.set_text(t("Start"));
        ui.status.set_part_text(0, &tf("Game over: {0} caught, {1} missed. Press Start to play again.", &[&self.score, &self.misses]));
        ui.board.invalidate();
    }
}

/// A filled circle, drawn as thin strips one pixel high - each strip as wide as the circle is at
/// that height (Pythagoras: half the width is the square root of r² - dy²).
fn fill_circle(e: &PaintEventArgs, cx: f32, cy: f32, r: f32, colour: &str) {
    let r_whole = r as i32;
    for dy in -r_whole..=r_whole {
        let half = (r * r - (dy * dy) as f32).max(0.0).sqrt();
        e.fill_rect((cx - half) as i32, cy as i32 + dy, (half * 2.0) as i32 + 1, 1, colour);
    }
}

impl MainWindowEvents for MainWindow {
    /// The window is on the screen, the board at its size. With PHAI_SHOT set (Phai Studio
    /// taking a picture of the program) a game starts at once, so the picture shows the ball.
    fn shown(&mut self, ui: &MainWindowUi) {
        if std::env::var_os("PHAI_SHOT").is_some() {
            self.start_game(ui);
        }
    }

    fn start_stop_clicked(&mut self, ui: &MainWindowUi) {
        if self.playing {
            self.stop_game(ui);
        } else {
            self.start_game(ui);
        }
    }

    /// The `mover` timer, every 20 ms: move the ball one step and bounce it off the edges.
    fn mover_tick(&mut self, ui: &MainWindowUi) {
        let (w, h) = ui.board.client_size();
        let (w, h) = (w as f32, h as f32);
        self.x += self.dx;
        self.y += self.dy;
        // Past the left or right edge: turn round, across. The same for top and bottom.
        if self.x - self.radius < 0.0 {
            self.x = self.radius;
            self.dx = self.dx.abs();
        }
        if self.x + self.radius > w {
            self.x = w - self.radius;
            self.dx = -self.dx.abs();
        }
        if self.y - self.radius < 0.0 {
            self.y = self.radius;
            self.dy = self.dy.abs();
        }
        if self.y + self.radius > h {
            self.y = h - self.radius;
            self.dy = -self.dy.abs();
        }
        ui.board.invalidate(); // draw the board again: board_paint runs
    }

    /// The `clock` timer, once a second: one second less; at 0 the game is over.
    fn clock_tick(&mut self, ui: &MainWindowUi) {
        self.seconds_left = self.seconds_left.saturating_sub(1);
        self.show_numbers(ui);
        if self.seconds_left == 0 {
            self.stop_game(ui);
        }
    }

    /// A mouse button pressed on the board: was it on the ball?
    fn board_mouse_down(&mut self, ui: &MainWindowUi, e: &MouseEventArgs) {
        if !self.playing {
            return;
        }
        // On the ball when the click is closer to its centre than its radius (Pythagoras again).
        let (ax, ay) = (e.x as f32 - self.x, e.y as f32 - self.y);
        if ax * ax + ay * ay <= self.radius * self.radius {
            self.score += 1;
            ui.status.set_part_text(0, &tf("Caught it! {0} so far.", &[&self.score]));
            self.new_ball(ui); // a new ball somewhere else, a little faster
        } else {
            self.misses += 1;
            ui.status.set_part_text(0, t("Missed - try again!"));
        }
        self.show_numbers(ui);
        ui.board.invalidate();
    }

    /// The board is drawn: its background, then the ball - or a message when no game is on.
    fn board_paint(&mut self, ui: &MainWindowUi, e: &PaintEventArgs) {
        e.clear(BOARD);
        // Faint squares, like a playing field.
        let step = (e.width.min(e.height) / 8).max(10);
        for x in (step..e.width).step_by(step as usize) {
            e.draw_line(x, 0, x, e.height, LINES);
        }
        for y in (step..e.height).step_by(step as usize) {
            e.draw_line(0, y, e.width, y, LINES);
        }

        if self.playing {
            fill_circle(e, self.x, self.y, self.radius, BALL);
            // A small light spot up and to the left makes it look round.
            fill_circle(e, self.x - self.radius * 0.35, self.y - self.radius * 0.35, self.radius * 0.25, SHINE);
        } else if let Some(score) = self.last_score {
            e.draw_text_centered(&tf("Game over! You caught the ball {0} times.\n\nPress Start to play again.", &[&score]), "#0f5132");
        } else {
            e.draw_text_centered(t("Click the ball as many times as you can in 30 seconds.\n\nPress Start to begin."), "#0f5132");
        }
    }
}
