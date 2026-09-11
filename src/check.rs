use std::{env, fs, process::ExitCode};
use system_designer::{exchange, model};
fn run() -> model::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        return Err(model::ModelError::one(
            "usage: designer-check PROJECT.json [PROPOSED.scope.json]",
        ));
    }
    let text = fs::read_to_string(&args[0]).map_err(model::ModelError::one)?;
    let project = model::parse(&text)?;
    if let Some(path) = args.get(1) {
        let packet: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(path).map_err(model::ModelError::one)?)
                .map_err(model::ModelError::one)?;
        let sid = packet["systemId"]
            .as_str()
            .ok_or_else(|| model::ModelError::one("missing systemId"))?;
        exchange::replace(&project, &packet, sid)?;
        println!("Scope replacement validates. No files changed.");
    } else {
        println!(
            "Valid project: {} systems, {} components, {} contracts",
            project.systems.len(),
            project.systems.iter().map(|s| s.nodes.len()).sum::<usize>(),
            project.contracts.len()
        );
    }
    Ok(())
}
fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
