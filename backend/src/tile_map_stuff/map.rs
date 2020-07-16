use std::io::{self, BufRead};

struct Viewport {
    cx: usize,
    cy: usize,
    cw: usize,
    ch: usize,
}

impl Viewport {
    fn move_up(&mut self) {
		if (self.cy != 0) {
			self.cy -= 1;
		}
    }

    fn move_down(&mut self) {
        self.cy += 1;
    }

    fn move_left(&mut self) {
		if (self.cx != 0) {
	        self.cx -= 1;
		}
    }

    fn move_right(&mut self) {
		self.cx += 1;
    }
}

struct Tile {
    id: u8,
    orient: u8,
}

struct TileMap {
    w: usize,
    h: usize,

    tile_map: Vec<Tile>,
}

impl TileMap {
    fn new(w: usize, h: usize) -> Self {
        let mut map = TileMap {
            w,
            h,
            tile_map: Vec::new(),
        };

        for x in 0..map.w {
            for y in 0..map.h {
                let new_tile = Tile { id: 0, orient: 0 };
                map.tile_map.push(new_tile);
            }
        }

        return map;
    }

    fn viewport(&mut self, vx: usize, vy: usize, vw: usize, vh: usize) -> Viewport {
        let view = Viewport {
            cx: vx,
            cy: vy,
            ch: vw,
            cw: vh,
        };

        return view;
    }

    fn gen_view(&self, view: &Viewport) -> Vec<u8> {
        let mut ret_val: Vec<u8> = Vec::new();
        for x in 0..view.cw {
            for y in 0..view.ch {
				if (x+view.cx >= 0 && y+view.cy >= 0 && x+view.cx <= self.w-1 && y+view.cy <= self.h-1) {
					ret_val.push(self.tile_map[((y + view.cy) * self.h + (x + view.cx))].id);
					ret_val.push(self.tile_map[((y + view.cy) * self.h + (x + view.cx))].orient);
				} else {
					ret_val.push(0);
					ret_val.push(0);
				}
            }
        }

        return ret_val;
    }

    fn generate_box_corner(&mut self) {
        for x in 0..self.w {
            for y in 0..self.h {
                if (x == 0 || y == 0) {
                    let new_tile = Tile { id: 2, orient: 0 };
                    self.tile_map[(y * self.h + x)] = new_tile;
                } else if (x == self.w - 1 || y == self.h - 1) {
                    let new_tile = Tile { id: 3, orient: 0 };
                    self.tile_map[(y * self.h + x)] = new_tile;
                }
            }
        }
    }

	fn translate_tile(&mut self, x: usize, y: usize, tx: usize, ty: usize) {
		let mut tile = &self.tile_map[(y * self.h + x)];
		if (x > 0 && y > 0 && x < self.w && y < self.h) {
			if (self.tile_map[(ty * self.h + tx)].id == 0) {
				let air_tile = Tile {
					id: 0,
					orient: 0,
				};
				let place_tile = Tile {
					id: tile.id,
					orient: tile.orient,
				};
				self.tile_map[(y * self.h + x)] = air_tile;
				self.tile_map[(ty * self.h + tx)] = place_tile;
			}
		}
	}

	fn rotate_tile(&mut self, x: usize, y:usize, orient: u8) {
		let mut tile = &self.tile_map[(y * self.h + x)];
		let new_tile = Tile {
			id: tile.id,
			orient: orient,
		};
		self.tile_map[(y * self.h + x)] = new_tile;
	}
}

//testing stuff ----------------------------------------------------------------------------------

struct Robot {
	x: usize,
	y: usize,
	orient: u8,
}

fn prompt() {
    println!("Enter an option: ");
    print!("w. mov up\ns. mov down\na. mov left\nd. mov right\n");
}

fn out_map(v_map: Vec<u8>, view: &Viewport) {
    for x in 0..view.ch {
        for y in 0..view.cw {
            print!("{}{} ", v_map[((y * 2) * view.ch + (x * 2))], v_map[((y * 2) * view.ch + (x * 2)) + 1]);
        }
        print!("\n");
    }
}

fn handle_robot(robot: &mut Robot, map: &mut TileMap, command: &str, param: Option<&str>) {
	if (command == "forward") {
		if (robot.x > 0 && robot.y > 0 && robot.x < map.w && robot.y < map.h)
		{
			if (robot.orient == 0) {
				map.translate_tile(robot.x, robot.y, robot.x, robot.y-1);
				robot.y -= 1;
			} else if (robot.orient == 1) {
				map.translate_tile(robot.x, robot.y, robot.x+1, robot.y);
				robot.x += 1;
			} else if (robot.orient == 2) {
				map.translate_tile(robot.x, robot.y, robot.x, robot.y+1);
				robot.y += 1;
			} else if (robot.orient == 3) {
				map.translate_tile(robot.x, robot.y, robot.x-1, robot.y);
				robot.x -= 1;
			} else {
				println!("Error, invalid robot orientation!");
			}
		}
	} else if (command == "rotate") {
		if let Some(param) = param {
			if (param == "left") {
				if (robot.orient == 3) {
					robot.orient = 0;
				} else {
					robot.orient += 1;
				}
				map.rotate_tile(robot.x, robot.y, robot.orient);
			} else if (param == "right") {
				if (robot.orient == 0) {
					robot.orient = 3;
				} else {
					robot.orient -= 1;
				}
				map.rotate_tile(robot.x, robot.y, robot.orient);
			} else {
				println!("Error, invalid input");
			}
		} else {
			println!("Error, invald input");
		}
	} else {
		println!("Error, invalid input");
	}
}


fn main() {
    let stdin = io::stdin();
    let mut map = TileMap::new(40, 40);
    let mut view = map.viewport(0, 0, 10, 10);
	let mut robot = Robot {
		x: 4,
		y: 4,
		orient: 0,
	};
	let mut robot_tile = Tile {
		id: 1,
		orient: 0,
	};
	map.tile_map[robot.y * map.h + robot.x] = robot_tile;
    map.generate_box_corner();
    print!("{}[2J", 27 as char);
    out_map(map.gen_view(&view), &view);
    prompt();
    for line in stdin.lock().lines() {
        let opt = line.unwrap();
		let tokens: Vec<&str> = opt.split(" ").collect();
		let option = tokens[0];
        if (option == "w") {
            view.move_up()
        } else if (option == "s") {
            view.move_down();
        } else if (option == "a") {
            view.move_left();
        } else if (option == "d") {
            view.move_right()
        } else if (option == "robot") {
			if (tokens.len() == 3) {
				handle_robot(&mut robot, &mut map, &tokens[1], Some(tokens[2]));
			} else if (tokens.len() == 2) {
				handle_robot(&mut robot, &mut map, &tokens[1], None);
			} else {
				println!("No command!");
			}
		}
        print!("{}[2J", 27 as char);
        out_map(map.gen_view(&view), &view);
        prompt();
    }
}
