pub mod leaderboard;
pub use leaderboard::leaderboard_router as leaderboard;

pub mod doggo;
pub use doggo::doggo_router as doggo;

pub mod upload;
pub use upload::upload_router as upload;

pub mod me;
pub use me::me_router as me;
