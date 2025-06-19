use std::num;
use std::time::Duration;
use anathema::component::{Children, Color, Component, Context};
use anathema::default_widgets::Canvas;
use anathema::geometry::LocalPos;
use anathema::runtime::Builder;
use anathema::state::{State, Value};
use anathema::widgets::{Attributes, Style};
use rand::prelude::Distribution;
use crate::ExtendedWidget;

#[derive(Default)]
pub struct Starfield {
    stars: Vec<Star>,
}

impl Starfield {
    pub(crate) fn initialise_stars(&mut self, width: i32, height: i32, no_of_stars: u16) {
        self.stars = (0..no_of_stars).map(|_| {
            let mut star= Star::new(width, height);
            star.counter_x = width / 2;
            star.counter_y = height / 2;
            star
        }).collect();
    }

    pub(crate) fn update_stars(&mut self, state: &mut StarfieldState, canvas: &mut Canvas, dt: Duration, width: i32, height: i32) {
        for star in self.stars.iter_mut() {
            star.draw(canvas, width, height, ' '); // Erase the star by drawing a space
        }

        for star in self.stars.as_mut_slice() {
            star.update(state, dt, width, height);
            star.draw(canvas, width, height, '*');
        }
    }
}

#[derive(State, Default)]
pub struct StarfieldState {
    pub x_start: Value<i32>,
    pub y_start: Value<i32>,
    pub x_pos: Value<f32>,
    pub y_pos: Value<f32>,
    pub x_ratio: Value<f32>,
    pub y_ratio: Value<f32>,
    pub counter_x: Value<i32>,
    pub counter_y: Value<i32>,
    pub radius: Value<f32>,
}

struct Star {
    x_start: i32,
    y_start: i32,
    pos_x: f32,
    pos_y: f32,
    radius: StarSize,
    max_radius: StarSize,
    speed: i32,
    counter_x: i32,
    counter_y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StarSize {
    Small,
    Medium,
    Large,
    ExtraLarge,
}

impl Distribution<StarSize> for rand::distr::StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> StarSize {
        match rng.random_range(0..4) {
            0 => StarSize::Small,
            1 => StarSize::Medium,
            2 => StarSize::Large,
            3 => StarSize::ExtraLarge,
            _ => unreachable!(),
        }
    }
}

impl From<StarSize> for i32 {
    fn from(size: StarSize) -> Self {
        match size {
            StarSize::Small => 0,
            StarSize::Medium => 1,
            StarSize::Large => 2,
            StarSize::ExtraLarge => 3,
        }
    }
}

impl From<i32> for StarSize {
    fn from(value: i32) -> Self {
        match value {
            0 => StarSize::Small,
            1 => StarSize::Medium,
            2 => StarSize::Large,
            3 => StarSize::ExtraLarge,
            _ => unreachable!("Invalid star size value: {}", value),
        }
    }
}

impl Star {
    pub fn new(width: i32, height: i32) -> Self {
        let center_x = width / 2;
        let center_y = height / 2;
        let x_start = Self::get_random(-center_x, center_x);
        let y_start = Self::get_random(-center_y, center_y);

        Star {
            x_start,
            y_start,
            pos_x: x_start as f32,
            pos_y: y_start as f32,
            radius: StarSize::Small,
            max_radius: StarSize::ExtraLarge, //rand::random(),
            speed: rand::random_range(1..3),
            counter_x: rand::random_range(0..width / 2),
            counter_y: rand::random_range(0..height / 2),
        }
    }

    fn get_random(start: i32, end: i32) -> i32 {
        let range = rand::random_range(start..end);
        range
    }

    fn update(&mut self, state: &mut StarfieldState, dt: Duration, width: i32, height: i32) {
        self.counter_x -= self.speed; // as u16 * dt.as_secs() as u16;
        self.counter_y -= self.speed; // as u16 * dt.as_secs() as u16;
        state.counter_x.set(self.counter_x);
        state.counter_y.set(self.counter_y);
        
        if self.counter_x < 1 || self.counter_y < 1 {
            *self = Star::new(width, height);
        }
        
        state.x_start.set(self.x_start);
        state.y_start.set(self.y_start);

        let x_ratio = self.x_start as f32 / self.counter_x as f32;
        let y_ratio = self.y_start as f32 / self.counter_y as f32;
        state.x_ratio.set(x_ratio);
        state.y_ratio.set(y_ratio);


        let star_x = Self::remap(x_ratio as f32, 0.0, 1.0, 0.0, width as f32 / 2.0);
        let star_y = Self::remap(y_ratio as f32, 0.0, 1.0, 0.0, height as f32 / 2.0);
        self.pos_x = star_x;
        self.pos_y = star_y;
        state.x_pos.set(self.pos_x);
        state.y_pos.set(self.pos_y);

        // let max_radius: i32 = self.max_radius.clone().into();
        let x = Self::remap(self.counter_x as f32, 0.0, width as f32 / 2.0, 0.0, 1.0);
        let y = Self::remap(self.counter_y as f32, 0.0, height as f32 / 2.0, 0.0, 1.0);
        let y = if x < y {
            x
        } else {
            y
        };
        
        state.radius.set(x);
        match y {
            0.0..0.25 => {
                self.radius = StarSize::ExtraLarge;
            },
            0.25..0.50 => {
                self.radius = StarSize::Large;
            },
            0.50..0.75 => {
                self.radius = StarSize::Medium;
            },
            _ => {
                self.radius = StarSize::Small;
            },
        };

        // check if the star is out of bounds
        if self.pos_x < (-width / 2) as f32 || self.pos_x > (width / 2) as f32 ||
           self.pos_y < ((-height) / 2) as f32 || self.pos_y > (height / 2) as f32 {
            *self = Star::new(width, height);
        }
    }

