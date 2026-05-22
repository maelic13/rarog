use std::sync::Arc;
use std::thread;

use lynx::engine::Engine;
use lynx::engine_command::{EngineCommandQueue, EngineControl};
use lynx::infra::capitalize_first_letter;
use lynx::uci_protocol::UciProtocol;

fn main() {
    println!(
        "{} {} by {}",
        capitalize_first_letter(env!("CARGO_PKG_NAME")),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS").replace(':', ", ")
    );

    let commands = EngineCommandQueue::default();
    let control = Arc::new(EngineControl::default());
    let mut engine = Engine::new(commands.clone(), Arc::clone(&control));
    let engine_thread = thread::spawn(move || engine.start());

    UciProtocol::new(commands, control).uci_loop();
    engine_thread.join().expect("Engine thread failed.");
}
