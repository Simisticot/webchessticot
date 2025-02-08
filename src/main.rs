use std::collections::HashMap;

use dioxus::logger::tracing::{info, Level};
use dioxus::prelude::{rsx, *};
use libchessticot::{
    BetterEvaluationPlayer, ChessMove, Coords, Piece, PieceColor, PieceKind, Player, Position,
};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");

fn main() {
    dioxus::logger::init(Level::INFO).expect("logger should initialize");
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let position = use_signal(|| Position::initial());
    let selected_square = use_signal(|| None);
    let highlighted_moves = use_signal(|| HashMap::new());
    let engine = use_signal(|| BetterEvaluationPlayer {});
    let promote_to = use_signal(|| {
        PieceKind::promoteable()
            .next()
            .expect("there are promoteable piece kinds")
            .clone()
    });
    let mut y = -1;
    let mut x = -1;
    rsx! {
    document::Link { rel: "icon", href: FAVICON }
    document::Link { rel: "stylesheet", href: MAIN_CSS }
    PromotionSelector{ promote_to }
    {if position.read().is_checkmate() { rsx!{ div { "Checkmate !" } }} else if position.read().is_stalemate() { rsx!{ div { "Stalemate." }}} else { rsx!{ div { "{piece_color_string(&position.read().to_move)} to move" }} }}
    div {
            {
                position.read().board.iter().map(
                    |rank| {
                        y+=1; x=-1; rsx!{
                            div {
                                class:"rank",
                                { rank.iter().map(|contents| {x+=1; let coordinates = Coords{x,y}; rsx!{ Square { square_contents: *contents, coordinates, selected_square, highlighted_moves, position, engine, promote_to}}} ) }
                            }
                        }
                    }
                )
            }
        }
    }
}

#[component]
fn Square(
    square_contents: Option<Piece>,
    coordinates: Coords,
    selected_square: Signal<Option<Coords>>,
    highlighted_moves: Signal<HashMap<Coords, ChessMove>>,
    position: Signal<Position>,
    engine: Signal<BetterEvaluationPlayer>,
    promote_to: Signal<PieceKind>,
) -> Element {
    let text = match square_contents {
        None => "".to_string(),
        Some(piece) => piece_display_name(&piece.kind),
    };
    let color = match square_contents {
        None => "white".to_string(),
        Some(piece) => piece_color_string(&piece.color),
    };
    let selected_class = if selected_square
        .read()
        .is_some_and(|square| square == coordinates)
    {
        "selected".to_string()
    } else {
        "not_selected".to_string()
    };
    let highlighted_class = if highlighted_moves
        .read()
        .keys()
        .collect::<Vec<&Coords>>()
        .contains(&&coordinates)
    {
        "highlighted".to_string()
    } else {
        "not_highlighted".to_string()
    };

    rsx! {
        div { class:"square {color} {selected_class} {highlighted_class}", onclick: move |_| {
                if selected_square.read().is_none(){
                    selected_square.set(Some(coordinates));
                    position.read().legal_moves_from_origin(&coordinates)
                    .iter().for_each(|chess_move|
                    {
                        info!("Adding a highlighted move");
                        highlighted_moves.write().insert(move_to_highlighted_square(chess_move, &position.read().to_move), chess_move.clone());
                    }
                );
                } else {
                if let Some(move_to_make) = highlighted_moves.read().get(&coordinates) {
                    let after_player_move = match move_to_make {
                        ChessMove::Promotion(movement, _) => position.read().after_move(&ChessMove::Promotion(movement.clone(), promote_to.read().clone())),
                        _ => position.read().after_move(move_to_make),
                    };
                    if after_player_move.is_checkmate() || after_player_move.is_stalemate() {
                        position.set(after_player_move);
                    } else {
                        let after_engine_move = after_player_move.after_move(&engine.read().offer_move(&after_player_move));
                        position.set(after_engine_move);
                    }
                }
                selected_square.set(None);
                highlighted_moves.set(HashMap::new());
            }


        } , "{text}"  }

    }
}

#[component]
fn PromotionSelector(promote_to: Signal<PieceKind>) -> Element {
    let update_promote_to = move |evt: FormEvent| {
        promote_to.set(kind_from_display_name(&evt.value()));
    };
    rsx! {
        "Promoting to "
        select {
                name:"promote_to", id:"promote_to",
                onchange: update_promote_to,
                for kind in PieceKind::promoteable() {
                    {
                        rsx!{
                            option { "{piece_display_name(kind)}" }
                        }
                    }
                }
        }
    }
}

fn move_to_highlighted_square(chess_move: &ChessMove, to_move: &PieceColor) -> Coords {
    let homerow = to_move.homerow();
    match chess_move {
        ChessMove::Promotion(movement, _)
        | ChessMove::PawnSkip(movement)
        | ChessMove::EnPassant(movement, _)
        | ChessMove::RegularMove(movement) => movement.destination,
        ChessMove::CastleLeft => Coords { y: homerow, x: 2 },
        ChessMove::CastleRight => Coords { y: homerow, x: 6 },
    }
}

fn piece_display_name(kind: &PieceKind) -> String {
    match kind {
        PieceKind::Pawn => "Pawn".to_string(),
        PieceKind::Rook => "Rook".to_string(),
        PieceKind::Knight => "Knight".to_string(),
        PieceKind::Bishop => "Bishob".to_string(),
        PieceKind::Queen => "Queen".to_string(),
        PieceKind::King => "King".to_string(),
    }
}

fn kind_from_display_name(display_name: &str) -> PieceKind {
    match display_name {
        "Pawn" => PieceKind::Pawn,
        "Rook" => PieceKind::Rook,
        "Knight" => PieceKind::Knight,
        "Bishob" => PieceKind::Bishop,
        "Queen" => PieceKind::Queen,
        "King" => PieceKind::King,
        _ => panic!("Tried to get nonexistent piece kind"),
    }
}

fn piece_color_string(color: &PieceColor) -> String {
    match color {
        PieceColor::White => "white".to_string(),
        PieceColor::Black => "black".to_string(),
    }
}
