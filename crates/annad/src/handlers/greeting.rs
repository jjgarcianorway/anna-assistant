//! LLM-generated greeting handler.
//! v0.0.275: Uses translator LLM to generate personalized greetings

use super::types::*;
use anna_shared::greeting_context::GreetingContext;
use crate::greeting_generator;

/// Handle GenerateGreeting request - uses translator LLM to generate personalized greeting
pub async fn handle_generate_greeting(
    state: SharedState,
    id: String,
    params: Option<serde_json::Value>,
) -> RpcResponse {
    // Parse greeting context from params
    let ctx: GreetingContext = match params {
        Some(p) => match serde_json::from_value(p) {
            Ok(ctx) => ctx,
            Err(e) => {
                warn!("Invalid greeting params: {}, using defaults", e);
                GreetingContext::default()
            }
        },
        None => GreetingContext::default(),
    };

    // Get translator model from state
    let translator_model = {
        let state = state.read().await;
        state
            .llm
            .translator_model
            .clone()
            .unwrap_or_else(|| state.config.llm.translator_model.clone())
    };

    info!(
        "Generating greeting for {} using {}",
        ctx.username, translator_model
    );

    // Generate greeting with 10 second timeout (greeting should be quick)
    let response = greeting_generator::generate_greeting(&translator_model, &ctx, 10).await;

    info!(
        "Greeting generated: {} chars, llm={}",
        response.greeting.len(),
        response.is_llm_generated
    );

    RpcResponse::success(id, serde_json::to_value(&response).unwrap())
}
