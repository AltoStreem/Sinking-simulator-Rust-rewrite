# Sinking Simulator source reference

SS2/decompiled/com/wicpar contains 217 CFR 0.152 Java-like reference files for the game and its engine, reconstructed from SS2/lib/sinkingsimulator-4.0-all.jar.

The archive contains compiled Kotlin/JVM classes rather than original source. Decompiled names, comments, and Kotlin constructs may be missing or imperfect, and this output is not a ready-to-build replacement project. The original archive and three earlier focused decompilations are preserved beside it. The CFR tool was run from the local Decompile workspace and is not required to run the Rust port.

## Rust port

rust-port contains the ongoing Bevy rewrite. See rust-port/README.md for build and control information. The game assets in SS2 belong to their respective creators; keep them local and follow their licenses.

## Current status (2026-10-14)

The current playable package is [Play.bat](C:/Users/Alto/Desktop/Decompile/SinkingSimulator-Rust-OptionalSteam-ResourceParity-20261014-r13/Play.bat). It includes 302 assets and the ABI-matched Steam DLL; executable SHA-256: a9e0c152ee4425da85aaae13f556563401fe70f1e4cf36c6e6a2e1e53616e648. The launcher starts Steam and music disabled and exposes independent toggles. The packaged executable reports steam=true and music=true; its help confirms the separate switches. Music can be enabled while Steam integration stays off. Live Steam/Workshop and audible playback remain unverified.

The Toolbox now fits its tab row to the available width and keeps every page reachable through a tab-list menu. The active ship stencil GPU pixel check still reads the target as green without expected hull/strut pixels. A real-ship PM Pacmaster source-vs-Bevy check diverges in force output at physics step 7; the exact arithmetic cause is not yet known. The inventory records both open parity issues.

Regular Steam-feature suite: 679 passed, 56 ignored. The focused source-GPU force test passes after its reference stage was aligned with the Bevy force-only test. Release build completed with 0 errors and 314 warnings.

Overall source-game conversion and 1:1 physics, graphics, GUI, and runtime parity remain in progress. See [CHANGELOG.md](CHANGELOG.md) for this release evidence.
