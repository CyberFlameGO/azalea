//! Significantly abstract azalea-protocol so it's actually useable for bots.

mod account;
mod chat;
mod client;
mod movement;
pub mod ping;
mod player;

pub use account::Account;
pub use client::{Client, Event};
pub use movement::MoveDirection;
pub use player::Player;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
