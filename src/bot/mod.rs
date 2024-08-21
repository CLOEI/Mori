use crate::types::{
    bot_info::{Info, Server, State},
    vector::Vector2,
};

struct Bot {
    info: Info,
    state: State,
    server: Server,
    position: Vector2,
}