    fn remap(value: f32, i_start: f32, i_stop: f32, o_start: f32, o_stop: f32) -> f32 {
        (o_start + (o_stop - o_start) * ((value - i_start) / (i_stop - i_start)))
    }

    fn draw(&self, canvas: &mut Canvas, width: i32, height: i32, character: char) {
        // Draw the star on the canvas
        let x = (self.pos_x + (width / 2) as f32) as u16;
        let y = (self.pos_y + (height / 2) as f32) as u16;

        match self.radius {
            StarSize::Small => {
                let style = Style::reset();
                // let style = Style {
                //     fg: Some(Color::Red),
                //     bg: None,
                //     attributes: Attributes::empty(),
                // };
                canvas.put(character, style, LocalPos { x, y, });
            },
            StarSize::Medium => {
                let style = Style::reset();
                // let style = Style {
                //     fg: Some(Color::Green),
                //     bg: None,
                //     attributes: Attributes::empty(),
                // };
                
                canvas.put(character, style, LocalPos { x, y, });
                if x + 1 < width as u16 {
                    canvas.put(character, style, LocalPos { x: x + 1, y, });
                }
            },
            StarSize::Large => {
                let style = Style::reset();
                // let style = Style {
                //     fg: Some(Color::Blue),
                //     bg: None,
                //     attributes: Attributes::empty(),
                // };
                
                canvas.put(character, style, LocalPos { x, y, });
                if x + 1 < width as u16 {
                    canvas.put(character, style, LocalPos { x: x + 1, y, });
                }
                if x > 0 {
                    canvas.put(character, style, LocalPos { x: x - 1, y, });
                }
            },
            StarSize::ExtraLarge => {
                let style = Style::reset();
                // let style = Style {
                //     fg: Some(Color::White),
                //     bg: None,
                //     attributes: Attributes::empty(),
                // };
                canvas.put(character, style, LocalPos { x, y, });
                if x + 1 < width as u16 {
                    canvas.put(character, style, LocalPos { x: x + 1, y, });
                }
                if x != 0 {
                    canvas.put(character, style, LocalPos { x: x - 1, y, });
                }
                if y + 1 < height as u16 {
                    canvas.put(character, style, LocalPos { x, y: y + 1, });
                }
                if y > 0 {
                    canvas.put(character, style, LocalPos { x, y: y - 1, });
                }
            },
        }
    }
}

impl Component for Starfield {
    type State = StarfieldState;
    type Message = ();

    fn on_init(&mut self, state: &mut Self::State, children: Children<'_, '_>, context: Context<'_, '_, Self::State>) {
        let width = context.attributes.get_as::<i32>("width")
            .unwrap_or(50);
        let height = context.attributes.get_as::<i32>("height")
            .unwrap_or(50);
        let no_of_stars = context.attributes.get_as::<u16>("stars")
            .unwrap_or(50);

        self.initialise_stars(width, height, no_of_stars);
    }

    fn on_tick(&mut self, state: &mut Self::State, mut children: Children<'_, '_>, context: Context<'_, '_, Self::State>, dt: Duration) {
        let width = context.attributes.get_as::<i32>("width")
            .unwrap_or(50);
        let height = context.attributes.get_as::<i32>("height")
            .unwrap_or(50);

        children.elements().by_tag("canvas")
            .first(|el, _| {
                let canvas = el.to::<Canvas>();
                self.update_stars(state, canvas, dt, width, height);
            });
    }
}

impl ExtendedWidget for Starfield {
    fn register(builder: &mut Builder<()>) {
        builder.component("starfield", "templates/starfield.aml", Starfield::default(), StarfieldState::default())
            .expect("Failed to register starfield component");
    }
}