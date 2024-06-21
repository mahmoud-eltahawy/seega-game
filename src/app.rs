use leptos::{ev, html::button, prelude::*};

#[derive(Debug, Clone, Copy)]
struct Square {
    state: RwSignal<SquareState>,
    position: Position,
}

#[derive(Debug, Clone, Copy)]
struct Position {
    y: usize,
    x: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Player {
    One,
    Two,
}

#[derive(Debug, Clone, Copy)]
enum SquareState {
    Player(Player),
    Empty,
    InZone(Square, Player),
    ZoneKeeper,
}

mod soliders_counter;

const ROW_WIDTH: usize = 5;
const MIDDLE_X_SQUARE: usize = ROW_WIDTH / 2;

lazy_static::lazy_static! {
    static ref ALL_SQUARES : AllSquares = Square::all_squares();
}

lazy_static::lazy_static! {
    static ref ALL_NEIGHBOURS : Vec<Vec<Vec<Square>>> = ALL_SQUARES
        .iter()
        .map(|xs|
            xs
            .iter()
            .map(|x| x.explore_neighbours())
            .collect::<Vec<_>>())
        .collect::<Vec<_>>();
}

impl Player {
    fn oposite(&self) -> Self {
        match self {
            Player::One => Player::Two,
            Player::Two => Player::One,
        }
    }
}

impl Position {
    fn left(self) -> Option<Position> {
        let Position { y, x } = self;
        x.checked_sub(1).map(|x| Position { y, x })
    }
    fn right(self) -> Option<Position> {
        let Position { y, x } = self;
        let x = x + 1;
        if x < ROW_WIDTH {
            Some(Position { y, x })
        } else {
            None
        }
    }

    fn up(self) -> Option<Position> {
        let Position { y, x } = self;
        y.checked_sub(1).map(|y| Position { y, x })
    }

