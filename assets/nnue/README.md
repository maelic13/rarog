# Qualified Dragon network

`dragon-network.nnue` is the exact extracted network used by the experimental
Dragon evaluator, imported at the maintainer's request. Size: 70,136,671 bytes.
SHA-256: `c6e25a7fde5971dc47e78e19764c82b8d18a438bbd5ee4da065d061115064417`.
Extraction/reconstruction provenance is in `vendor/dragon-nnue/PROVENANCE.json`
at the repository root. These immutable model bytes are not trained by Rarog.

Rarog embeds this asset at build time, so selecting `Evaluation = DragonNNUE`
works from any working directory without another repository. `DragonEvalFile`
defaults to `<embedded>`; an explicit filename overrides it and still receives
the full identity check. Classical remains the default evaluator.
