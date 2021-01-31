use std::collections::HashMap;
use rand::Rng;

use std::ops::Index;
use std::time::Duration;

use druid::widget::prelude::*;
use druid::widget::{Button, Flex, Label, Slider};
use druid::{
    AppLauncher, Color, Data, Lens, LocalizedString, MouseButton, Point, Rect, TimerToken,
    WidgetExt, WindowDesc,
};


const WINDOW_TITLE: LocalizedString<AppData> = LocalizedString::new("Game Of Life");
const GRID_SIZE: u32 = 40;
const POOL_SIZE: u32 = GRID_SIZE * GRID_SIZE;
const BG: Color = Color::grey8(20 as u8);



#[derive(Clone)]
struct ColorScheme {
    colors: Vec<Color>,
    current: usize,
}

impl ColorScheme {
    fn random(num: u32) -> Self {
        let mut rng = rand::thread_rng();
        let mut colors = vec!(Color::from_rgba32_u32(0xEBF1F0));
        for _ in 0..num {
            colors.push(Color::from_rgba32_u32(rng.gen::<u32>()));
        }
        ColorScheme {
            colors: colors,
            current: 0,
        }
    }
}


#[derive(Clone, Data, Lens)]
struct AppData {
    grid: Game,
    drawing: bool,
    removing: bool,
    paused: bool,
    speed: f64,
}

impl AppData {
    pub fn iter_interval(&self) -> u64 {
        (1000. / self.fps()) as u64
    }
    pub fn fps(&self) -> f64 {
        self.speed.max(0.01) * 10.0
    }
}


struct GameWidget {
    timer_id: TimerToken,
    cell_size: Size,
    color_scheme: ColorScheme,
}

impl GameWidget {
    fn grid_pos(&self, p: Point) -> Option<Pos> {
        let w0 = self.cell_size.width;
        let h0 = self.cell_size.height;
        if p.x < 0.0 || p.y < 0.0 || w0 == 0.0 || h0 == 0.0 {
            return None;
        }
        let row = (p.x / w0) as u32;
        let col = (p.y / h0) as u32;
        if row >= GRID_SIZE || col >= GRID_SIZE {
            return None;
        }
        Some(Pos { x: col, y: row})
    }
}

impl Widget<AppData> for GameWidget {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppData, _env: &Env) {
        match event {
            Event::WindowConnected => {
                ctx.request_paint();
                let deadline = Duration::from_millis(data.iter_interval() as u64);
                self.timer_id = ctx.request_timer(deadline);
            }
            Event::Timer(id) => {
                if *id == self.timer_id {
                    if !data.paused {
                        data.grid.next();
                        ctx.request_paint();
                    }
                    let deadline = Duration::from_millis(data.iter_interval() as u64);
                    self.timer_id = ctx.request_timer(deadline);
                }
            }
            Event::MouseDown(e) => {
                if e.button == MouseButton::Left {
                    data.drawing = true;
                    let grid_pos_opt = self.grid_pos(e.pos);
                    grid_pos_opt
                        .iter()
                        .for_each(|pos| data.grid.insert(pos));
                }
                if e.button == MouseButton::Right {
                    data.removing = true;
                    let grid_pos_opt = self.grid_pos(e.pos);
                    grid_pos_opt
                        .iter()
                        .for_each(|pos| data.grid.remove(pos));
                }
            }
            Event::MouseUp(e) => {
                if e.button == MouseButton::Left {
                    data.drawing = false;
                }
                if e.button == MouseButton::Right {
                    data.removing = false;
                }
            }
            Event::MouseMove(e) => {
                if data.drawing {
                    let grid_pos_opt = self.grid_pos(e.pos);
                    grid_pos_opt.iter().for_each(|pos| data.grid.insert(pos));
                }
                if data.removing {
                    let grid_pos_opt = self.grid_pos(e.pos);
                    grid_pos_opt.iter().for_each(|pos| data.grid.remove(pos));
                }
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut LifeCycleCtx,
        _event: &LifeCycle,
        _data: &AppData,
        _env: &Env,
    ) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &AppData, _data: &AppData, _env: &Env) {
        ctx.request_paint();
    }

    fn layout(
        &mut self,
        _layout_ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        _data: &AppData,
        _env: &Env,
    ) -> Size {
        let max_size = bc.max();
        let min_side = max_size.height.min(max_size.width);
        Size {
            width: min_side,
            height: min_side,
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &AppData, _env: &Env) {
        let size: Size = ctx.size();
        let w0 = size.width / GRID_SIZE as f64;
        let h0 = size.height / GRID_SIZE as f64;
        let cell_size = Size {
            width: w0,
            height: h0,
        };
        self.cell_size = cell_size;
        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let pos = Pos {x: col, y: row};

                match data.grid.world.life.get(&pos) {
                    Some(life) => {
                        let point = Point {
                            x: w0 * row as f64,
                            y: h0 * col as f64,
                        };
                        let rect = Rect::from_origin_size(point, cell_size);
                        let color_s = &self.color_scheme;
                        ctx.fill(
                            rect,
                            color_s.colors.index(
                                (life.generation as usize) % color_s.colors.len())
                            );
                    },
                    None => (),
                };
            }
        }
    }
}


