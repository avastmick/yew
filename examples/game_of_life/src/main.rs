#![recursion_limit="128"]

#[macro_use]
extern crate yew;
extern crate rand;

use yew::prelude::*;
use std::time::Duration;
use yew::html::ComponentUpdate;
use yew::services::Task;
use yew::services::interval::IntervalService;

struct Context {
    interval: IntervalService,
}

#[derive(Clone, Copy, PartialEq)]
enum LifeState {
    Alive,
    Dead,
}

#[derive(Clone, Copy)]
struct Cellule {
    life_state: LifeState
}

struct GameOfLife {
    cellules: Vec<Cellule>,
    cellules_width: usize,
    cellules_height: usize,
    job: Option<Box<Task>>,
}

impl Cellule {
    pub fn set_alive(&mut self) {
        self.life_state = LifeState::Alive;
    }

    pub fn set_dead(&mut self) {
        self.life_state = LifeState::Dead;
    }

    pub fn alive(&self) -> bool {
        self.life_state == LifeState::Alive
    }

    pub fn count_alive_neighbors(neighbors: &[Cellule]) -> usize {
        neighbors.iter().filter(|n| n.alive()).count()
    }

    pub fn alone(neighbors: &[Cellule]) -> bool {
        Self::count_alive_neighbors(neighbors) < 2
    }

    pub fn overpopulated(neighbors: &[Cellule]) -> bool {
        Self::count_alive_neighbors(neighbors) > 3
    }

    pub fn can_be_revived(neighbors: &[Cellule]) -> bool {
        Self::count_alive_neighbors(neighbors) == 3
    }
}

fn wrap(coord: isize, range: isize) -> usize {
    let result = if coord < 0 {
        (coord + range)
    } else if coord >= range {
        (coord - range)
    } else {
        coord
    };
    result as usize
}


impl GameOfLife {
    pub fn random_mutate(&mut self) {
        for cellule in self.cellules.iter_mut() {
            if rand::random() {
                cellule.set_alive();
            } else {
                cellule.set_dead();
            }
        }
    }

    fn reset(&mut self) {
        for cellule in self.cellules.iter_mut() {
            cellule.set_dead();
        }
    }

    pub fn step(&mut self) {
        let mut to_dead = Vec::new();
        let mut to_live = Vec::new();
        for row in 0..self.cellules_height {
            for col in 0..self.cellules_width {
                let neighbors = self.neighbors(row as isize, col as isize);

                let current_idx = self.row_col_as_idx(row as isize, col as isize);
                if self.cellules[current_idx].alive() {
                    if Cellule::alone(&neighbors) || Cellule::overpopulated(&neighbors) {
                        to_dead.push(current_idx);
                    }
                } else {
                    if Cellule::can_be_revived(&neighbors) {
                        to_live.push(current_idx);
                    }
                }
            }
        }
        to_dead.iter().for_each(|idx| self.cellules[*idx].set_dead());
        to_live.iter().for_each(|idx| self.cellules[*idx].set_alive());
    }

    fn neighbors(&self, row: isize, col: isize) -> [Cellule; 8] {
        [
            self.cellules[self.row_col_as_idx(row + 1, col)],
            self.cellules[self.row_col_as_idx(row + 1, col + 1)],
            self.cellules[self.row_col_as_idx(row + 1, col - 1)],
            self.cellules[self.row_col_as_idx(row - 1, col)],
            self.cellules[self.row_col_as_idx(row - 1, col + 1)],
            self.cellules[self.row_col_as_idx(row - 1, col - 1)],
            self.cellules[self.row_col_as_idx(row, col - 1)],
            self.cellules[self.row_col_as_idx(row, col + 1)],
        ]
    }

    fn row_col_as_idx(&self, row: isize, col: isize) -> usize {
        let row = wrap(row, self.cellules_height as isize);
        let col = wrap(col, self.cellules_width as isize);

        row * self.cellules_width + col
    }

    fn toggle_cellule(&mut self, idx: usize) {
        let cellule = self.cellules.get_mut(idx).unwrap();
        if cellule.life_state == LifeState::Alive {
            cellule.life_state = LifeState::Dead
        } else {
            cellule.life_state = LifeState::Alive
        };
    }
}

enum Msg {
    Random,
    Start,
    Step,
    Reset,
    Stop,
    ToggleCellule(usize)
}

impl Component<Context> for GameOfLife {
    type Msg = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: &mut Env<Context, Self>) -> Self {
        GameOfLife {
            cellules: vec![Cellule { life_state: LifeState::Dead }; 2000],
            cellules_width: 50,
            cellules_height: 40,
            job : None
        }
    }

    fn update(&mut self, msg: Self::Msg, context: &mut Env<Context, Self>) -> ShouldRender {
        match msg {
            Msg::Random => {
                self.random_mutate();
                println!("Random");
            },
            Msg::Start => {
                let callback = context.send_back(|_| Msg::Step);
                let handle = context.interval.spawn(Duration::from_millis(200), callback);
                self.job = Some(Box::new(handle));
                println!("Start");
            },
            Msg::Step => {
                self.step();
            },
            Msg::Reset => {
                self.reset();
                println!("Reset");
            },
            Msg::Stop => {
                if let Some(mut task) = self.job.take() {
                    task.cancel();
                }
                self.job = None;
                println!("Stop");
            },
            Msg::ToggleCellule(idx) => {
                self.toggle_cellule(idx);
            }
        }
        true
    }
}

impl Renderable<Context, GameOfLife> for GameOfLife {
    fn view(&self) -> Html<Context, Self> {
        html! {
            <div>
                <section class="game-container",>
                    <header class="app-header",>
                        <img src="favicon.ico", class="app-logo",/>
                        <h1 class="app-title",>{ "Game of Life" }</h1>
                    </header>
                    <section class="game-area",>
                        <div class="game-of-life",>
                            { for self.cellules.iter().enumerate().map(view_cellule) }
                        </div>
                        <div class="game-buttons",>
                            <button class="game-button", onclick=move|_| Msg::Random,>{ "Random" }</button>
                            <button class="game-button", onclick=move|_| Msg::Step,>{ "Step" }</button>
                            <button class="game-button", onclick=move|_| Msg::Start,>{ "Start" }</button>
                            <button class="game-button", onclick=move|_| Msg::Stop,>{ "Stop" }</button>
                            <button class="game-button", onclick=move|_| Msg::Reset,>{ "Reset" }</button>
                        </div>
                    </section>
                </section>
                <footer class="app-footer",>
                    <strong class="footer-text",>
                      { "Game of Life - a yew experiment " }
                    </strong>
                    <a href="https://github.com/DenisKolodin/yew", target="_blank",>{ "source" }</a>
                </footer>
            </div>
        }
    }
}

fn view_cellule((idx, cellule): (usize, &Cellule)) -> Html<Context, GameOfLife> {
    html! {
        <div class=("game-cellule", if cellule.life_state == LifeState::Alive { "cellule-live" } else { "cellule-dead" }),
            onclick=move |_| Msg::ToggleCellule(idx),> </div>
    }
}

fn main() {
    yew::initialize();
    let context = Context {
        interval: IntervalService::new(),
    };
    let mut app: App<_, GameOfLife> = App::new(context);

    // Send initial message. For demo purposes only!
    // You should prefer to initialize everything in `Component::create` implementation.
    let msg = ComponentUpdate::Message(Msg::Random);
    app.get_env().sender().send(msg);

    app.mount_to_body();
    yew::run_loop();
}
