use poise::serenity_prelude::{Context, Interaction};
use crate::app::{
    handlers::{button, modal},
    utils::database::Database,
};
use tracing::error;

pub async fn interaction_create(
    ctx: Context,
    interaction: Interaction,
    db: &Database,
) {
    match interaction {
        Interaction::MessageComponent(component) => {
            if let Err(e) = button::handle_button(&ctx, &component, db).await {
                error!("Error handling button: {}", e);
            }
        }
        Interaction::ModalSubmit(modal_submit) => {
            if let Err(e) = modal::handle_modal(&ctx, &modal_submit, db).await {
                error!("Error handling modal: {}", e);
            }
        }
        _ => {}
    }
} 