fn main() {
    let window = WindowDesc::new(make_widget)
        .window_size(Size {
            width: 800.0,
            height: 800.0,
        })
   //     .resizable(false)
        .title(WINDOW_TITLE);

    let game = Game::random_world(GRID_SIZE, GRID_SIZE, (POOL_SIZE/10) as u64);
    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(AppData {
            grid: game,
            drawing: false,
            removing: false,
            paused: true,
            speed: 0.1,
        })
        .expect("launch failed");
    
}

fn make_widget() -> impl Widget<AppData> {
     Flex::column()
        .with_flex_child(
            GameWidget {
                timer_id: TimerToken::INVALID,
                cell_size: Size {
                    width: 0.0,
                    height: 0.0,
                },
                color_scheme: ColorScheme::random(100),
            },
            1.0,
        )
        .with_child(
            Flex::column()
                .with_child(
                    Flex::row()
                        .with_flex_child(
                            // Generation label
                            Label::dynamic(|data: &AppData, _| format!("Generation: {}",
                                                                       data.grid.world.id)),
                            1.0,
                        )
                        .with_flex_child(
                            // Oldest label
                            Label::dynamic(|data: &AppData, _| format!("Oldest: {}",
                                            match data.grid.world.find_oldest() {
                                                Some(life) => data.grid.world.id - life.generation,
                                                None => 0,
                                            })),
                            1.0,
                        )
                        .with_flex_child(
                            // Alive label
                            Label::dynamic(|data: &AppData, _| format!("Alive: {}", data.grid.world.life.keys().len())),
                            1.0,
                        )
                        .with_flex_child(
                            // died label
                            Label::dynamic(|data: &AppData, _| format!("Dead: {}", data.grid.life_counter - data.grid.world.life.keys().len() as u64)),
                            1.0,
                        )
                        .padding(8.0),
                )
                .with_child(
                    // a row with two buttons
                    Flex::row()
                        .with_flex_child(
                            // pause / resume button
                            Button::new(|data: &bool, _: &Env| match data {
                                true => "Resume".into(),
                                false => "Pause".into(),
                            })
                            .on_click(|ctx, data: &mut bool, _: &Env| {
                                *data = !*data;
                                ctx.request_layout();
                            })
                            .lens(AppData::paused)
                            .padding((5., 5.)),
                            1.0,
                        )
                        .with_flex_child(
                            // clear button
                            Button::new("Clear")
                                .on_click(|ctx, data: &mut Game, _: &Env| {
                                    data.clear();
                                    ctx.request_paint();
                                })
                                .lens(AppData::grid)
                                .padding((5., 5.)),
                            1.0,
                        )
                        .padding(8.0),
                )
                .with_child(
                    Flex::row()
                        .with_child(
                            Label::new(|data: &AppData, _env: &_| format!("{:.2}FPS", data.fps()))
                                .padding(3.0),
                        )
                        .with_flex_child(Slider::new().expand_width().lens(AppData::speed), 1.)
                        .padding(8.0),
                )
                .background(BG),
        )
}


#[derive(Eq, PartialEq, Debug, Clone, Data)]
struct Game {
    life_counter: u64,
    #[data(same_fn = "PartialEq::eq")]
    world: World,
}

impl Game {
    fn random_world(width: u32, hight: u32, num_life: u64) -> Game {
        let mut rng = rand::thread_rng();
        
        
        let mut world = World{
            id: 0,
            width: width,
            hight: hight,
            life: HashMap::new(),
        };
        

        for _ in 0..num_life {
            loop {
                let x: u32 = rng.gen_range(0..world.width);
                let y: u32 = rng.gen_range(0..world.hight);
                let pos = Pos{x:x, y:y};
                if !world.life.contains_key(&pos) {
                    let life = Life{
                        generation: 0,
                        pos: pos.clone(),
                    };
                    world.life.insert(pos, life);
                    break;
                }
            }
        }
        
        let game = Game {
            life_counter: num_life,
            world: world,
        };

        return game;
    }

    fn next(&mut self) {
        let old_world = &self.world;
        let mut world = World{
            id: old_world.id + 1,
            width: old_world.width,
            hight: old_world.hight,
            life: HashMap::new(),
        };

        for y in 0..world.hight {
            for x in 0..world.width {
                let pos = Pos{x: x, y: y};
                let num_n = old_world.num_neighbours(&pos);

                if old_world.life.contains_key(&pos) {
                    if num_n > 0 && num_n < 4 {
                        world.life.insert(pos.clone(), old_world.life.get(&pos).unwrap().clone());
                    }
                } else {
                    if num_n > 1 && num_n < 4 {
                        let life = Life{
                            generation: world.id,
                            pos: pos.clone(),
                        };
                        self.life_counter += 1;
                        world.life.insert(pos.clone(), life);
                    }
                }
            }
        }

        self.world = world;
    }


