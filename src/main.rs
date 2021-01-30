use std::collections::HashMap;
use rand::Rng;


fn main() {
    let mut game = Game::random_world(20, 10, 20);
    let mut i = 0;
    while !game.no_change() && i < 100 {
        game.next();
        i += 1;
    }
    game.print();
    game.print_stats();
}


struct Game {
    life_counter: u64,
    worlds: Vec<World>,
}

impl Game {
    fn random_world(width: u32, hight: u32, num_life: u64) -> Game {
        let mut rng = rand::thread_rng();
        
        let mut game = Game {
            life_counter: 0,
            worlds: vec!(),
        };
        
        let mut world = World{
            id: 0,
            width: width,
            hight: hight,
            life: HashMap::new(),
        };

        for i in 0..num_life {
            loop {
                let x: u32 = rng.gen_range(0..world.width);
                let y: u32 = rng.gen_range(0..world.hight);
                let pos = Pos{x:x, y:y};
                if !world.life.contains_key(&pos) {
                    let life = Life{
                        id: i,
                        generation: 0,
                        pos: pos.clone(),
                    };
//                    println!("{:?}", &pos);
                    world.life.insert(pos, life);
                    break;
                }
            }
        }
        game.life_counter = num_life;
        game.worlds.push(world);

        return game;
    }

    fn next(&mut self) {
        let old_world = &self.worlds[self.worlds.len() - 1];
        let mut world = World{
            id: self.worlds.len() as u64,
            width: self.worlds.get(0).unwrap().width,
            hight: self.worlds.get(0).unwrap().hight,
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
                            id: self.life_counter,
                            generation: world.id,
                            pos: pos.clone(),
                        };
                        self.life_counter += 1;
                        world.life.insert(pos.clone(), life);
                    }
                }
            }
        }

        self.worlds.push(world);
    }

    fn print(&self) {
        for w in self.worlds.iter() {
            w.print();
        }
    }

    fn no_change(&self) -> bool {
        let len = self.worlds.len();
        if len < 2 {
            return false;
        }
        let old = &self.worlds[len - 2].life;
        let new = &self.worlds[len - 1].life;

        if old.len() == new.len() && old.keys().all(|k| new.contains_key(k)) {
            return true;
        } else {
            if len > 2 {
                let older = &self.worlds[len - 3].life;
                return older.len() == new.len() && older.keys().all(|k| new.contains_key(k));
            }
            return false;
        }
    }

    fn print_stats(&self) {
        let world = &self.worlds[self.worlds.len() - 1];
        println!("Oldest: {:?}", world.find_oldest());
        println!("Youngest: {:?}", world.find_youngest());
    }
}

struct World {
    id: u64,
    width: u32,
    hight: u32,
    life: HashMap<Pos, Life>,
}


impl World {
    fn print(&self) {
        let mut world = format!("World: {}\n", self.id);

        for y in 0..self.hight {
            for x in 0..self.width {
                match self.life.get(&Pos{x:x, y: y}) {
                    Some(_) => world.push('#'),
                    None => world.push('.'),
                };
            }
            world.push('\n');
        }

        println!("{}", world);
//        println!("{:?}\n", self.life);
    }

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

    fn num_neighbours(&self, pos: &Pos) -> u32 {
        let mut num = 0;
        
        if self.left_neighbour(pos) { num += 1;}
        if self.right_neighbour(pos) { num += 1;}
        if self.top_neighbour(pos) { num += 1;}
        if self.bottom_neighbour(pos) { num += 1;}

        return num;
    }

    fn find_oldest(&self) -> &Life {
        &self.life.values().min_by(|a, b| a.generation.cmp(&b.generation)).unwrap()
    }

    fn find_youngest(&self) -> &Life {
        &self.life.values().max_by(|a, b| a.generation.cmp(&b.generation)).unwrap()
    }
}


#[derive(Eq, PartialEq, Debug, Clone)]
struct Life {
    id: u64,
    generation: u64,
    pos: Pos,
}


#[derive(Hash, Eq, PartialEq, Debug, Clone)]
struct Pos {
    x: u32,
    y: u32,
}

