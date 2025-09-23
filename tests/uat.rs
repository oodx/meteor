//! Meteor UAT (User Acceptance Tests)
//!
//! Visual demonstrations and user acceptance validation following RSB patterns.
//! These tests demonstrate functionality with explanatory output.
//!
//! Tests are organized in tests/uat/ subdirectory for detailed demonstrations.

extern crate meteor;

#[cfg(test)]
mod tests {
    use meteor::{parse_shower, BracketNotation, MeteorShower, StorageData};

    /// UAT: Basic functionality demonstration
    #[test]
    fn uat_basic_functionality_demo() {
        println!("🌠 METEOR UAT DEMONSTRATION");
        println!("==========================");
        println!();

        println!("📝 Parsing simple token: app:ui:key=value");
        let shower = meteor::parse_shower("app:ui:key=value").unwrap();
        println!("✅ Successfully parsed into MeteorShower!");
        let found = shower.find("app", "ui", "key");
        println!("   Retrieved value: {:?}", found.map(|m| m.token().value()));
        println!();

        println!("📝 Parsing bracket notation: app:list:list[0]=item");
        let shower = meteor::parse_shower("app:list:list[0]=item").unwrap();
        println!("🔄 Bracket notation supported");
        let found = shower.find("app", "list", "list[0]");
        println!("   Retrieved value: {:?}", found.map(|m| m.token().value()));
        println!();

        println!("🎉 Basic functionality demonstration complete!");
    }

    /// UAT: Bracket notation trait demonstration
    #[test]
    fn uat_bracket_notation_trait_demo() {
        println!("✨ DEMO: BracketNotation Trait");
        println!("==============================");
        println!();

        let flat = "list__i_0";
        println!("📝 Input: {}", flat);
        let bracket = flat.to_bracket();
        println!("🔄 Reconstructed: {}", bracket);
        assert_eq!(bracket, "list[0]");
        println!("✅ Inverse parsing works!");
        println!();

        println!("🎉 BracketNotation trait demonstration complete!");
    }

    /// UAT: MeteorShower collection demonstration
    #[test]
    fn uat_meteor_shower_demo() {
        println!("🚿 DEMO: MeteorShower Collection");
        println!("===============================");
        println!();

        println!("📝 Input: app:ui:button=click; user:settings:theme=dark");
        let shower = meteor::parse_shower("app:ui:button=click; user:settings:theme=dark").unwrap();
        println!("✅ Successfully parsed {} meteors!", shower.len());
        println!();

        println!("🔍 Querying app context:");
        let app_meteors = shower.by_context("app");
        println!("   Found {} meteors in app context", app_meteors.len());
        println!();

        println!("🔍 Querying user context:");
        let user_meteors = shower.by_context("user");
        println!("   Found {} meteors in user context", user_meteors.len());
        println!();

        println!("🎉 MeteorShower demonstration complete!");
    }
}