    fn clear(&mut self) {
        self.life_counter = 0;
        self.world = World{
            id: 0,
            width: GRID_SIZE,
            hight: GRID_SIZE,
            life: HashMap::new(),
        };
    }

    fn insert(&mut self, pos: &Pos) {
        if !self.world.life.contains_key(pos) {
            let life = Life{
                generation: self.world.id,
                pos: pos.clone(),
            };
            self.life_counter += 1;
            self.world.life.insert(pos.clone(), life);
        } 
    }

    fn remove(&mut self, pos: &Pos) {
        if self.world.life.contains_key(pos) {
            self.world.life.remove(pos);
        } 
    }

//    fn change(&mut self, pos: &Pos) {
//        if !self.world.life.contains_key(pos) {
//            let life = Life{
//                generation: self.world.id,
//                pos: pos.clone(),
//            };
//            self.life_counter += 1;
//            self.world.life.insert(pos.clone(), life);
//        } else {
//            self.world.life.remove(pos);
//        }
//    }
}

#[derive(Eq, PartialEq, Debug, Clone, Data)]
struct World {
    id: u64,
    width: u32,
    hight: u32,
    #[data(same_fn = "PartialEq::eq")]
    life: HashMap<Pos, Life>,
}


impl World {
    fn left_neighbour(&self, pos: &Pos) -> bool {
        let left;
        if pos.x == 0 {
            left = Pos{x: self.width - 1, y: pos.y};
        } else {
            left = Pos{x: pos.x - 1, y: pos.y};
        }
        
        return self.life.contains_key(&left);
    }
    
    fn right_neighbour(&self, pos: &Pos) -> bool {
        let right;
        if pos.x == self.width - 1 {
            right = Pos{x: 0, y: pos.y};
        } else {
            right = Pos{x: pos.x + 1, y: pos.y};
        }
        
        return self.life.contains_key(&right);
    }
    
    fn top_neighbour(&self, pos: &Pos) -> bool {
        let top;
        if pos.y == 0 {
            top = Pos{x: pos.x, y: self.hight - 1};
        } else {
            top = Pos{x: pos.x, y: pos.y - 1};
        }
        
        return self.life.contains_key(&top);
    }
    
    fn bottom_neighbour(&self, pos: &Pos) -> bool {
        let bottom;
        if self.hight - 1 == pos.y {
            bottom = Pos{x: pos.x, y: 0};
        } else {
            bottom = Pos{x: pos.x, y: pos.y + 1};
        }
 
        return self.life.contains_key(&bottom);
    }
    
    fn top_left_neighbour(&self, pos: &Pos) -> bool {
        let x; 
        if pos.x == 0 {x = self.width - 1} else {x = pos.x - 1}
        let y;
        if pos.y == 0 {y = self.hight - 1} else {y = pos.y - 1}
        return self.life.contains_key(&Pos{x:x, y: y});
    }

    fn top_right_neighbour(&self, pos: &Pos) -> bool {
        let x; 
        if pos.x == self.width - 1 {x = 0} else {x = pos.x + 1}
        let y;
        if pos.y == 0 {y = self.hight - 1} else {y = pos.y - 1}
        return self.life.contains_key(&Pos{x:x, y: y});
    }
    
    fn bottom_left_neighbour(&self, pos: &Pos) -> bool {
        let x; 
        if pos.x == 0 {x = self.width - 1} else {x = pos.x - 1}
        let y;
        if pos.y == self.hight - 1 {y = 0} else {y = pos.y + 1}
        return self.life.contains_key(&Pos{x:x, y: y});
    }

    fn bottom_right_neighbour(&self, pos: &Pos) -> bool {
        let x; 
        if pos.x == self.width - 1 {x = 0} else {x = pos.x + 1}
        let y;
        if pos.y == self.hight - 1 {y = 0} else {y = pos.y + 1}
        return self.life.contains_key(&Pos{x:x, y: y});
    }

    fn num_neighbours(&self, pos: &Pos) -> u32 {
        let mut num = 0;
        
        if self.left_neighbour(pos) { num += 1;}
        if self.right_neighbour(pos) { num += 1;}
        if self.top_neighbour(pos) { num += 1;}
        if self.bottom_neighbour(pos) { num += 1;}
        if self.top_left_neighbour(pos) { num += 1;}
        if self.top_right_neighbour(pos) { num += 1;}
        if self.bottom_left_neighbour(pos) { num += 1;}
        if self.bottom_right_neighbour(pos) { num += 1;}

        return num;
    }

    fn find_oldest(&self) -> Option<&Life> {
        self.life.values().min_by(|a, b| a.generation.cmp(&b.generation))
    }
}


#[derive(Eq, PartialEq, Debug, Clone, Data)]
struct Life {
    generation: u64,
    pos: Pos,
}


#[derive(Hash, Eq, PartialEq, Debug, Clone, Data)]
struct Pos {
    x: u32,
    y: u32,
}

