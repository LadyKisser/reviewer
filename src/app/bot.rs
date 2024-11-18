use poise::serenity_prelude as serenity;
use crate::app::{
    utils::{database::Database, cache::Cache},
    handlers::{button, modal},
};

#[derive(Clone)]
pub struct Bot {
    db: Database,
    cache: Cache,
}

impl Bot {
    pub fn new(db: Database, cache: Cache) -> Self {
        Self { db, cache }
    }

    pub async fn handle_interaction(
        &self,
        ctx: &serenity::Context,
        interaction: serenity::Interaction,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        match interaction {
            serenity::Interaction::MessageComponent(component) => {
                button::handle_button(ctx, &component, &self.db, &self.cache).await
            }
            serenity::Interaction::ModalSubmit(modal_submit) => {
                modal::handle_modal(ctx, &modal_submit, &self.db, &self.cache).await
            }
            _ => Ok(()),
        }
    }
} 