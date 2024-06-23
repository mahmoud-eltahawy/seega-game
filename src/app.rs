use leptos::{
    ev,
    html::{button, div},
    prelude::*,
};

type Square = RwSignal<SquareState>;

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
const TOTAL_SOLIDERS: usize = ROW_WIDTH * ROW_WIDTH;
const ONE_SIDE_SOLIDERS: usize = TOTAL_SOLIDERS / 2;

lazy_static::lazy_static! {
    static ref ALL_SQUARES :[[Square;ROW_WIDTH] ;ROW_WIDTH]  = [[Square::new(SquareState::Empty);ROW_WIDTH] ;ROW_WIDTH];
    static ref ONE_SOLIDERS :[Square ;ONE_SIDE_SOLIDERS]  = [Square::new(SquareState::Player(Player::One)) ;ONE_SIDE_SOLIDERS];
    static ref TWO_SOLIDERS :[Square ;ONE_SIDE_SOLIDERS]  = [Square::new(SquareState::Player(Player::Two)) ;ONE_SIDE_SOLIDERS];
}

lazy_static::lazy_static! {
    static ref ALL_NEIGHBOURS : Vec<Vec<Vec<Square>>> = ALL_SQUARES
        .iter().enumerate()
        .map(|(y,xs)|
            xs
            .iter()
            .enumerate()
            .map(|(x,_)| Position{y,x}.explore_neighbours())
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

    fn explore_neighbours(self) -> Vec<Square> {
        let mut result = Vec::new();
        let mut push_to_result = |direct: Option<Position>| {
            if let Some(position) = direct {
                result.push(ALL_SQUARES.get_square(position).cloned().unwrap());
            };
        };
        let left = self.left();
        let right = self.right();
        let up = self.up();
        let down = self.down();
        push_to_result(left);
        push_to_result(right);
        push_to_result(up);
        push_to_result(down);
        result
    }

    fn get_neighbours(self) -> &'static Vec<Square> {
        let Position { y, x } = self;
        &ALL_NEIGHBOURS[y][x]
    }

    fn get_square(self) -> Square {
        let Self { y, x } = self;
        ALL_SQUARES[y][x]
    }
}

type AllSquares = [[RwSignal<SquareState>; ROW_WIDTH]; ROW_WIDTH];
trait Sq {
    fn should_die(&self, position: Position, players_soliders: soliders_counter::SolidersCounter);
}
impl Sq for Square {
    fn should_die(&self, position: Position, players_soliders: soliders_counter::SolidersCounter) {
        let get_state = |position: Option<Position>| {
            position
                .map(|pos| ALL_SQUARES.get_square(pos))
                .and_then(|x| x.map(|x| x.get_untracked()))
        };
        let SquareState::Player(self_player) = self.get_untracked() else {
            return;
        };
        let compare = {
            let enemy_player = self_player.oposite();
            move |one: Player, two: Player| one == two && two == enemy_player
        };
        let should_it = if let (
            Some(SquareState::Player(left_player)),
            Some(SquareState::Player(right_player)),
        ) = (get_state(position.left()), get_state(position.right()))
        {
            compare(left_player, right_player)
        } else if let (
            Some(SquareState::Player(up_player)),
            Some(SquareState::Player(down_player)),
        ) = (get_state(position.up()), get_state(position.down()))
        {
            compare(up_player, down_player)
        } else {
            false
        };
        if should_it {
            self.set(SquareState::Empty);
            let s = players_soliders.get(self_player);
            s.update(|x| *x -= 1);
        }
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
    provide_context(players_soliders);
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
        <AllSolidersFields/>
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
        ALL_SQUARES
            .iter()
            .for_each(|y| y.iter().for_each(|x| x.set(SquareState::Empty)));
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
fn AllSolidersFields() -> impl IntoView {
    (player_one_field(), battle_field(), player_two_field())
}

fn player_one_field() -> impl IntoView {
    let players_soliders = use_context::<soliders_counter::SolidersCounter>().unwrap();
    let player_one = move || format!("player one : {}", players_soliders.get(Player::One).get());
    div().child(player_one)
}

fn player_two_field() -> impl IntoView {
    let players_soliders = use_context::<soliders_counter::SolidersCounter>().unwrap();
    let player_two = move || format!("player two : {}", players_soliders.get(Player::Two).get());
    div().child(player_two)
}

fn battle_field() -> impl IntoView {
    let clean_zone = RwSignal::new(true);
    div()
        .attr(
            "class",
            "grid grid-cols-5 gap-10 m-5 text-center justify-content-center justify-items-center",
        )
        .child(
            ALL_SQUARES
                .iter()
                .enumerate()
                .map(|(y, squares)| {
                    squares
                        .iter()
                        .enumerate()
                        .map(|(x, _)| square_comp(Position { y, x }, clean_zone))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>(),
        )
}

fn square_comp(position: Position, clean_zone: RwSignal<bool>) -> impl IntoView {
    let square = position.get_square();
    let players_soliders = use_context::<soliders_counter::SolidersCounter>().unwrap();
    let player_turn = use_context::<RwSignal<Player>>().unwrap();
    let neighbours = position.get_neighbours();
    let empty_neighbours = move || {
        neighbours
            .iter()
            .filter(|x| matches!(x.get(), SquareState::Empty))
            .collect::<Vec<_>>()
    };
    let clickable = move || match square.get() {
        SquareState::Player(player) if !empty_neighbours().is_empty() && clean_zone.get() => {
            player == player_turn.get()
        }
        SquareState::InZone(_, _) => true,
        _ => false,
    };

    let on_click = move |_| {
        let state = square.get_untracked();
        match state {
            SquareState::Player(player) => {
                square.set(SquareState::ZoneKeeper);
                empty_neighbours()
                    .iter()
                    .for_each(|x| x.set(SquareState::InZone(square, player)));
                clean_zone.set(false);
            }
            SquareState::InZone(owner, player) => {
                square.set(SquareState::Player(player));
                owner.set(SquareState::Empty);
                player_turn.update(|player| match player {
                    Player::One => *player = Player::Two,
                    Player::Two => *player = Player::One,
                });
                square.should_die(position, players_soliders);
                neighbours
                    .iter()
                    .for_each(|x| x.should_die(position, players_soliders));
                position
                    .get_neighbours()
                    .iter()
                    .filter(|x| matches!(x.get(), SquareState::InZone(_, _)))
                    .for_each(|x| x.set(SquareState::Empty));
                clean_zone.set(true);
            }
            SquareState::Empty | SquareState::ZoneKeeper => unreachable!(),
        };
    };

    let class = move || match square.get() {
        SquareState::Player(Player::One) => "bg-blue-700 w-24 h-24 rounded-full",
        SquareState::Player(Player::Two) => "bg-rose-700 w-24 h-24 rounded-full",
        SquareState::Empty => "border-2 w-24 h-24 rounded-full",
        SquareState::InZone(_, _) => "bg-lime-700 w-24 h-24 rounded-full",
        SquareState::ZoneKeeper => "bg-gray-700 w-24 h-24 rounded-full",
    };
    let style = move || {
        if clickable() && !matches!(square.get(), SquareState::Empty) {
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
        .child(format!("{:#?}", position))
}
