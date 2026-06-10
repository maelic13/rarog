use std::sync::Arc;
use std::thread;

use rarog::engine::Engine;
use rarog::engine_command::{EngineCommandQueue, EngineControl};
use rarog::infra::capitalize_first_letter;
use rarog::uci_protocol::UciProtocol;

const ENGINE_THREAD_STACK_SIZE: usize = 16 * 1024 * 1024;

fn main() {
    if !pext_build_is_supported() {
        eprintln!(
            "{} PEXT build requires a CPU with BMI2/PEXT support. Use the AVX2 build on this machine.",
            capitalize_first_letter(env!("CARGO_PKG_NAME"))
        );
        std::process::exit(1);
    }

    println!(
        "{} {} by {}",
        capitalize_first_letter(env!("CARGO_PKG_NAME")),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS").replace(':', ", ")
    );

    let commands = EngineCommandQueue::default();
    let control = Arc::new(EngineControl::default());
    let engine_commands = commands.clone();
    let engine_control = Arc::clone(&control);
    let engine_thread = thread::Builder::new()
        .name("rarog-engine".to_string())
        .stack_size(ENGINE_THREAD_STACK_SIZE)
        // Construct the Engine (which owns the large inline-array Searcher)
        // *inside* this 16 MB thread, not on the caller's stack. In debug builds
        // the default 1 MB Windows main-thread stack overflows while building the
        // Searcher (no copy elision); doing it here keeps the big frames on the
        // large stack the search already runs on. Zero search impact.
        .spawn(move || {
            let mut engine = Engine::new(engine_commands, engine_control);
            engine.start();
        })
        .expect("Engine thread failed to start.");

    UciProtocol::new(commands, control).uci_loop();
    engine_thread.join().expect("Engine thread failed.");
}

#[cfg(all(rarog_pext, target_arch = "x86_64"))]
fn pext_build_is_supported() -> bool {
    std::is_x86_feature_detected!("bmi2")
}

#[cfg(all(rarog_pext, target_arch = "x86"))]
fn pext_build_is_supported() -> bool {
    std::is_x86_feature_detected!("bmi2")
}

#[cfg(not(all(rarog_pext, any(target_arch = "x86", target_arch = "x86_64"))))]
fn pext_build_is_supported() -> bool {
    true
}
