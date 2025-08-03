use bevy::prelude::*;

use crate::{
    gameplay::{DespawnSet, enemy::EnemyType},
    screens::Screen,
    theme::{palette::HEADER_TEXT, widget},
};

pub(crate) struct ScorePlugin;
impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            .add_systems(OnExit(Screen::Gameplay), record_score.before(DespawnSet))
            .add_systems(OnEnter(Screen::Score), spawn_score);
    }
}

#[derive(Resource, Default)]
struct Score {
    blue: usize,
    green: usize,
    red: usize,
    purple: usize,
    yellow: usize,
    cyan: usize,
    white: usize,
}

fn record_score(mut commands: Commands, enemies: Query<&EnemyType>) {
    let mut score = Score::default();
    for enemy in &enemies {
        match enemy {
            EnemyType::Red => score.red += 1,
            EnemyType::Green => score.green += 1,
            EnemyType::Blue => score.blue += 1,
            EnemyType::Purple => score.purple += 1,
            EnemyType::Yellow => score.yellow += 1,
            EnemyType::Cyan => score.cyan += 1,
            EnemyType::White => score.white += 1,
            EnemyType::None => todo!(),
        }
    }

    commands.insert_resource(score);
}

fn spawn_score(mut commands: Commands, score: Res<Score>) {
    commands.spawn((
        widget::ui_root("Score"),
        GlobalZIndex(2),
        StateScoped(Screen::Score),
        children![
            widget::header("Score"),
            score_text("White", score.white),
            score_text("Red", score.red),
            score_text("Green", score.green),
            score_text("Blue", score.blue),
            score_text("Purple", score.purple),
            score_text("Yellow", score.yellow),
            score_text("Cyan", score.cyan),
            widget::button("Quit to title", quit_to_title),
        ],
    ));
}

/// A simple header label. Bigger than [`label`].
pub fn score_text(text: impl Into<String>, score: usize) -> impl Bundle {
    (
        Name::new("Header"),
        Node {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            width: Val::Percent(100.),
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                Text(text.into()),
                TextFont::from_font_size(20.0),
                TextColor(HEADER_TEXT),
                Node {
                    width: Val::Px(100.),
                    ..default()
                },
            ),
            (
                Text(score.to_string()),
                TextFont::from_font_size(20.0),
                TextColor(HEADER_TEXT),
            )
        ],
    )
}

fn quit_to_title(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<Screen>>) {
    next_screen.set(Screen::Title);
}
