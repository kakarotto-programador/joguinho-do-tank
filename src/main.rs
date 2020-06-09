use ncurses::*;
use rand::prelude::*;
use std::collections::HashMap;
use std::error::Error;

const COMPONENT_HEIGHT: i32 = 6;
const COMPONENT_WIDTH: i32 = 12;

const WORLD_LIN: i32 = 35;
const WORLD_COL: i32 = 35;

fn draw_entity(y: usize, x: usize, world: bool) -> Result<WINDOW, Box<dyn Error>> {
    let win = if world {
        newwin(WORLD_LIN, WORLD_COL, y as i32, x as i32)
    } else {
        newwin(COMPONENT_HEIGHT, COMPONENT_WIDTH, y as i32, x as i32)
    };

    box_(win, 0, 0);
    wrefresh(win);
    Ok(win)
}

fn destroy_win(win: WINDOW) {
    let ch = ' ' as chtype;
    wborder(win, ch, ch, ch, ch, ch, ch, ch, ch);
    wrefresh(win);
    delwin(win);
}

#[derive(Debug, Clone, Copy)]
struct Enemy {
    // line, col
    pos: (usize, usize),
    dmg: usize,
    hp: usize,
}

impl Enemy {
    fn new(mut rng: ThreadRng) -> Self {
        Self {
            pos: (rng.gen_range(2, 40), rng.gen_range(5, 50)),
            dmg: 2,
            hp: 20,
        }
    }
}

impl Enemy {
    fn generate_random_position(&mut self, mut rng: ThreadRng) -> Result<(), Box<dyn Error>> {
        let minus_or_more = vec![-1, 1];
        let lin = if self.pos.0 as i32 + minus_or_more.choose(&mut rng).unwrap() < 0 {
            1
        } else if self.pos.0 as i32 + minus_or_more.choose(&mut rng).unwrap() > WORLD_LIN {
            WORLD_LIN
        } else {
            let aux = minus_or_more.choose(&mut rng).unwrap();
            rng.gen_range(self.pos.0 as i32 + aux, self.pos.0 as i32 + (aux + 1))
        };

        let col = if self.pos.1 as i32 + minus_or_more.choose(&mut rng).unwrap() < 0 {
            1
        } else if self.pos.1 as i32 + minus_or_more.choose(&mut rng).unwrap() > WORLD_COL {
            WORLD_COL
        } else {
            let aux = minus_or_more.choose(&mut rng).unwrap();
            rng.gen_range(self.pos.1 as i32 + aux, self.pos.1 as i32 + (aux + 1))
        };
        self.pos = (lin as usize, col as usize);
        Ok(())
    }
}

#[derive(Debug)]
enum Dest {
    N,
    S,
    L,
    O,
}

#[derive(Debug)]
struct Player {
    // line, col
    pos: (usize, usize),
    dest: Dest,
    dmg: usize,
    hp: usize,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            pos: (5, 5),
            dmg: 2,
            dest: Dest::O,
            hp: 20,
        }
    }
}

impl Player {
    fn move_position(&mut self, key: i32) {
        self.set_dest(key);
        match key {
            KEY_LEFT => self.pos = (self.pos.0, self.pos.1 - 1),
            KEY_RIGHT => self.pos = (self.pos.0, self.pos.1 + 1),
            KEY_UP => self.pos = (self.pos.0 - 1, self.pos.1),
            KEY_DOWN => self.pos = (self.pos.0 + 1, self.pos.1),
            _ => {}
        }
    }

    fn set_dest(&mut self, key: i32) {
        match key {
            KEY_LEFT => self.dest = Dest::L,
            KEY_RIGHT => self.dest = Dest::O,
            KEY_UP => self.dest = Dest::N,
            KEY_DOWN => self.dest = Dest::S,
            _ => {}
        }
    }
}

struct GameState {
    is_alive: bool,
    player: Player,
    enemies: Vec<Enemy>,
    entities: HashMap<i8, WINDOW>,
    rng: ThreadRng,
}

impl GameState {
    fn new(rng: ThreadRng) -> Result<Self, Box<dyn Error>> {
        let player = Player::default();
        let enemies = vec![Enemy::new(rng), Enemy::new(rng)];
        let mut entities: HashMap<i8, WINDOW> = HashMap::new();
        entities.insert(1, draw_entity(player.pos.0, player.pos.1, false)?);
        for (i, enemy) in enemies.iter().enumerate() {
            entities.insert(2 + i as i8, draw_entity(enemy.pos.0, enemy.pos.1, false)?);
        }

        Ok(GameState {
            is_alive: true,
            player,
            enemies,
            entities,
            rng,
        })
    }

    fn draw_arm(&mut self, w: WINDOW) {
        match self.player.dest {
            Dest::N => mvwaddch(w, 0, COMPONENT_WIDTH / 2, '|' as chtype),
            Dest::S => mvwaddch(w, COMPONENT_HEIGHT - 1, COMPONENT_WIDTH / 2, '|' as chtype),
            Dest::L => mvwaddstr(w, COMPONENT_HEIGHT / 2, 0, "O"),
            Dest::O => mvwaddstr(w, COMPONENT_HEIGHT / 2, COMPONENT_WIDTH - 1, "O"),
        };
    }

    fn draw(&mut self, x: usize, y: usize, win_id: i8) -> Result<(), Box<dyn Error>> {
        if let Some(current_window) = self.entities.get(&win_id) {
            destroy_win(*current_window);
            let w = draw_entity(x, y, false)?;
            // Serve para definir quem eh quem
            // mvwaddch(w, COMPONENT_HEIGHT / 2, COMPONENT_WIDTH / 2, 'P' as chtype);
            self.draw_arm(w);
            wrefresh(w);
            self.entities.insert(win_id, w);
        }
        Ok(())
    }

    fn update_enemies(&mut self) -> Result<(), Box<dyn Error>> {
        for enemy in self.enemies.iter_mut() {
            enemy.generate_random_position(self.rng)?;
        }
        Ok(())
    }

    fn draw_turn(&mut self) -> Result<(), Box<dyn Error>> {
        mvaddstr(LINES() - 1, 0, &format!("player {:?}", self.player));
        self.draw(self.player.pos.0, self.player.pos.1, 1 as i8)?;

        for (i, enemy) in self.enemies.clone().iter().enumerate() {
            self.draw(enemy.pos.0, enemy.pos.1, 2 + i as i8)?;
            mvaddstr(
                LINES() - (2 + i) as i32,
                0,
                &format!("enemy {:? } {:?}", 2 + i, enemy),
            );
        }
        Ok(())
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            match getch() {
                KEY_LEFT => self.player.move_position(KEY_LEFT),
                KEY_RIGHT => self.player.move_position(KEY_RIGHT),
                KEY_UP => self.player.move_position(KEY_UP),
                KEY_DOWN => self.player.move_position(KEY_DOWN),
                KEY_F4 => break,
                _ => {}
            }
            self.update_enemies()?;
            self.draw_turn()?;
            if !self.is_alive {
                break;
            }
        }
        endwin();
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    initscr();
    raw();
    timeout(100);

    // allow for extended keyboard
    keypad(stdscr(), true);

    let mut screen_height = 40;
    let mut screen_width = 40;
    getmaxyx(stdscr(), &mut screen_height, &mut screen_width);
    let rng = rand::thread_rng();
    let mut game = GameState::new(rng)?;
    game.run()?;
    Ok(())
}
