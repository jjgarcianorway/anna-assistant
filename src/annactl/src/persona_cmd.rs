//! Persona management commands

use anyhow::Result;
use anna_common::{
    anna_narrative, anna_info, anna_ok, anna_warn, anna_box, MessageType,
    bundled_personas, get_persona_state, set_persona, explain_persona_choice,
    PersonaMode,
};

/// Get current persona
pub async fn persona_get() -> Result<()> {
    anna_narrative("Let me check which persona I'm using...");

    let state = get_persona_state()?;
    let personas = bundled_personas();

    if let Some(persona) = personas.get(&state.current) {
        anna_ok(format!("Current persona: {}", persona.name));
        println!("  {}", persona.description);
        println!();
        println!("  Traits:");
        println!("    Verbosity:      {}/5", persona.traits.verbosity);
        println!("    Emojis:         {}", if persona.traits.emojis { "yes" } else { "no" });
        println!("    Colorfulness:   {}/3", persona.traits.colorfulness);
        println!("    Tips frequency: {}/5", persona.traits.tips_frequency);
        println!();

        let mode_str = match state.mode {
            PersonaMode::Auto => "auto-detect (may change based on context)",
            PersonaMode::Fixed => "fixed (won't change automatically)",
        };
        anna_info(format!("Mode: {}", mode_str));
    } else {
        anna_warn(format!("Unknown persona: {}", state.current));
    }

    Ok(())
}

/// Set persona
pub async fn persona_set(name: &str, mode: PersonaMode) -> Result<()> {
    let personas = bundled_personas();

    // Validate persona exists
    if !personas.contains_key(name) {
        anna_warn(format!("I don't know a persona called '{}'.", name));
        anna_info("Available personas:");
        for persona_name in personas.keys() {
            if let Some(p) = personas.get(persona_name) {
                println!("  • {} - {}", p.name, p.description);
            }
        }
        return Ok(());
    }

    let mode_str = match mode {
        PersonaMode::Auto => "auto-detect mode",
        PersonaMode::Fixed => "fixed mode",
    };

    anna_narrative(format!("I'll switch to the '{}' persona in {}...", name, mode_str));

    set_persona(name, mode)?;

    let persona = &personas[name];
    anna_ok(format!("Switched to '{}' persona!", persona.name));
    println!("  {}", persona.description);

    if mode == PersonaMode::Fixed {
        anna_info("This is now fixed - I won't auto-switch based on context.");
    } else {
        anna_info("This is in auto-detect mode - I may adapt based on your usage patterns.");
    }

    Ok(())
}

/// Explain persona choice
pub async fn persona_why() -> Result<()> {
    anna_box(&["Why am I using this persona?"], MessageType::Narrative);
    println!();

    let state = get_persona_state()?;
    let explanation = explain_persona_choice();

    anna_info(format!("Current persona: {}", state.current));
    println!();
    println!("{}", explanation);
    println!();

    if state.mode == PersonaMode::Fixed {
        anna_info("You set this persona explicitly with 'annactl persona set --fixed'.");
    } else {
        anna_info("This is either the default or I auto-detected it based on your system.");
    }

    Ok(())
}

/// List all available personas
pub async fn persona_list() -> Result<()> {
    anna_narrative("Here are the personas I can adopt:");
    println!();

    let personas = bundled_personas();
    let mut names: Vec<String> = personas.keys().cloned().collect();
    names.sort();

    for name in names {
        if let Some(persona) = personas.get(&name) {
            println!("📋 {}", persona.name);
            println!("   {}", persona.description);
            println!("   Verbosity: {}/5 | Emojis: {} | Colors: {}/3 | Tips: {}/5",
                persona.traits.verbosity,
                if persona.traits.emojis { "yes" } else { "no" },
                persona.traits.colorfulness,
                persona.traits.tips_frequency
            );
            println!();
        }
    }

    anna_info("Try: annactl persona set <name> --fixed");

    Ok(())
}
