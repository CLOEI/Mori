use crate::astar::AStar;
use crate::types::bot::State;
use std::sync::{Mutex, MutexGuard, RwLock};

pub struct MovementController {
    position: RwLock<(f32, f32)>,
    state: Mutex<State>,
    astar: Mutex<AStar>,
}

impl MovementController {
    pub fn new() -> Self {
        Self {
            position: RwLock::new((0.0, 0.0)),
            state: Mutex::new(State::default()),
            astar: Mutex::new(AStar::new()),
        }
    }

    pub fn position(&self) -> (f32, f32) {
        let pos = self.position.read().unwrap();
        *pos
    }

    pub fn try_position(&self) -> Option<(f32, f32)> {
        self.position.try_read().ok().map(|pos| *pos)
    }

    pub fn set_position(&self, x: f32, y: f32) {
        let mut pos = self.position.write().unwrap();
        *pos = (x, y);
    }

    pub fn translate(&self, dx: f32, dy: f32) {
        let mut pos = self.position.write().unwrap();
        pos.0 += dx;
        pos.1 += dy;
    }

    pub fn state(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap()
    }

    pub fn astar(&self) -> MutexGuard<'_, AStar> {
        self.astar.lock().unwrap()
    }
}

impl Default for MovementController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_position_updates() {
        let movement = MovementController::new();
        assert_eq!(movement.position(), (0.0, 0.0));

        movement.set_position(32.0, 64.0);
        assert_eq!(movement.position(), (32.0, 64.0));

        movement.translate(-32.0, 0.0);
        assert_eq!(movement.position(), (0.0, 64.0));
    }

    #[test]
    fn test_state_locking() {
        let movement = MovementController::new();
        {
            let mut state = movement.state();
            state.hack_type = 42;
        }
        let state = movement.state();
        assert_eq!(state.hack_type, 42);
    }

    #[test]
    fn test_concurrent_position_updates() {
        let movement = Arc::new(MovementController::new());
        let mut handles = Vec::new();

        for _ in 0..10 {
            let movement = Arc::clone(&movement);
            handles.push(thread::spawn(move || {
                movement.translate(1.0, 1.0);
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let pos = movement.position();
        assert_eq!(pos, (10.0, 10.0));
    }
}
