use bevy::prelude::*;

use crate::screens::Screen;

pub struct PlayingStatePlugin;
impl Plugin for PlayingStatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<Playing>()
            .add_systems(OnEnter(Screen::Gameplay), init_state)
            .add_systems(Update, transition_to_dead.run_if(in_state(Playing::Dying)))
            .add_systems(OnEnter(Playing::Dead), transition_to_score_screen);
    }
}

#[derive(States, Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
#[states(scoped_entities)]
pub enum Playing {
    #[default]
    Live,
    Dying,
    Dead,
}

fn init_state(mut next_state: ResMut<NextState<Playing>>) {
    next_state.set(Playing::Live);
}

#[derive(Deref, DerefMut)]
struct DyingTimer(Timer);
impl Default for DyingTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Once))
    }
}

fn transition_to_dead(
    mut next_state: ResMut<NextState<Playing>>,
    time: Res<Time>,
    mut timer: Local<DyingTimer>,
) {
    if timer.tick(time.delta()).finished() {
        next_state.set(Playing::Dead);
        timer.reset();
    }
}

fn transition_to_score_screen(mut next_state: ResMut<NextState<Screen>>) {
    next_state.set(Screen::Score);
}
