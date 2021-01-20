use ggez::event::KeyCode;
use ggez_goodies::input::{
    self,
    InputStateBuilder,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameAxis { }
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameButton { 
    A
}

pub type Binding = input::InputBinding<GameAxis, GameButton>;
pub type _Event = input::InputEffect<GameAxis, GameButton>;
pub type State = input::InputState<GameAxis, GameButton>;

pub fn create_input_binding() -> Binding {
    input::InputBinding::new()
       .bind_key_to_button(KeyCode::A, GameButton::A)
}

pub fn create_input_state() -> State {
    InputStateBuilder::new().with_binding(create_input_binding()).build()
}

