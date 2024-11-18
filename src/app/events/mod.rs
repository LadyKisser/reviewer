pub mod ready;
pub mod interaction_create;

pub use ready::ready;
pub use interaction_create::interaction_create; 

// TODO: Add events for when the bot is added and removed from a guild. With the option to enable and disable logging.