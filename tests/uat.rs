//! Meteor UAT (User Acceptance Tests)
//!
//! Visual demonstrations and user acceptance validation following RSB patterns.
//! These tests demonstrate functionality with explanatory output.
//!
//! Tests are organized in tests/uat/ subdirectory for detailed demonstrations.

extern crate meteor;

#[cfg(test)]
mod tests {
    use meteor::{parse, parse_shower, BracketNotation};

    /// UAT: Basic functionality demonstration
    #[test]
    fn uat_basic_functionality_demo() {
        println!("🌠 METEOR UAT DEMONSTRATION");
        println!("==========================");
        println!();

        println!("📝 Parsing simple token: key=value");
        let bucket = meteor::parse("key=value").unwrap();
        println!("✅ Successfully parsed!");
        println!("   Retrieved value: {:?}", bucket.get("", "key"));
        println!();

        println!("📝 Parsing bracket notation: list[0]=item");
        let bucket = meteor::parse("list[0]=item").unwrap();
        println!("🔄 Transformed to: list__i_0=item");
        println!("   Retrieved value: {:?}", bucket.get("", "list__i_0"));
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