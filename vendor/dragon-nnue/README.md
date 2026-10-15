# Scalar Dragon inference

The runtime is the qualified Rust reconstruction snapshot recorded in
PROVENANCE.json, copied with its original arithmetic and model identity checks.
It has no registry dependencies. Rarog embeds the exact weights from
assets/nnue/ at its host adapter. No executable or analysis runtime is a build
dependency. The original full scalar API remains the reference; cached.rs and
cached/x86.rs implement Rarog's qualified lazy accumulators and runtime kernels. Original module-doc contract paths refer to the
reconstruction repository; the host integration is documented in
../../docs/DRAGON_NNUE.md. Rarog owns adapter/cache/search tests under tests/ and
src/search/mod.rs; the upstream model-dependent test module is not vendored.