    fn down(self) -> Option<Position> {
        let Position { y, x } = self;
        let y = y + 1;
        if y < ROW_WIDTH {
            Some(Position { y, x })
        } else {
            None
        }
    }
}

type AllSquares = Vec<Vec<Square>>;
impl Square {
    fn new(state: SquareState, y: usize, x: usize) -> Self {
        Self {
            state: RwSignal::new(state),
            position: Position { y, x },
        }
    }
    fn should_die(&self, players_soliders: soliders_counter::SolidersCounter) {
        let get_state = |position: Option<Position>| {
            position
                .map(|pos| ALL_SQUARES.get_square(pos))
                .and_then(|x| x.map(|x| x.state.get_untracked()))
        };
        let SquareState::Player(self_player) = self.state.get_untracked() else {
            return;
        };
        let compare = {
            let enemy_player = self_player.oposite();
            move |one: Player, two: Player| one == two && two == enemy_player
        };
        let should_it = if let (
            Some(SquareState::Player(left_player)),
            Some(SquareState::Player(right_player)),
        ) = (
            get_state(self.position.left()),
            get_state(self.position.right()),
        ) {
            compare(left_player, right_player)
        } else if let (
            Some(SquareState::Player(up_player)),
            Some(SquareState::Player(down_player)),
        ) = (
            get_state(self.position.up()),
            get_state(self.position.down()),
        ) {
            compare(up_player, down_player)
        } else {
            false
        };
        if should_it {
            self.state.set(SquareState::Empty);
            let s = players_soliders.get(self_player);
            s.update(|x| *x -= 1);
        }
    }
    fn all_squares() -> AllSquares {
        assert_eq!(ROW_WIDTH % 2, 1);
        (0..ROW_WIDTH)
            .map(|y| {
                (0..ROW_WIDTH)
                    .map(|x| {
                        let state = if y < MIDDLE_X_SQUARE {
                            SquareState::Player(Player::One)
                        } else if y > MIDDLE_X_SQUARE {
                            SquareState::Player(Player::Two)
                        } else if x < MIDDLE_X_SQUARE {
                            SquareState::Player(Player::One)
                        } else if x > MIDDLE_X_SQUARE {
                            SquareState::Player(Player::Two)
                        } else {
                            SquareState::Empty
                        };
                        Self::new(state, y, x)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    }

    fn get_neighbours(&self) -> &'static Vec<Square> {
        let Position { y, x } = self.position;
        ALL_NEIGHBOURS.get(y).and_then(|list| list.get(x)).unwrap()
    }

    fn explore_neighbours(&self) -> Vec<Square> {
        let mut result = Vec::new();
        let mut push_to_result = |direct: Option<Position>| {
            if let Some(position) = direct {
                result.push(ALL_SQUARES.get_square(position).cloned().unwrap());
            };
        };
        let left = self.position.left();
        let right = self.position.right();
        let up = self.position.up();
        let down = self.position.down();
        push_to_result(left);
        push_to_result(right);
        push_to_result(up);
        push_to_result(down);
        result
    }
}

trait AQ {
    fn get_square(&self, position: Position) -> Option<&Square>;
}
impl AQ for AllSquares {
    fn get_square(&self, Position { y, x }: Position) -> Option<&Square> {
        self.get(y).and_then(|list| list.get(x))
    }
}

#[component]
pub fn App() -> impl IntoView {
    let players_soliders = soliders_counter::SolidersCounter::new();
    let player_turn = RwSignal::new(Player::One);
    let winner = RwSignal::new(None::<Player>);
    provide_context(player_turn);
    provide_context(players_soliders.clone());
    provide_context(winner);
    Effect::new(move |_| {
        let pn1 = players_soliders.get(Player::One).get();
        let pn2 = players_soliders.get(Player::Two).get();
        if pn1 == 0 || pn1 == 1 {
            winner.set(Some(Player::Two));
        }
        if pn2 == 0 || pn2 == 1 {
            winner.set(Some(Player::One));
        }
    });
    view! {
        <main class="flex flex-col min-h-screen justify-center items-center">
        <Show when=move || winner.get().is_some()>
             <WinningCard/>
        </Show>
        <BattleField/>
        </main>
    }
}

#[component]
fn WinningCard() -> impl IntoView {
    let winner = use_context::<RwSignal<Option<Player>>>().unwrap();
    let players_soliders = use_context::<soliders_counter::SolidersCounter>().unwrap();
    let on_click = move |_| {
        winner.set(None);
        players_soliders.reset();
        ALL_SQUARES.iter().enumerate().for_each(|(yi, y)| {
            y.iter().enumerate().for_each(|(xi, x)| {
                if yi < MIDDLE_X_SQUARE {
                    x.state.set(SquareState::Player(Player::One))
                } else if yi > MIDDLE_X_SQUARE {
                    x.state.set(SquareState::Player(Player::Two))
                } else if xi < MIDDLE_X_SQUARE {
                    x.state.set(SquareState::Player(Player::One))
                } else if xi > MIDDLE_X_SQUARE {
                    x.state.set(SquareState::Player(Player::Two))
                } else {
                    x.state.set(SquareState::Empty)
                }
            })
        });
    };
    view! {
        <div class="bg-white z-20 text-2xl">
            <p class="border-2 p-5 m-5">{format!("winner is Player {:#?}",winner.get().unwrap())}</p>
            <button
                on:click=on_click
                class="border-2 p-5 m-5"
            >"play again"</button>
        </div>
    }
}

#[component]
fn BattleField() -> impl IntoView {
    let clean_zone = RwSignal::new(true);
    let players_soliders = use_context::<soliders_counter::SolidersCounter>().unwrap();
    let player_one = move || format!("player one : {}", players_soliders.get(Player::One).get());
    let player_two = move || format!("player two : {}", players_soliders.get(Player::Two).get());
    view! {
        <>
        <div>{player_one}</div>
        <div class="grid grid-cols-5 gap-10 m-5 text-center justify-content-center justify-items-center">
        {
            ALL_SQUARES
                .clone()
                .into_iter()
                .map(|squares| {
                    view! {
                        <For
                            each=move || squares.clone()
                            key=|square| {square.position.x}
                            let:square
                        >
                            <SquareComp square clean_zone/>
                        </For>

                    }
                })
                .collect::<Vec<_>>()
                .into_view()
        }
        </div>
        <div>{player_two}</div>
        </>
    }
}

#[component]
fn SquareComp(square: Square, clean_zone: RwSignal<bool>) -> impl IntoView {
    let players_soliders = use_context::<soliders_counter::SolidersCounter>().unwrap();
    let player_turn = use_context::<RwSignal<Player>>().unwrap();
    let neighbours = square.get_neighbours();
    let empty_neighbours = move || {
        neighbours
            .iter()
            .filter(|x| matches!(x.state.get(), SquareState::Empty))
            .collect::<Vec<_>>()
    };
    let clickable = move || match square.state.get() {
        SquareState::Player(player) if !empty_neighbours().is_empty() && clean_zone.get() => {
            player == player_turn.get()
        }
        SquareState::InZone(_, _) => true,
        _ => false,
    };

    let on_click = move |_| {
        let state = square.state.get_untracked();
        match state {
            SquareState::Player(player) => {
                square.state.set(SquareState::ZoneKeeper);
                empty_neighbours()
                    .iter()
                    .for_each(|x| x.state.set(SquareState::InZone(square, player)));
                clean_zone.set(false);
            }
            SquareState::InZone(owner, player) => {
                square.state.set(SquareState::Player(player));
                owner.state.set(SquareState::Empty);
                player_turn.update(|player| match player {
                    Player::One => *player = Player::Two,
                    Player::Two => *player = Player::One,
                });
                square.should_die(players_soliders);
                neighbours
                    .iter()
                    .for_each(|x| x.should_die(players_soliders));
                owner
                    .get_neighbours()
                    .iter()
                    .filter(|x| matches!(x.state.get(), SquareState::InZone(_, _)))
                    .for_each(|x| x.state.set(SquareState::Empty));
                clean_zone.set(true);
            }
            SquareState::Empty | SquareState::ZoneKeeper => unreachable!(),
        };
    };

    let class = move || match square.state.get() {
        SquareState::Player(Player::One) => "bg-blue-700 w-24 h-24 rounded-full",
        SquareState::Player(Player::Two) => "bg-rose-700 w-24 h-24 rounded-full",
        SquareState::Empty => "border-2 w-24 h-24 rounded-full",
        SquareState::InZone(_, _) => "bg-lime-700 w-24 h-24 rounded-full",
        SquareState::ZoneKeeper => "bg-gray-700 w-24 h-24 rounded-full",
    };
    let style = move || {
        if clickable() && !matches!(square.state.get(), SquareState::Empty) {
            "outline-style : solid"
        } else {
            ""
        }
    };
    button()
        .attr("class", class)
        .attr("style", style)
        .attr("disabled", move || !clickable())
        .on(ev::click, on_click)
        .into_view()
}
