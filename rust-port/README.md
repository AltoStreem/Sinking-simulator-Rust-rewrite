# Sinking Simulator — Bevy port

This is a playable Rust/Bevy port with a collapsible left toolbox, Ships/Physics/Graphics/Music/Performance/Advanced tabs, source-mapped day/night sky with a procedural star field, shader-gradient ocean depth, a shader-driven wave surface, ripple-distorted ship reflection, and depth-attenuated underwater effects. The Toolbox header and tab layout follow the supplied source screenshots; numeric settings are drag-editable, Graphics includes a hue/saturation/value sea-color picker, the ship browser has a searchable list, and the tool strip exposes layer and size controls. These are source-inspired recreations, not a pixel-identical ImGui skin yet. The application renders at 2554 × 1378 while keeping the full-size world and interface layout. Ships with paired `_base.png` maps retain per-texel material identities and source eight-neighbor connectivity. WGSL compute passes translate the source solver's structural forces, spring breakage, wave-relative drag and buoyancy, floor collision, hull-water ingress, eight-neighbor interior-water transfer, and water-dependent mass updates. Per-texel positions and water state are read back asynchronously for rendering and UI. The rest of the application, including some editing tools and rendering effects, is still a work in progress and is not claimed to be a complete game-parity port. Break cuts a brush-sized material-map hole and Repair restores it. Drop a PNG ship image onto the game window to import it into the searchable ship list; imported images use the port's default hull material and are saved under `assets/user_ships/`. **Mouse wheel** zooms toward the pointer, **+/-** zoom, **0** resets zoom, **right-drag** pans (or select Move and left-drag), **P** pumps, **F** adds water, **D** drains, **Space** pauses, and **R** resets. Use **1** for damage and **2** to repair near a leak.

Physics, Graphics, and Performance tabs expose the original game's main settings, including wave geometry, buoyancy, flow/influx, spring properties, gravity, day-cycle length, and solver controls. The Music tab plays the bundled tracks with volume control. White/unmapped material pixels remain air; keyed near-white mattes on ship art are made transparent. Ships without paired material maps use inferred/default hull physics. The underwater effect is a world-space overlay rather than the original screen-space post-processing. Steam Workshop integration and the source's layered ship editor are not ported. The adjacent SS2/decompiled/ folder now contains 217 CFR 0.152 reference files for the game and engine packages. They reconstruct the Kotlin/JVM bytecode into Java-like code, so use them as behavioral references rather than original source. The local SS2 folder contains all 125 PNGs from its ship directory, mirrored into assets/ss2_ships/; the browser lists standalone root ships and paired material-map vessels while excluding structure-building pieces. Licensed game art and soundtrack assets remain local and should not be redistributed without their rights.

With a current stable Rust toolchain installed, run `cargo run` from this directory for a development build. To produce a refreshed Windows release bundle, run `build-playable.bat` from this directory; it builds the optimized executable and copies runtime assets into `playable/`, excluding the redundant `assets/original` archive. Launch `playable/Play.bat` to run the packaged build. Porting is ongoing: the Bevy application is playable, but it remains an approximation of the original game. Bevy's official quick start documents the `App`/ECS approach used here: https://bevy.org/learn/quick-start/getting-started/ and https://bevy.org/learn/quick-start/getting-started/apps/.


## Rendering conversion progress (2026-10-06)

- `src/fragment_shaders.rs` and `assets/shaders/ship_texture.wgsl` translate the recovered generic fragment operations and Ship's daylight, interior/exterior lighting, and non-hull flood-color calculations. The selected layer loads its own light maps; absent maps use black. Reference arithmetic tests and WGSL validation pass.
- `src/texture_2d.rs` translates the ShipResource texture sampler configuration recovered from `ShipResource$texture$1.class`: nearest magnification/minification and linear mip-level filtering. Ship appearance and light-map images now include mip chains. Border transparency is handled in the ship shader without requiring optional GPU sampler features. CPU mip reduction uses image's triangle filter; exact equality with the original driver's generated mip pixels has not been established. This covers the ship texture path, not every original Texture2D constructor and resource lifecycle operation.
- `src/floor.rs` translates Floor's gray half-plane at the configured sea depth. The visible rectangle follows camera bounds and renders after the ship and before the underwater overlay. Full source framebuffer/post-processing composition is still incomplete.

Current verification: 45 tests pass; `default_ship_solver_does_not_tear_without_user_damage` still fails on the retained CPU solver's deformation check. These rendering conversions have compiled, but runtime visual parity has not been verified. The complete 217-class conversion remains unfinished.


## Music conversion progress (2026-10-06)

`src/music_player.rs` now contains the translated playlist state and Bevy audio adapter. It discovers Ogg tracks recursively, collapses duplicate track names, follows Java HashMap bucket ordering for the bundled track list, removes each selected track from the queue, supports shuffle, refills the queue when repeat is enabled, and pauses when a non-repeating queue is exhausted. Finished tracks advance automatically. The source Main's initially empty player starts paused; press the play icon to begin.

Volume changes and pause/resume now act on the existing AudioSink instead of recreating the track. The original shuffle, repeat, play/pause, and next icons are connected. The playback-position bar seeks within the selected track. Decoder duration calculations run on a background task and are cached. The UI remains a Bevy recreation rather than the original ImGui layout, and actual audible playback/seeking still needs a runtime comparison.

Six music-specific tests pass, including queue transitions, no-repeat exhaustion, shuffle removal, automatic-next behavior, Java hash buckets, and decoding the original Night Vigil soundtrack. Full regression results after this conversion: 51 passed, 1 existing CPU deformation test failed. Full game parity and the remaining class conversions are still unfinished.


## Sky and geometry conversion progress (2026-10-06)

The source-based `src/sky.rs` renderer is now connected to the application, replacing the separate main-file backdrop approximation. It uses the original day/horizon lookup map with source linear filtering, moves its fullscreen plane with the camera, translates source sea y=0 to the port's sea coordinate, and applies source star noise, eight-bit quantization, and OpenGL framebuffer Y orientation. The source bakes a star texture; the port currently evaluates the same field in its sky shader. Exact driver-to-driver star values and complete source framebuffer composition remain unverified.

Separate Rust files now implement `Model`, `UVModel`, `Fullscreen`, and `VertexShaders` position/UV geometry and transform contracts. The active sky pass uses those adapters. Bevy owns the mesh buffers and rendering lifecycle. Tests cover fullscreen index/corner order, fitting to camera bounds, loop/fan geometry, vertex transforms, horizon mapping, and WGSL validation. Full regression results: 56 passed, 1 existing CPU deformation failure.

`conversion_inventory.csv` records all 217 Java reference files and their currently identified dedicated Rust modules. An implementation entry is not a completion claim: full class coverage and runtime parity still require inspection. Classes without a recorded dedicated file remain in the audit scope; some behavior may still be embedded in main.rs. Full 1:1 conversion remains unfinished.


## Camera conversion progress (2026-10-06)

`src/camera_2d.rs` translates Camera2D's sixteen-pixel native constructor, matrix translation, relative zoom, top-left resize anchor, and callback registration/update/removal. The active `camera_control.rs` adapter now uses those operations: scrolling zooms around the pointer, +/- (including repeated key events) zoom around the view center, resizing preserves the top-left world point and pixel scale, and right-drag starts even over UI as in the source. The previous port-only zoom clamps and unused duplicate camera handler were removed.

Brushes and ship movement now use the immediately updated camera matrix rather than the prior frame's Bevy GlobalTransform. UI blocking follows visible toolbox/header/tool-panel rectangles. The advertised 0 reset shortcut remains a port extension. The active scene still uses the port's normalized world units; conversion to source-native ship dimensions and initial camera magnification remains unfinished, so the source-native camera constructor alone does not establish visual parity.

Six camera tests pass, including a Bevy input-message test for pointer anchoring beyond the old zoom limit and center-anchored repeated keyboard zoom. Full regression results: 62 passed, 1 existing CPU deformation failure. Interactive comparison and full game/class conversion remain unfinished.

## Property conversion progress (2026-10-06)

Separate BackedProperty, FloatProperty, IntProperty, and Vector2Property modules preserve deferred notifications, immediate registration callbacks, default old=0, listener order/removal, and Java float equality. Vector component slices borrow the shared backing data. These utility classes are not wired into the application window; automatic before-events registration and Resource lifecycle translation remain pending. This is class conversion progress, not a full parity claim.

## Movement and water tool conversion progress (2026-10-06)

`src/tools/move_tool.rs` translates MoveTool's unblocked left-click activation and Shift restriction, plus Ship's preview/commit drag protocol. The active solver freezes while dragging; release translates every GPU position while preserving velocities. The drag can start anywhere in the scene, and blocked cursor/release events retain source behavior. Reset and replacement of the ship clear the drag. Bevy's asynchronous position readback still introduces preview/commit display latency; complete input/render parity is unverified.

FloodTool and DryTool now queue a separate GPU brush pass that samples current GPU positions and water. It changes only water X using the source radius-minus-distance formula, preserving Y/ZW and transport scratch planes. The earlier CPU snapshot re-upload was removed because it could overwrite newer simulation state. Water brushes are no longer rejected solely by the original undeformed ship rectangle. GPU command delivery during shader initialization and asynchronous readback/reset behavior still require runtime auditing.

`src/kotlin_helpers.rs` translates the generic and float mapInPlace/mapInPlaceIndexed overloads using borrowed Rust slices; tests check replacement order, original allocation, object values, and empty arrays. The source utility has no identified callers in the decompiled game packages.

Verification: 75 tests pass, including movement state transitions, actual Bevy command consumption/freeze settings, source water arithmetic, property contracts, and full physics WGSL parsing/type validation. The retained inactive CPU solver's deformation test still fails at 19.523502. These checks do not prove full GPU gameplay stability or 1:1 parity. The packaged playable executable from the earlier build has not been refreshed with this conversion.

## Physics pass fidelity corrections (2026-10-06)

The recovered ShipPhysics force shader samples mass-strength channels W/Y/Z. The Rust compute shader now uses W (effective mass) for both spring stiffness and acceleration. Its water/mass pass writes only W, mixing the unchanged raw mass X with fluid density; it previously overwrote X and mixed from the initialized weighted mass. This correction preserves the source's material-channel meaning rather than tuning around the mismatch.

The source finalPass also repairs reciprocal structural/water links after each water step. Two compute dispatches now snapshot mask inputs into a second storage plane and intersect each link with its neighbor's opposite direction bit. This supplies stable input for the reciprocal pass. Allocation, reset, damage uploads, and readback now consistently handle the two mask planes; rendering/readback consume only the live plane. Drag exposure uses the post-break water-link mask, and coincident springs still evaluate breakage while skipping undefined normalization, as in the source shader.

The runtime audit found Bevy resolving assets beside target/release while direct source-style file reads used the working directory. AssetPlugin now uses that same working-directory assets path. A release build ran for 18 seconds with no asset or GPU initialization errors (only a gamepad mapping warning). This is startup evidence, not proof of stable sinking, long-running behavior, or visual parity.

Verification after final changes: 77 tests pass; the existing inactive CPU deformation test remains failed at 19.523502. Tests cover effective-mass channel preservation/no feedback, reciprocal one-sided cuts, full physics shader validation, and previous conversion regressions. The complete class/engine conversion and runtime 1:1 comparison remain unfinished. The packaged playable executable has not been refreshed with these changes.

## Sea source conversion and current packaged build (2026-10-06)

Added src/sea.rs for Sea.java wave arithmetic, including its literal 3.141592 constant. The active scene uses that function. The depth shader now uses the source linear depth / 1000 * waterDarkness calculation instead of a sea-depth-normalized power curve and reciprocal brightness. Sea geometry expands to the current inverse-camera bounds, including camera pan and zoom; configured sea depth no longer clips water above the viewport bottom. This is a partial translation: source framebuffer reflection sampling, boundary antialiasing, alpha composition, and replacement of the remaining surface/underwater adapters are still required.

The packaged playable executable and assets were refreshed before this sea conversion. That package stayed running for 15 seconds without asset or shader errors; only a gamepad mapping warning appeared. Source-native coordinates, default Pacmaster selection, Break's Y/Z mask cuts, and source brush preview colors/radius are included in that package. The full suite before the sea change had 83 passes and one failure: the retained inactive CPU Titanic deformation regression (20.579706). Default Pacmaster passing does not establish Titanic or GPU stability. No full 1:1 completion is claimed.
## Fullscreen Sea and ScreenFBO conversion (2026-10-06)

Added src/screen_fbo.rs for the original window-sized RGBA8 render target and resize behavior. Sea now renders a fullscreen composition of the world framebuffer, following the current camera matrix. The earlier ocean surface/depth, underwater overlay and separate reflection entities are removed from the active scene. The source shader's wave height, reflected coordinate, two-pixel edge transition, reflection weight, linear depth darkening and final RGB/alpha composition are translated in assets/shaders/sea.wgsl. Default sea RGB now matches 0/71/159. Source framebuffer GL_CLAMP_TO_BORDER behavior is implemented with transparent-black bilinear border loads.

The original framebuffer uses regenerated mipmaps and LINEAR_MIPMAP_LINEAR. The adapter currently samples the base level only; mipmap regeneration and the original distance-based reflection LOD remain incomplete. Stencil behavior, configurable sea alpha, lifetime/dependency callbacks, and runtime visual comparison also remain unverified. These are partial conversions, not completed class parity.

Verification: the fullscreen sea shader passed Naga parsing/type validation, and the full suite reported 85 passes and one retained inactive CPU Titanic deformation failure (20.579636). The release version of the new framebuffer path stayed running for 15 seconds without asset, shader, or rendering initialization errors (one gamepad mapping warning). Border/default-colour refinements were made after that startup check. The packaged playable folder remains the earlier build; complete 1:1 conversion remains unfinished.
## Toolbox RGBA graphics controls (2026-10-06)

Toolbox.java calls colorPicker4 with AlphaBar and AlphaPreviewHalf. Added src/toolbox.rs for the adapted picker input contract and connected the existing panel's alpha slider, all four RGBA channel readouts, a live alpha marker, and the opaque/transparent half preview over a checkerboard. The alpha bar displays current colour blended over checks. Channel drags use 1/255 increments; RGB changes keep the hue picker synchronized, and hue/saturation changes preserve alpha. Simulation now carries sea alpha from the original GameParameterProvider default (191/255), and the active Sea material receives the full current RGBA value rather than a fixed alpha. The ship material receives the same source waterColor uniform; its source shader intentionally uses RGB and retains its fixed 0.75 interior-water blend.

Three picker tests pass, including a Bevy system test proving the alpha preview and A readout receive the changed state. This is partial Toolbox conversion in the permitted adapted UI layout; keyboard text editing, other pages, ship editor/upload/workshop, complete graphical comparison, and full 1:1 conversion remain pending. The packaged playable folder was not refreshed by this source conversion.
## RenderFBO mipmap and reflection filtering conversion (2026-10-06)

RenderFBO.onUnbind regenerates each framebuffer texture's mipmaps. Added src/render_fbo.rs for that GPU operation, keeping the window-sized target allocation/resize in src/screen_fbo.rs. The world camera output is copied into a complete RGBA8 mip chain, then each level is generated after the world output pass and before the Sea camera. The adapter uses linear downsampling into quantized RGBA8 storage; OpenGL driver's exact mip-generation kernel remains unverified, particularly for odd dimensions. The sea fragment now interpolates adjacent mip levels using screen derivatives plus the original distance * 20 reflection LOD bias, with linear per-level sampling and transparent border texels.

Allocation/resize tests verify complete mip counts and stable image handles. Compute and sea shaders pass Naga validation. The full suite reported 90 passes and the retained inactive CPU Titanic deformation failure (20.579636). The release implementation stayed running for 18 seconds without GPU, shader, or asset initialization errors; the class-file split afterward also compiles and passes the framebuffer tests. Startup evidence does not prove pixel parity or long-running gameplay. Multiple framebuffer attachments, source stencil behavior, generic resource lifetime, driver filter parity and full runtime comparison remain pending. The packaged playable build was not refreshed.
## TimeSync and source frame-clock conversion (2026-10-06)

Added src/time_sync.rs translating TimeSync.java's 0.016666668 reference, nanosecond-derived resource/speed samples, integer-millisecond sleep, drained averages and report text. The active adapter runs at the end of Bevy's frame and reports through the window title. Its callback is executed on the main thread instead of the original coroutine dispatcher; Resource disposal/background-callback scheduling remain incomplete. Window presentation now requests AutoNoVsync to match the original glfwSwapInterval(0), subject to platform fallback behavior.

Main.java advances game time by the fixed reference after rendering. The port previously combined fixed per-frame GPU integration with a wall-time scene clock; it now advances scene time once per frame after settings/materials consume the current value. Pause is retained as a requested port feature. The added daylight phase offset was removed: the cycle follows the source float phase/remainder and double cosine calculation, starting at day 1. Day edits no longer rewrite an invented cycle offset.

Three timing tests pass, including actual Bevy frame advancement/pause. The full suite reports 93 passes and one retained inactive CPU Titanic deformation failure (20.579636). The release build compiled; startup/report verification is recorded separately below. These checks do not establish full game/engine conversion or 1:1 runtime parity. The packaged playable build was not refreshed.
Runtime timing check: the release stayed running for 15 seconds without asset, shader, GPU or scheduling errors. Its live title reported Resources: 57% usage, 100% speed. Only the existing gamepad mapping warning appeared. This verifies startup and live reporting, not full behavioral parity.

## Resource, GLResource and ALResource conversion (2026-10-06)

Added separate src/resource.rs, src/gl_resource.rs and src/al_resource.rs translations. Resource preserves weak dependents/registry entries, immediate freed flags, idempotent closure, dependent-before-parent cleanup, deferred FIFO main-thread work, and closeAll registry locking. Concurrent close waits for another thread's closure to finish; same-thread dependency cycles use the source reentrant-close behavior. Rust last-owner drop adapts JVM finalization, and Bevy frame-boundary pruning/synchronous registration adapts the coroutine/daemon timing. These timing differences and exception behavior require further auditing before full class parity can be claimed.

GLResource retains its integer ID and strong explicit context dependency. ALResource retains its generic ID and forwards dependencies. Default current-GLContext construction and concrete graphics/audio consumers remain pending. TimeSync is the first active lifecycle consumer: closeAll marks its handle closed, while reporting stops only after deferred free runs. Resource's frame queue, application-exit cleanup and TimeSync frame pacing are constrained to Bevy's main thread through a non-send marker. AppExit closes all registered resources and flushes their queued frees, corresponding to Main.java's exit sequence.

Tests cover dependency/FIFO ordering, idempotence, weak last-owner behavior, cycles, worker-thread closes, actual Bevy exit cleanup, GL/audio ID dependencies, and TimeSync's deferred disposal. The full suite reports 102 passes and the retained inactive CPU Titanic deformation failure (20.579636). The release build compiled. Full engine consumer migration, original coroutine/JVM lifecycle timing and end-to-end native shutdown parity remain incomplete. The packaged playable build was not refreshed.
## FileReader and ImageData conversion (2026-10-06)

Separate Rust files now preserve FileReader's requested-path, SSHome,
SSResources, then bundled-resource search order. Existing-path open failures
stop resolution, as in Java. Extracted assets replace JVM classpath streams.
FileShipResource now uses this reader for image and material palette loading.
ImageData retains source dimensions, pixel bytes and OpenGL component/type
metadata, including the default unsigned-byte type (5121).

Verification: PNG RGBA round-trip and path priority/error checks pass. Full
suite: 104 passed, 1 failed (unchanged Titanic approximate CPU deformation
regression, displacement 20.579636). This is partial conversion: image crate
replaces STB, so grayscale conversion, 16-bit decoding and non-PNG formats
require source comparison. Text decoding assumes UTF-8 rather than the JVM's
platform charset. Rust owned image bytes do not yet reproduce shared mutable
ByteBuffer positions/limits or Java hash/toString semantics. FileReaderKt
archive extraction is still pending. The previously packaged playable build
predates this source change.

## FileReaderKt resource extraction conversion (2026-10-06)

`src/file_reader_kt.rs` translates resource installation for directories and
JAR archives into a separate Rust module. It preserves destination basename
selection for the archive root, per-path filters, relative destinations,
parent creation, overwrites, and continuation after individual read/write
errors. The default filter accepts directories too, preserving their read
errors rather than silently treating them as files. Structured reports retain
errors for callers rather than Java's immediate stack-trace printing.

The JAR adapter reads central directory lengths (including entries whose local
headers use data descriptors), handles stored and deflated files, and verifies
length and CRC. All 13,005 files in the supplied original JAR decode correctly.
Extracted icon PNGs match the previously extracted archive byte-for-byte.
Directory extraction tests cover default directory errors and overwrites.
Full suite: 107 passed, 1 failed; the existing Titanic approximate CPU solver
regression is unchanged (20.579636 displacement).

Remaining: URI parsing/percent encoding, ZIP64 and other compression methods,
implicit directory enumeration and Java ZIP filesystem traversal order require
further conversion. No active source call site was found outside this utility,
so installation has not been added to game startup speculatively. Full game
conversion and gameplay fidelity remain incomplete. The playable package is
not refreshed by this conversion step.

## Shared physics data-holder conversion (2026-10-06)

Added separate Rust translations of GLDataHolder, TypedDataHolder,
FloatDataHolder and UInt8DataHolder. Source format tables, scalar types,
component sizes and nearest/clamp-to-border/no-mipmap settings are retained.
Float allocations start at zero and use padded WGSL vec4 storage. The active
ForceDataHolder (two components) and WaterDataHolder (four components) now
use this shared allocation implementation, including water state planes.
UInt8DataHolder preserves the source default single component and null initial
data rather than assuming that unspecified OpenGL texture memory was zero.

Source texture configuration lambdas were recovered with CFR and compared.
Tests cover descriptors, zero initialization, dimensions, state planes and
integer allocation metadata. The descriptor is an explicit storage-backend
adapter, not an actual OpenGL texture object: GPU texture lifecycle, GLSL
builder types and UInt8 GPU binding integration remain pending. Shader bounds
checks implement border behavior separately and still need complete auditing.
Full conversion is not complete; the known Titanic CPU regression remains.

## Physics pass contract conversion (2026-10-06)

Eight source files now have separate Rust counterparts under `src/passes`:
Pass, StatefulPass, InitializablePass, InitializableStatefulPass, CustomPass,
DirectPass, ProviderPass and TargetPass. Source defaults are no-op uniform
setters. Composites render in child order and forward arguments only to
stateful children. Provider setup invokes only initializable children;
Target setup instead renders its distinct setup list. Source bindings use
ascending units and unbind in ascending order. Targets preserve viewport
capture, framebuffer bind, attachment selection, target viewport, render,
unbind and prior viewport restoration. Target uniform forwarding visits
setup passes before runtime passes.

Tests exercise repeated callbacks, ordering, capability filtering, scalar and
matrix arguments, empty/default passes, attachment selection and viewport
restoration. Binding and framebuffer traits are explicit backend adapters;
these generic source contracts are not yet wired into the active GPU solver.
StandardPass, StencilPass, PassBuilder, GPU framebuffer creation and source
shader builder translation remain pending. The active solver's normal
iteration/final-pass schedule matches the source loop comparison; its existing
settings clamp prevents division by zero where the Java source would throw.
Full runtime/stencil and shader parity remain unproven. The playable package
is unchanged by this source conversion step.

## StandardPass and StencilPass conversion (2026-10-06)

Added separate Rust modules for StandardPass and StencilPass, plus an explicit
shader-pass backend contract. Constructor sampler assignment follows source
list order; StandardPass forwards output binding names, while StencilPass has
no color output bindings. Rendering enables stencil testing, captures/apply
state, starts the shader, draws fullscreen, stops the shader, restores state,
and disables stencil testing. StencilPass additionally disables all color
writes before state capture and enables all color writes after restoration,
matching the source's unconditional behavior.

Uniform forwarding includes source location lookup and program start/stop,
with float/integer arity constrained to 1..=4. Invalid arity leaves the program
started as the source exception does; Rust currently represents that exception
as a panic. Matrix transpose flags and all 16 values are forwarded unchanged.
Checks use an operation-recording backend, not a substitute gameplay renderer.
Full suite: 115 passed, 1 failed (unchanged Titanic approximate CPU deformation
regression, 20.579636 displacement).

Remaining: a concrete Bevy shader/stencil backend, ShaderProgram/Shader full
lifecycle and reflection, GLState implementations, source vertex shader
compilation and PassBuilder integration. These translated classes are not yet
used by active rendering/physics. This step verifies source operation contracts
only; visual/stencil parity and complete game conversion remain unproven.

## Stencil render-state conversion (2026-10-06)

Added separate Rust modules for StencilWriteMask, StencilFunc, StencilOp,
StencilConfig and UtilKt, with the GLState interface/None in their module root.
Source integer query selectors, application order, nullable components,
one-time lazy snapshots, value equality and wrapping Java hashes are retained.
Elementary data-class copies reset their lazy snapshot; config copies share
component objects while resetting their own snapshot. Config restoration
captures all default components even if its requested state has null fields.
CFR extraction of the source currentState lambdas confirmed this behavior.

Tests cover first-use capture, repeated cached restoration, component-sharing
copy semantics, query/apply order and source callback failure without finally
restoration. These contracts use a StateBackend interface, not an actual GPU
stencil implementation. Full suite: 119 passed, 1 failed; the existing Titanic
approximate CPU deformation regression is unchanged (20.579636 displacement).

Remaining: concrete Bevy stencil integration, synthetic per-field default
argument overloads, JVM singleton identity/toString exactness and Concat.
Concat's recovered lambda currently appears to call its parent's currentState
recursively; independent bytecode evidence is needed before translation.
These state modules are not yet wired into gameplay rendering. Full game
conversion and visual parity remain incomplete.

## GLState.Concat bytecode verification (2026-10-06)

The class-bytecode inspection in `tools/concat-bytecode-evidence.txt` resolves
invoke byte offset 69 to `GLState$Concat.getCurrentState`, with the receiver
loaded through the lambda's enclosing `this$0` field. This confirms the apparent
recursive snapshot in the decompilation is in the original bytecode. It does
not call each child state's currentState. Added `src/gl_state/concat.rs` for
ordered apply, cached empty snapshots and explicit nonempty snapshot failure.
Rust represents Java's eventual StackOverflowError with a controlled panic;
it does not abort the process by deliberately overflowing the Rust stack.
The inspector prints candidate byte offsets and raw hex, so it is evidence
for this examined instruction sequence, not a general JVM disassembler.

Also preserved NoState snapshot identity on the render thread and added
source partial-default constructors for StencilFunc/StencilOp: only omitted
parameters query current state, in source parameter order. Tests verify child
apply order, no child snapshot calls, identity/cache behavior and omitted-field
queries. Full suite: 122 passed, 1 failed; existing Titanic CPU deformation
regression remains at 20.579636 displacement.

Remaining: concrete GPU state integration, source constructor overloads for
StencilConfig, global JVM singleton versus Rust render-thread identity,
formatted JVM string representations, full PassBuilder and shader lifecycle
conversion. No gameplay caller of GLState.Concat was found in the decompiled
game package. Full conversion and runtime fidelity remain incomplete.

## PassBuilder conversion (2026-10-06)

Added `src/passes/pass_builder.rs` with the source outer with/build/setup
sequence, texture/direct/target steps, separate setup and runtime lists,
custom/shader/stencil pass construction, named sources/destinations, retained
shader handles, and optional shared stencil. Shared stencil allocation uses
the first source dimensions and source format 35056. Target allocation without
shared stencil uses the first destination; with shared stencil it uses stencil
dimensions. Resetting shared stencil affects later targets. TargetPass now
retains its stencil object explicitly. Shader passes returned by builder calls
retain the same object that composites execute, using shared Rust ownership.

Tests verify immediate setup before return, no direct-runtime execution during
setup, repeated runtime order, shared stencil reuse, reset, dimensions and
empty-input source error cases. Standard/stencil shader operation contracts
remain covered by their earlier backend-recording tests; builder shader paths
need additional end-to-end GPU verification. The factory requires explicit
shader, stencil and framebuffer implementations; no no-op gameplay backend
has been substituted.

Remaining: concrete Bevy factory/solver integration, GLSL-to-WGSL builder and
shader lifecycle conversion. Consuming Rust build methods differ from Java's
reusable mutable builder/list reference behavior and require a further audit.
Source shared resource and render-thread ownership adapters also remain.
Full 1:1 conversion is incomplete; this step does not refresh the playable
package or fix the retained approximate CPU solver's Titanic regression.

## Shader and ShaderProgram conversion (2026-10-06)

Separate Rust modules now translate Shader and ShaderProgram. Both integrate
with GlResource's real dependency/deferred main-thread destruction mechanism.
Shader preserves source type constants, case-sensitive file suffix recognition,
source/compile/log/status sequence, attach/detach and delete. Failed compile
returns the source message and schedules cleanup through Rust destruction;
this timing differs from JVM finalization.

ShaderProgram preserves first shader attribute bindings and last shader output
bindings before attachment and linking. It retains shader objects for deferred
detach-before-delete. Source link logs are printed without inventing a link
status check. Uniforms start the program, accept 1..=4 components, upload and
stop; arity errors leave the program started. Matrices preserve transpose and
16 values. Validation binds/unbinds the supplied VAO. Reflection preserves the
source inclusive 0..count query and 50-byte attribute name buffer request.
Display strings use the source formatting.

Five new checks verify compilation errors, lifecycle, constructor call order,
uniforms, reflection and validation with operation-recording GPU adapters.
Full suite: 130 passed, 1 failed (unchanged Titanic approximate CPU deformation
regression, 20.579636 displacement). The concrete compiler/linker/VAO/reflection
backends, render-thread enforcement, current-context default constructor and
Bevy shader-pass integration remain pending. These translated classes are
not yet compiling gameplay shaders. Full conversion and 1:1 behavior remain
unproven; the playable package is unchanged by this step.

## GLSL builder/reference conversion (2026-10-06)

Added separate Rust modules for GLBuilder, GLReference, GLType, GLFloat,
GLVec4, GLIVec2, GLSampler2DType and GLSamplerKt. The supplied source builder
filter/map/build methods are empty and remain empty. Its program context
preserves holder-identity lookup scopes, repeated declarations, global/local
text, output names and little-endian base-52 variable suffixes.

Constructor bytecode in `tools/gl-builder-context-bytecode-evidence.txt`
confirms IDX initialization invokes globalVar before setting inc to 1.
The initial declaration is therefore `ivec2 v_ = ivec2((gl_FragCoord.xy));`.
Subsequent names begin at suffix b. Tests check this field order, exact sampler
fetch/declaration text, shared input names, per-pass output names, counter
wrapping and digit order. The 217-file inventory was compared with the actual
com/wicpar source tree and contains all 217 paths.

Full suite: 133 passed, 1 failed (unchanged Titanic approximate CPU deformation
regression at 20.579636). These are source declaration utilities, not a complete
shader compiler. Numeric/vector/sampler family class hierarchy and compile-time
constraints remain partial; the shared Sampler adapter is not a claim that
individual sampler Java classes have all been converted. HolderIdentity is a
Rust adapter pending integration with actual TypedDataHolder instances.
GLVectorUtilKt, GLOpsKt and concrete GPU translation remain. No game-package
GLBuilder call site was found. Full 1:1 conversion remains incomplete.

### Vector source conversion — 2026-10-06

Converted 52 Java vector hierarchy files to dedicated Rust modules: component and size markers, ascending/descending marker inheritance, scalar-family vector traits, relatives, and all twelve concrete vectors. Retained source constants including GLVec1/GLIVec1/GLUVec1 names ending in vec2 and uppercase Y/Z/W components. The sampler helper now requires a TWO-sized integer vector, replacing its previously unconstrained index argument.

Rust traits and zero-sized markers represent the source abstract classes and singleton types; JVM object identity and Java Vector collection behavior are not implemented by these value markers. This infrastructure is not yet connected to a concrete GPU shader-builder backend. This does not complete the game or broader JAR conversion.

Regression result: 136 passed, 1 failed. The retained Titanic CPU solver stability regression still reports hull displacement 20.579636; this failure is not fixed or suppressed. New checks cover all twelve vector sizes/names/relatives and component constants/marker inheritance. The playable package predates this conversion.

### GLSL operations and vector helper conversion — 2026-10-06

Added dedicated GLOpsKt and GLVectorUtilKt Rust modules. All eight source arithmetic/bitwise expression constructors preserve operand type and exact parentheses/operator text. Swizzles of length two/three/four, component access, scalar casts, and equal-size vector casts select the source relatives and target type names. Scalar casts accept the existing Float/Integer family carriers; broader JVM Number and reference identity semantics remain outside these Rust value adapters.

GLBuilder now constructs IDX using the translated swizzle/cast helpers. This corrects the previous hardcoded lowercase xy to the source's xY, as confirmed by GLBuilder's X.INSTANCE/Y.INSTANCE call and Y's uppercase component constant. The source utility can consequently emit GLSL expressions invalid for compilation; these literals are retained for fidelity. This does not change active WGSL gameplay shaders, and a concrete GPU builder backend remains pending.

Verification: all nine builder tests pass. Full regression suite: 139 passed, one retained Titanic CPU solver stability failure (displacement 20.579636). This is partial conversion progress, not completed game parity.

### Bitmap enum source conversion — 2026-10-06

Added five separate Rust modules for IntEnum, BitmapEnum, EnumSet, StaticEnumSet, and MutableEnumSet. Source masks, reversed containsAll relation, mutable change reporting, bit-count sizes, constructor filtering/map snapshots, retained enum array references, same-class bitmap/array equality, and JVM wrapping hash arithmetic are represented. Caller-provided enum identity hashes are required for JVM-compatible hash values.

Mutable iterator next does not advance after returning a value, hasNext compares the current bit with a construction-time highestOneBit maximum, and remove only clears the current bit. Confirmed in tools/enum-iterator-bytecode-evidence.txt. Bounded tests preserve repeated next calls, source remove behavior, sign-bit masks, and the source empty-iterator behavior of next returning the first value even when hasNext is false.

Incomplete fidelity: Java collection bridge methods, typed toArray behavior, mutable toString traversal, reified enum-class discovery, JVM exceptions, and active integration remain pending. The Rust iterator reports pathological zero-shift loops as an error instead of hanging forever; this is an explicit divergence. No active non-utility Java EnumSet call sites were found in the supplied com/wicpar tree.

Also reconciled eleven existing scalar/sampler modules whose inventory entries had not yet been recorded. Full regression suite: 143 passed, one retained Titanic CPU stability failure (20.579636 displacement). The overall 1:1 conversion remains incomplete.

### Buffer and model draw conversion — 2026-10-06

Added dedicated IDrawable, VBO, VAO, and ShadedModel Rust files. Added SourceModel in model.rs alongside the existing Bevy Mesh adapter. VBO preserves Byte/Short/Int/Float/Double GL type constants, capacity-based size metadata, remaining-window upload, bind/unbind sequence, and GLResource deferred deletion. VAO attachment leaves the element buffer bound exactly as in the source; attribute setup uses source type, zero stride/offset and source unbind ordering.

SourceModel retains VAO and VBOs, has an independent Resource with empty cleanup, and draws using the index buffer capacity and unsigned-int draw type 5125. ShadedModel validates its VAO, registers model lifetime as a shader dependent, and implements render, renderWith and renderShaderless ordering. GLResource/ShaderProgram now expose dependent registration for this path.

Two recording-backend tests verify all five buffer types, sliced upload versus capacity metadata, deferred destruction, VAO setup, indexed draw, shader wrapping, and shader-dependent model closure. Full suite: 145 passed, one existing Titanic CPU stability failure (displacement 20.579636).

Fidelity gaps: these required backend interfaces do not yet have a concrete Bevy render implementation; the existing Bevy mesh path remains active. Current-context default constructors, Java NIO unsupported buffer and exception handling, source overload/default bridges, and complete UVModel/ShadedModel inheritance integration remain pending. This is conversion infrastructure progress, not verified game parity or a refreshed playable package.

### Framebuffer source conversion — 2026-10-06

Added dedicated FramebufferTarget, FBO, RenderBuffer and TexturedFBO Rust modules. Preserved source framebuffer/render-buffer constants, storage dimensions/format, attachment ordering, the 24-color-target limit, exact error-check labels/order, per-unbind hooks, viewport save/restore, source failure behavior without finally cleanup, and GLResource deferred deletion. Attachments retain a shared fixed-length boxed array; constructor bindings are a snapshot while subsequent draws use the current target array. RenderBuffer implements the existing StencilTarget contract, and TexturedFbo implements TargetBinding for translated passes.

Three recording-backend tests cover construction and draw call order, depth-stencil attachment, hook timing, replacement target arrays, the target limit, failed callbacks leaving bound state, storage setup and dependency cleanup. Full suite: 148 passed, one existing Titanic CPU solver stability failure (20.579636 displacement).

Remaining fidelity/integration: concrete GPU backend implementations, current-context defaults, RenderFBO resize/allocation inheritance and Texture framebuffer attachment integration are not complete. TexturedFBO itself has no resize method in the source; that behavior belongs to RenderFBO. Rust Result/panic and earlier drop cleanup replace JVM exception/finalizer timing on constructor failure. Active game rendering remains on the existing Bevy path. This is partial conversion progress, not complete 1:1 parity.

### Texture source conversion — 2026-10-06

Added dedicated Texture, Texture1D and Texture2DArray Rust modules and a SourceTexture2D path in texture_2d.rs. Preserved source unit binding/reset, integer parameters, dimension-specific upload metadata, nullable image storage, configuration callbacks after upload/unbind, mipmap generation timing, framebuffer attachment/error checks, float readback formats and GLResource deferred deletion. The compiled default configuration callbacks are empty.

Source Texture2DArray binds target 35866 but calls its 3D image upload with target 3553; this source constant is retained. Texture.downloadFloats also uses 3553 independently of the instance target. Three recording-backend tests cover those constants, all dimension constructors, nullable/retained buffers, callback/parameter ordering, component formats, readback failure binding and context cleanup. Full regression suite: 151 passed, one retained Titanic CPU stability failure (20.579636).

Remaining gaps: generic reified loadTexture and default overload bridges, Java NIO cursor/limit/native-endian buffer semantics, concrete Bevy backend, and RenderFBO source resize integration. SourceTexture2D.from_image currently copies ImageData bytes, so Java ByteBuffer object identity is not preserved. Uploaded Rust PixelBuffer storage retains shared bytes but does not yet carry NIO position/limit. Active Bevy textures continue through the existing adapter. Full 1:1 conversion remains incomplete.

### RenderFBO source allocation and resize — 2026-10-06

Added SourceRenderFbo in render_fbo.rs, connecting the converted FBO, TexturedFBO, Texture2D and RenderBuffer resource paths. Preserved RGBA source format 6408, unsigned-int data type 5125, caller internal format, optional depth-stencil format 36013 and attachment 33306, pre-allocation target limit, constructor ordering, and source empty ranges for negative target counts.

The mipmap hook is enabled after superclass construction, matching the source field's initial false value. Resize allocates one new texture per current target-array entry, attaches each while the old array remains installed, then replaces the array and optional depth buffer. Source onUnbind therefore regenerates old-array mipmaps during attachment and new-array mipmaps afterward. TexturedFbo hooks now receive the current array, and framebuffer targets expose texture capability for the source instanceof filter without treating render buffers as textures.

Two recording-backend tests verify constructor mipmap timing, old/new array mipmap order, externally retained old array identity, depth replacement, draw-time mipmaps, negative/over-limit counts, disabled mipmaps, and current array length differing from the original number. Full regression suite: 153 passed, one existing Titanic CPU stability failure (20.579636 displacement).

Remaining integration/fidelity: concrete Bevy render backend, ScreenFBO source window callback lifecycle, Java constructor/default bridges and garbage-collection timing. Rust releases replaced attachments when their last owner drops rather than JVM finalization. Active rendering still uses the existing Bevy screen/mipmap adapter. This does not complete 1:1 game conversion.

### Memory and output utility conversion — 2026-10-06

Added separate MemUtil and TeeOutputStream Rust modules. Memory wrappers copy Float/Int/Byte arrays, expose capacity/position/limit/remaining state, preserve float bits and native-order Float/Int byte encoding, and decode ordered native NUL-terminated string pointers. Confirmed BufferUtils native byte order from the bundled org/lwjgl/BufferUtils.class. The source model array constructor now uses these wrappers through VBO Float/Int constructors, retaining full capacity metadata and remaining upload ranges.

TeeOutputStream forwards all five source operations (single byte, byte array, offset/length write, flush, close) to the original output first and the tee second. First-output errors prevent the second call; second-output errors retain the first side effect. Rust Write is also supported, but explicit source close remains necessary. Six new tests cover wrapper state/copying, native float bit patterns, pointer strings, output ordering/failures, and model array constructor integration.

Full regression suite: 159 passed, one retained Titanic CPU stability failure (20.579636 displacement). Remaining gaps include actual JVM-equivalent direct memory allocation, NIO marks/shared buffer views and identity, malformed UTF8 decoder parity, concrete stream implementations and Main console/file redirection. NativeBuffer uses Rust Vec storage and explicit validation errors; raw-pointer string decoding requires the documented readable-pointer preconditions. Conversion remains incomplete.

### Input handler interface conversion — 2026-10-06

Added separate InputHandler and InputHandlerDelegate Rust modules. All fifteen source callback defaults return the incoming blocked flag; beforeEvents/afterEvents default to no action. Delegate callbacks visit every handler in order, pass the same original blocked flag and window/event arguments to each, collect their results and return whether any result is true. Empty delegates return false even when the incoming blocked flag is true. Event hooks visit all handlers in order.

Preserved source argument order including position(ypos,xpos), size(height,width), framebufferSize(width,height), full key/scancode/action/mods values, f64 cursor/scroll coordinates and drop payloads. Three tests cover every callback payload, all defaults, no short-circuiting, original blocked propagation, empty delegates and hook order. Full suite: 162 passed, one existing Titanic CPU stability failure (20.579636 displacement).

These source interfaces do not themselves store key/mouse/window state; that belongs to Window. Generic W temporarily represents the yet-unconverted source Window. Source callback registration/list mutation and concrete Bevy event integration remain pending; Rust's borrowed handler slice differs from Java iterable mutation/exception behavior. Active camera/tools continue through the current Bevy systems. Full conversion remains incomplete.

### Source Window conversion

Added src/window.rs for Window.java: fifteen typed callback lists, callback-first handler folds with each stack initially unblocked, source position/size argument ordering, iconify callback-only dispatch, native property queries, relative mouse coordinates, clear/draw/swap/event hook sequencing, default zero clear color and parent resource closure. Refresh redraw captures the original window as the Java closure does. Three focused checks pass for dispatch, query coordinates and frame/resource cleanup order.

This is a partial class conversion. Native window creation/hints, GLContext setup, icons, viewport callback and concrete Bevy backend remain pending. The backend currently returns a batch of events after polling rather than invoking callbacks during native polling. Rust callback-list borrows differ from Java concurrent modification exceptions. The source refresh closure creates a cycle; Rust Rc requires explicit callback clearing, unlike JVM garbage collection. Active gameplay continues through the existing Bevy systems. Full 1:1 conversion remains incomplete.

### Window constructor and synthetic viewport callback

Translated Window.java's constructor operation sequence through required WindowConstructorBackend operations: default size queries before resource registration, hidden/resizable/debug hints, macOS context/profile/retina hints, null monitor/share handles, exact creation-failure text, fifteen native callback registrations in source order, framebuffer viewport callback installation, integer centering, current-context setup, context retrieval, swap interval zero and final show. Default size truncates monitor dimensions multiplied by 0.8. Two constructor tests pass for default sizing/order and macOS failure sequencing.

Decompiled the bundled Window$2$16.class separately with CFR and added its own Rust file, src/window_framebuffer_callback.rs. Its callback forwards glViewport(0,0,width,height), including zero and negative dimensions without invented clamping. This supersedes the earlier statement that constructor/viewport behavior was entirely untranslated. Concrete Bevy/native construction, GLContext implementation, icons and integration between this constructor interface and SourceWindow lifetime remain pending; passing operation-recording tests does not prove native backend parity.

### GLContext conversion and current-context resource construction

Added src/gl_context.rs for GLContext.java. A thread-local cache retains the first context and returns it on later requests, even for another window or after closure, matching the source. Construction registers the window dependency before creating capabilities, prints GL Version from query 7938 (null becomes the text null), and prints only true primitive-boolean public fields in backend-provided reflection order. Capability data retains an opaque native object by identity. Cleanup defers backend GL destruction through ResourceRuntime and does not clear the thread-local cache.

GlResource now exposes the source default current-context constructor and retains the full context strongly. SourceWindow now has the source late-initialized glContext getter/setter and exposes its resource dependency for context creation. Checks cover first-context reuse, per-thread missing-context failure, capability identity/report order, closed-context retention and GPU-before-context-before-window cleanup. Concrete native capability reflection, LWJGL function resolution, Bevy rendering integration and JVM exception/finalization semantics remain pending. The constructor backend is still not connected to the active game; this is conversion progress, not proof of 1:1 parity.

### GLFW and Monitor conversions

Added separate glfw.rs, monitor.rs, monitor_video_mode.rs, glfw_monitors.rs and monitor_scale.rs. GLFW registers its resource, installs the print-error callback, initializes GLFW, enumerates monitor buffer entries from index zero through capacity, constructs each monitor, then queries/constructs the primary monitor separately. Shutdown queues termination followed by clearing the error callback. Initialization and missing-monitor/video-mode failures preserve source text and stop subsequent queries.

Monitor preserves native current-mode identity, nullable available-mode enumeration, construction-time size/scale and fresh physical-size queries. VideoMode copies channel/refresh values and retains a shared mutable size in default copies; equality and Java hash read current size values. The vector hash seed/order was checked against bundled JOML Vector2i bytecode decompilation. Three GLFW checks and a VideoMode identity/hash check were added. Empty nonnull monitor buffers still allow primary-monitor construction, as the source does.

Concrete native/Bevy monitor integration, PointerBuffer limit/error semantics, native structure lifetimes, JOML/locale-dependent VideoMode string formatting and JVM exception/finalization behavior remain incomplete. Rust resource cleanup after failed initialization follows last-owner disposal rather than waiting for Java GC. These source modules are not yet the active Bevy window backend; full 1:1 game parity remains unverified.

### ExecutorKt conversion

Added src/executor_kt.rs. glfwSafe and blockGlfwSafe execute the supplied closure immediately on the caller and preserve borrowed values/return results/panics. getGlContext retains a singleton dispatcher named GL. Bundled ThreadPoolDispatcherKt, ThreadPoolDispatcher and PoolThread were separately inspected: one worker is created lazily, named GL, marked daemon, and scheduled executor shutdown accepts no new work while retaining existing delayed tasks. Rust detached worker lifetime matches process-exit behavior; accepted tasks run sequentially and panic results do not kill the worker. Three checks cover wrappers, singleton/thread identity/order, delayed shutdown and panic recovery.

Java shutdown policy evidence: https://docs.oracle.com/javase/8/docs/api/java/util/concurrent/ScheduledThreadPoolExecutor.html (execute-existing-delayed-tasks default true). Full kotlinx coroutine context/continuation/cancellation semantics, rejection fallback, periodic scheduling, Java interruption and native integration remain pending. Source game code has no getGlContext call outside its declaration in the supplied decompile; TimeSync uses a separate dispatcher and still uses the existing Bevy callback adapter. This does not establish complete library or game parity.

### Source TimeSync background reporting connected to Bevy

Added SourceTimeSync in time_sync.rs and a separate time_sync_reporter.rs for TimeSync$1.class. Source construction registers its resource, initializes shared sample lists, launches an immediate background report and then initializes the atomic last timestamp. Update exchanges that timestamp before adding source resource/speed samples and sleeping the truncated nonnegative millisecond offset. Reporting replaces each list with a fresh list and averages the retained old list, preserving references held by callers. The worker calls the callback, schedules a 1000ms delay and checks the continuation flag again. Resource cleanup turns the flag off only when deferred cleanup runs; it does not cancel the pending delay.

Active Bevy startup now installs SourceTimeSync, and sync_frame uses its pacing and background averages. Title changes travel through a channel to Bevy's main thread. Existing standalone sample helpers remain for conversion checks, but no longer collect/report active runtime samples. Checks cover initial background report, nanosecond/11ms pacing, old-list identity, replacement setters, deferred stop and active startup/frame wiring.

Remaining fidelity: source callback writes the native window title from its worker while Bevy applies messages on the next main-thread frame; original TimeSync worker uses JDK default thread names, while this worker is named TimeSync. Mutex protection changes Java ArrayList race/exception behavior. Original coroutine continuation/cancellation ABI, Java interruption and exact scheduling latency remain unconverted. CFR inspection of TimeSync$Companion$dispatcher$1.class verified its default-thread-factory plus daemon flag. This supersedes earlier statements that all TimeSync reporting occurs on the main thread; full 1:1 game parity remains unfinished.

### OpenAL device/context/error conversions

Added al_device.rs, al_context.rs, al_context_start_reference.rs and al_util_kt.rs. ALDevice class state retains its default instance, querying alcGetString(0,4100) before opening it; explicit constructors initialize that default first, then open the requested nullable name. Zero handles and failed close/make-current boolean results remain unchecked as in the source. Context creation passes device ID and attributes [0] before registering the resource dependency; context ownership retains its device.

ALContext preserves a process-global current pointer, native make-current before pointer assignment, stop clearing whichever context is current, capability initialization only while alcCapabilities is absent, source ALC.createCapabilities(context ID) argument, and AL creation using the same ALC capability object. Cleanup destroys the context without clearing current. Late-initialized capability getters, setters and the separate mutable property reference are translated. Error checking consumes one AL error and maps all five named errors plus unknown numeric errors to exact source strings.

Checks cover class initialization and default identity, cross-thread global context access, capability identity/order, manually supplied ALC capabilities skipping AL initialization, late-property failures, context-before-device cleanup, null names/zero IDs, closed-default reuse and exact AL error messages. Concrete OpenAL/Bevy backend integration is pending; active music still uses the existing audio adapter. Rust mutexes/typed setters alter Java race/type-cast/exception semantics, and reflection owner is represented by its class name rather than a Kotlin KDeclarationContainer. Per-backend AlDeviceClass values model loaded-class state for checks; native code uses the global class accessor. This is not complete audio or 1:1 game parity.

### ALBuffer and ALSource conversions

Added separate al_buffer.rs and al_source.rs. ALBuffer preserves generation before resource registration, empty dependency list, case-sensitive ogg extension handling (including hidden .ogg filenames), two stack-push/int-slot allocations, decode/read/pop/pop/upload/free order, mono16/stereo16/unsupported format values and null decoded PCM skipping upload/free. Input-memory decode reads the buffer's remaining range without advancing its cursor. Buffer metadata getters use source AL constants; samples repeat channel/bit queries and preserve signed integer overflow, while duration keeps source IEEE division by frequency. Bundled ALBuffer bytecode was saved in tools/al-buffer-bytecode-evidence.txt for decompiler control-flow review.

ALSource preserves native ID-only buffer attachment, ordered batch queues including duplicate and empty arrays, processed-count requery after each unqueue, playback controls, state comparisons, sample/seconds offsets, volume and deferred deletion. Values are forwarded without added clamping or finite checks. Buffer attachment/queueing does not create resource dependencies; closing a buffer does not close a source. Three checks cover decode formats/order, case/arithmetic metadata, and distinct-ID queue/control/lifetime behavior.

Concrete STB/Vorbis decoding and OpenAL playback backends remain unconverted; active music still uses its existing adapter. Native direct/short buffer views and allocation identity, JVM stack-failure/type/exception semantics and concurrency remain pending. Native PCM free is an explicit backend operation; the Rust sample Vec additionally owns its adapter copy. These tests prove translated operation contracts, not actual audio backend equivalence or complete 1:1 game parity.

### MusicPlayer source audio behavior and startup correction

SourceMusicPlayer now uses the converted ALSource operation contract, retaining a shared mutable track map and remove-on-play queue. Construction creates the source before copying map keys and selecting the first track. Playback preserves stop/unqueue/optional-buffer-queue/pause-or-play order, including unknown track names. Empty repeating maps remain unpaused; Main.java populates its original map and calls nextTrack. Active discovery and Simulation now preserve that unpaused startup instead of forcing pause. This supersedes earlier startup comments claiming the original pauses its empty initial playlist.

Separate music_player_volume_reference.rs and music_player_progress_reference.rs translate the two drawUI property references. Three new checks cover constructor/native operation order, pause state queries, missing buffers, shared-map population, repeat exhaustion, shuffle removal, automatic progression, numeric conversion and active startup. Nine music checks pass. Full suite: 188 passed, one retained failure in titanic_cpu_solver_does_not_tear_without_user_damage (hull deformation 20.579636). That CPU solver failure is unresolved.

The active game still uses the Bevy music adapter. Source icon loading/configuration, exact Kotlin RNG sequence, UI drawing, native OpenAL/Vorbis integration and JVM reflection/exception behavior remain pending. Backend-contract checks do not establish actual audio equivalence or full game parity.

### Additional user-supplied reverse engineering references

Reviewed [Hex-Rays IDA MCP](https://github.com/HexRaysSA/ida-mcp), [Ghidra MCP](https://github.com/bethington/ghidra-mcp) and the [Rust rewrites and ports guide](https://github.com/trevaintdead/ai-game-modding-guides/blob/main/guides/03-rust-rewrites-and-ports.md). The guide reinforces original-game comparisons and explicit implemented/missing behavior tracking. Its embedded starter prompts are reference material, not changes to the user's conversion request. IDA MCP requires an installed IDA environment; Ghidra MCP requires a compatible Ghidra installation and its bridge. Neither was installed or connected during this review. The supplied JVM class files and CFR/bytecode inspection remain the primary evidence for Java behavior; these tools are candidates for later native-library inspection when needed.

### Tool interface and MoveTool source conversion

Added SourceTool to tools/tool.rs with the original InputHandler inheritance and texture, activeTexture, name and update contract. SourceMoveTool in tools/move_tool.rs loads Move.png before Move2.png through FileReader and SourceTexture2D, using internal format 32856 and mipmaps. Two newly inspected class files now have separate Rust configuration modules: move_tool_texture.rs and move_tool_active_texture.rs. CFR confirms each sets MAG=9729, MIN=9987 and both wraps=33069 in that order.

MoveActivation translates the original mouse callback: only left press, unblocked and without Shift, arms a one-shot update and returns true. Other left actions clear it; other buttons retain it. Update clears active before calling the global-ship drag operation. SourceMoveTool receives that operation as a callback so the supplied global ship can be resolved on each invocation. The active Bevy move handler now uses the same activation protocol, including press/release before update. SourceWindow input callbacks otherwise inherit the translated InputHandler defaults.

Two added checks cover texture load order/getter identity/name, modifiers/blocked/other-button behavior, deferred start/one-shot update, consumption before callback panic, and first-image failure stopping construction. Existing drag checks remain. Actual native texture backend/UI integration, Kotlin object/exception identity and the original GLFW event ordering versus Bevy frame snapshots remain incomplete. This is conversion progress, not full game parity. The playable package was not rebuilt for this source-conversion increment.

Verification after active MoveActivation integration: 190 checks passed, one unchanged Titanic CPU hull-deformation failure (20.579636). Full conversion remains incomplete.

### Break/Flood/Dry input and icon callback conversion

Added class-specific SourceBreakCallbacks, SourceFloodCallbacks and SourceDryCallbacks to their existing separate Rust files, plus shared brush_callbacks.rs for identical source operations. Held-click activation preserves active on blocked/Shift/repeat presses and clears on any left release; other buttons leave it unchanged. Cursor callbacks forward f64-to-f32 mouse values to preview then action uniforms even when blocked. Camera changes invert the matrix and forward the same inverse with transpose=false. Break's size callback queries screen size before framebuffer size, while Flood/Dry use event width/height for scale; both pass reciprocal event dimensions in source order without zero guards.

Active Bevy brushing now uses separate activation state for each tool, matching separate original instances. Move clicks do not arm damage tools. A held brush continues across UI boundaries, and no added pause condition is imposed on input (GUI.java.render calls tool.update independently). Release clears only the selected tool, preserving the source's retained inactive-tool state. GPU brush dispatch timing while physics is paused still needs runtime comparison.

Six additional synthetic classes were individually decompiled and saved as Java evidence, with separate break_tool_texture.rs, break_tool_active_texture.rs, flood_tool_texture.rs, flood_tool_active_texture.rs, dry_tool_texture.rs and dry_tool_active_texture.rs. Each sets MAG LINEAR, MIN LINEAR_MIPMAP_LINEAR and S/T CLAMP_TO_BORDER, preserving values/order.

These callbacks do not complete the tool classes. Their resource/pass constructors, camera callback registration/removal, FBO replacement lifecycle and native texture/UI integration remain incomplete. glam matrix inverse versus JOML arithmetic/exception behavior and GLFW-versus-Bevy event timing require further comparison. Checks cover held-click behavior, per-tool state and size arithmetic/order including zero-size IEEE results; native callback and rendering parity remain unverified. Full suite before the per-tool-state addition: 192 passed, one unchanged Titanic CPU deformation failure (20.579636).

Final verification for this increment: 193 checks passed, one retained Titanic CPU hull-deformation failure. Native backend/tool construction and complete 1:1 conversion remain unfinished.

### Brush pass and framebuffer update translation

Added tools/brush_runtime.rs and assembled SourceBreakTool, SourceFloodTool and SourceDryTool in their separate original class files. They expose the source Tool texture/activeTexture/name/update contract and forward source window callbacks through their class-specific callback implementations. BrushPassUniforms writes into the same converted StatefulPass instances used during update.

BrushRuntime reads tool size independently for preview and action uniforms, enables blending for preview render, disables blending, and only then resolves the current ship for an active brush. Ship comparison uses retained object identity. Changed ships are assigned to lastShip before framebuffer construction; target/filter getters precede width/height and FBO creation. Rendering uses converted TexturedFbo.draw, binding positions on unit zero and the action target on unit one, rendering, then unbinding target followed by positions. Live getters are queried again for unbinding. Existing TexturedFbo implements draw-buffer, viewport and error-check order; no finally cleanup is added if preview/action rendering fails.

Three additional checks exercise actual converted Texture/Fbo/TexturedFbo operations with recording backends: complete preview/draw order, separate size-property reads, inactive/current-ship behavior, identity-sensitive FBO replacement, no replacement for unchanged identity, and source failure leaving blending or framebuffer state unchanged. Test backend helpers are shared from the existing texture/framebuffer checks. This evidence covers operation contracts, not actual native GPU output.

from_parts is explicitly an assembly API, not a completed translation of each original constructor. Source Resource registration/free, camera callback identity/registration/removal, pass factory/shader compilation, native backend implementation and connection to active Bevy remain unfinished. Replaced FBO last-owner cleanup follows the existing Rust resource adapter rather than JVM GC timing. Active Bevy brush shaders are still the separate current adapter. Full 1:1 behavior has not been established.

Verification for brush runtime/class assembly: 196 checks passed; one unchanged Titanic CPU hull-deformation failure (20.579636). Full game and bundled-library conversion remain incomplete.

### Source camera objects and brush callback lifecycle

Added SourceCamera2D in camera_2d.rs, preserving matrix/size reference identity and caller-visible mutation, signed/zero constructor dimensions, initial scale sixteen, in-place inverse followed by inverse for resize/scale, and source mutation of the supplied scale-relative Vector4. Local camera callbacks preserve receiver/owner/name/signature equality, immediate invocation after insertion, first-equal removal and LinkedList-style size/modification checks during iteration. The existing Send/Sync Bevy adapter remains separate.

Individually decompiled BreakTool$1, BreakTool$free$1, FloodTool$1, FloodTool$free$1, DryTool$1 and DryTool$free$1. Each now has a separate camera_reference/free_camera_reference Rust file with callback invocation and metadata. Reconstructed constructor/free references compare equal for the same receiver; different owners or receivers remain distinct. Reflection owner is represented by the source class name, not a Kotlin reflection object.

ResourceRuntime.allocate_local retains non-Send cleanup objects on the resource main thread; the existing deferred queue carries only a lookup key and invokes/removes the local closure during run_main. Cross-thread close still defers cleanup, and reentrant cleanup may allocate another local resource. Source brush classes can register_camera: allocate an empty-dependency resource, retain its handle, register a strong receiver callback with immediate invocation, and remove an equal callback during deferred free. Cleanup removes the camera callback only, preserving independent texture/FBO ownership as in source free().

Checks cover mutable references/relative argument, duplicate method-reference removal, insertion-before-initial-invoke, iteration modification, cross-thread close/deferred local cleanup, metadata equality, strong receiver retention and release, and zero/signed-size construction. Singular/exception behavior, JOML bit-exact inversion/scaleAround arithmetic and LinkedList structural/race ABI still require further work. Rust Rc cycles have no JVM tracing GC; explicit resource close removes camera-to-tool cycles. Local cleanup registration is restricted to the resource main thread, and callback metadata equality covers converted method references rather than arbitrary Kotlin function objects.

Original brush constructor allocation/icon/pass/uniform/subscription/FBO order is not yet implemented end to end: from_parts plus register_camera is an assembly path. The SourceCamera2D path and native brush classes are not wired into active Bevy. Thus this is additional source conversion and lifecycle evidence, not a claim of complete constructors or full 1:1 parity.

Verification after source camera/lifecycle conversion: 201 checks passed, one unchanged Titanic CPU hull-deformation failure (20.579636). Full conversion remains active and incomplete.

### End-to-end source brush constructor sequence

SourceBreakTool::new, SourceFloodTool::new and SourceDryTool::new now construct the translated source objects in original order. The empty-dependency resource is allocated before reading/uploading the normal and active icon, both using RGBA8 internal format, mipmaps and their individually converted callbacks. Preview creation uses the converted empty-source PassBuilder/direct StandardPass path and immediate setup. Action creation uses StandardPass with original input/output bindings and NoState. Exact original preview/action GLSL string contents are stored as Rust constants in each class file.

Initial uniform order preserves the class distinction: Break writes action window, preview window, preview scale; Flood/Dry write preview scale, preview window, action window. Construction then registers the camera callback, immediately writes both inverse uniforms, resolves the global ship and creates the initial FBO. Tool runtime is late initialized until that final step, matching the source callback-before-FBO sequence. BrushRuntime::from_shared reuses exactly the pass objects already supplied to callbacks. A deferred cleanup key is filled with the allocated tool's receiver identity before callback registration; resource cleanup before registration performs no camera removal.

Two added checks exercise all three actual constructors through original icon asset decoding and converted Texture2D, PassBuilder, StandardPass, Camera2D and TexturedFbo operations. Checks verify icon/mipmap count before shader creation, class-specific initial uniform order, callback-before-ship/FBO order, immediate/subsequent camera callbacks, receiver retention/release and an icon-load failure stopping before pass creation/registration. Recording shader/texture/window backends establish operation contracts, not native shader compilation or visual equivalence.

This supersedes the earlier statement that only from_parts/register_camera assembly exists. Those helpers remain available for focused checks, while new translates the constructor sequence. Concrete native graphics/pass-factory/Ship adapters and active Bevy integration remain missing. Original CameraControl/GameParameterProvider object getters and Kotlin/JVM type/reflection/exception semantics are not fully represented by the environment's window/camera/property callbacks. GLSL has been preserved in Rust constants for source-pass construction; the active Bevy WGSL brush path remains separate. Full class and 1:1 game parity are not yet established.

Final constructor verification: 203 checks passed; one retained Titanic CPU deformation failure (20.579636). Full source/library conversion and 1:1 game parity remain incomplete.

### CameraControl source object and active left-button pan

Added SourceCameraControl in camera_control.rs with retained Window/Camera2D identity and mutable lastPos reference identity. Explicit construction performs no window-size query; default construction reads screen size and creates the source camera. Specialized InputHandler methods preserve resize despite blocked input (only zero dimensions skip), scrolling around the retained window's live relative mouse with double-precision pow then float cast, right presses regardless of blocked status, unblocked left presses without a modifier check, either-button release clearing the single dragging flag, and dragging cursor updates despite blocked status. Key 45/61 press/repeat zooms around the center only when unblocked and not dragging; other callbacks inherit defaults.

BrushEnvironment now retains the original SourceCameraControl object instead of a separate window/camera pair. All three brush classes expose camera_control() returning the same object and resolve its window/camera for construction and deferred cleanup. Constructor checks also verify that identity. The active Bevy adapter now starts a left-button drag when Shift causes the selected tool to decline the scene click, pans while the shared dragging state is set, and clears it on either release. A new active-system check verifies displacement and release.

Two source-object checks cover constructors/getter aliases, lastPos external mutation, button/modifier/blocked conditions, float cursor deltas, window query order, double pow scaling, press/repeat keys, zero and signed resize dimensions and inherited focus behavior. The native source path is still separate from the active Bevy adapter. Active mouse/keyboard/wheel event ordering, pixel-scroll conversion, numpad/reset extensions, initial sea-origin offset, exact JOML arithmetic and JVM exception ABI remain differences requiring further work. Source CameraControl is not proof of whole-game input parity.

Final camera verification: 206 checks passed; one retained Titanic CPU hull-deformation failure (20.579636). Five focused camera checks passed. Full source conversion, native integration and 1:1 game parity remain incomplete. The playable package was not rebuilt for this increment.

### Source parameter identity and brush provider getters

Added SourceGameParameterProvider in game_parameters.rs alongside the active adapter. The source object uses signed i32 iteration counts, mutable Rc/RefCell vector references, the complete 22-field constructor, getters/component methods, scalar/waves setters, constructor default mask and shallow copy default mask. WaterColor has no replacement setter, matching the final source field, but its returned vector is mutable and shared. Default alpha is exactly 0.75; the active adapter default was corrected from 191/255 to 0.75. Reference copies retain vector identity; selected constructor-default vector bits allocate new vectors.

GameParameterProviderKt now has its own game_parameter_provider_kt.rs with retained-object getter/setter. This local source implementation is confined to the engine thread; thread_local storage does not yet reproduce JVM process-global cross-thread visibility. Equality/hashCode/toString and glm vector object semantics, null/default-constructor-marker ABI and the full active Bevy parameter/UI integration remain pending. The translated synthetic APIs accept typed source objects rather than nullable JVM argument slots.

Source Break/Flood/Dry constructors now retain the supplied parameter object and expose its getter. Their runtime queries that retained object's tool field independently for preview/action. Replacing the global provider after tool construction does not redirect existing tools, matching the source final field. The from_parts assembly path now also retains the supplied provider and binds its runtime settings.

Four added checks cover signed/unrestricted setters, defaults, getter/component vector aliases, shallow copies/default masks, provider replacement and all three brush getter identities and retained size after global replacement. Full verification: 210 passed; one unchanged Titanic CPU hull-deformation failure (20.579636). Concrete native adapters and full 1:1 parity remain unfinished; no playable rebuild was made. Temporary PowerShell editing scripts were removed; all runtime implementations are Rust files.

### Parameter equality/hash evidence and GUIKt translation

Inspected the supplied glm Vec2.class and Vec4.class with CFR, and the original GameParameterProvider hash/equals bytecode. Added explicit equals/hash_code methods to SourceGameParameterProvider. Scalar floats use canonical Float.floatToIntBits for Float.compare equality and hashing; glm vector equality uses primitive component comparisons. Source object self-identity short-circuits equality, while equal vector references do not shortcut glm NaN comparisons. Wrapping i32 arithmetic preserves Java overflow. The original glm signed-zero equality/hash inconsistency is intentionally retained, and the Rust object does not implement Eq/Hash with stronger guarantees.

A small JVM oracle invokes the unmodified classes in sinkingsimulator-4.0-all.jar without requiring a Java compiler. Its bytecode and output are local evidence under C:/Users/Alto/Desktop/Decompile/vector-reference. Original default hash=-975189733; scalar NaN hash=-1642084069; scalar negative-zero hash=1172293915 versus positive-zero=-975189733; vector negative-zero hash=-67122917 versus positive-zero=2080360731. Rust checks reproduce those values and the JVM equality results. CFR references for glm vectors are supporting evidence, not a completed conversion of the entire glm library. Parameter toString/Java float formatting and JVM null/reflection ABI remain pending.

Created src/gui_kt.rs for the missing GUIKt.java. GuiKt retains tool-list/font references, default scale/index, late-initialized getters, nullable synthetic assignments/list entries and signed index failures. Its description method gets the item ID, initializes its timestamp only when absent, checks hover with zero flags, uses a strict greater-than-300-ms delay, and resets timestamps while not hovered. Tooltip wrapping is fontSize*35; text flags/end defaults are preserved. Nested source_finally operations execute pop/end cleanup on backend panics, including failure boundaries before each guard is entered.

GuiKt currently represents the original singleton as an explicitly retained instance. The required DescriptionBackend contract does not yet supply native ImGui drawing or fonts, and the active Bevy UI is not wired to this source path. JVM typed exceptions and global singleton/cross-thread semantics remain incomplete. Three checks cover aliases/defaults/uninitialized getters/indexing, tooltip timing/read order and every tooltip failure/cleanup boundary. Two additional parameter checks cover NaN payloads, signed zeros and original JVM hash fixtures.

Full verification: 215 passed; one unchanged Titanic CPU hull-deformation failure (20.579636). Conversion and 1:1 game parity remain active and unfinished. The playable package was not rebuilt for this source conversion increment.

### GUI source routing and frame/render translation

Added src/gui.rs for GUI.java. SourceGui retains Window/CameraControl and GuiKt object identities and dynamically snapshots all non-null tool handlers on each broadcast. Every handler receives the original blocked flag before results are ORed; a true result does not skip later handlers. Native mouse/cursor/enter/scroll/character/key callbacks run before querying ImGui capture. Capture suppresses delegation; remaining window callbacks and before/after-events broadcast directly.

Number-key press handling preserves action==1 and strict key>48/key<48+toolList.size conditions, including lazy list lookup. A matching index toggles back to zero, even if the incoming blocked flag is true; keyboard capture suppresses this. Shift character callbacks always consume the event. Character conversion truncates to Java's 16-bit char, and R/r performs separate live global-ship getter calls around thumbnail retrieval, close, replacement construction, global assignment, list clear and final global-ship add. Required ResetShipOperations defines each operation rather than replacing the sequence with a single reset helper.

handle checks GL error outside the source Throwable catch, then performs GLFW/GL3/ImGui new-frame, toolbox render and ImGui render inside it. render updates only the selected tool before obtaining draw data, replaces both framebuffer-scale components with the captured scale and renders. Errors in either caught region are forwarded to the required backend print-throwable operation. free performs GL3 shutdown, GLFW shutdown and context destroy sequentially with no added finally cleanup.

The current from_parts path is an assembly API, not a completed source constructor. Native Context/GLFW/GL3 creation, DPI/style/font setup, actual tool construction, Resource registration/deferred free, concrete ImGui/draw-data/font and global-Ship adapters, singleton binding and active Bevy integration remain pending. Rust panic boundaries model the original Throwable control flow but do not establish JVM exception/reflection ABI. The active Bevy adapter's selected-tool input path still differs from this source all-tool broadcast; it has not silently been marked equivalent.

Four checks cover native-before-capture/all-handler input order and getter aliases, strict key toggling/capture/repeat, Shift char truncation/reset getter order, frame/selected-tool render/shutdown order and original exception boundaries. Full verification: 219 passed; one unchanged Titanic CPU hull-deformation failure (20.579636). Full file conversion and 1:1 runtime parity remain incomplete. No playable rebuild was made for this source conversion increment.

### GUI constructor and concrete source tool factory

SourceGui::new now translates original GUI construction order through required GuiConstructionBackend and GuiToolFactory contracts. It allocates a Resource dependent on the supplied Window before context creation, wraps that window handle, initializes GLFW with callback installation enabled, then creates GL3. It queries the CameraControl window's screen and framebuffer dimensions, calculates both f32 ratios, retains the x framebuffer scale, reads the supplied window's monitor scale x, sets global GUI scale, scales style sizes and sets reciprocal framebuffer font scaling.

Break/Flood/Dry each obtain the current global GameParameterProvider separately. The null-first five-entry tool list is published only after all four tool constructors succeed. FiraSans-Regular.ttf is loaded at 18*fontscale then 12*fontscale; each returned font is published immediately, with a source null failure stopping later work. Toolbox construction follows both fonts. The backend is retained via Rc/RefCell for main-thread deferred cleanup; closing the source window closes this GUI resource, whose free sequence remains GL3/GLFW/context. from_parts remains an assembly helper without a constructed resource.

Added gui_tool_factory.rs as an adapter using the actual SourceBreakTool, SourceFloodTool, SourceDryTool and SourceMoveTool constructors. It requires the source BrushEnvironment/native texture/pass/ship bindings and retains the supplied controller/provider identities. Move icon construction directly uses the translated reader/Texture2D path and retained drag callback. This helper is additional runtime wiring, not a substitute implementation of the original tools.

Two constructor checks verify DPI arithmetic/startup sequence, separate live provider reads, tool-list and font publication boundaries, failed tool/font construction, deferred cleanup and the Window dependency. A third check constructs all four actual source tools with their original icons/names via the concrete factory and invokes registered camera callbacks. Existing source GUI input/frame and GUIKt checks remain; ten focused GUI checks pass.

Concrete native ImGui Context/GLFW/GL3, draw-data/font/monitor adapters, original Toolbox construction/rendering, global Ship adapters and active Bevy integration remain missing. Backend allocation methods are required rather than default no-ops; recording backends establish sequence evidence, not actual visual parity. Synthetic JVM constructor defaults, exceptions and tracing-GC timing are not yet fully reproduced. Resource cleanup after a failed Rust constructor uses last-owner drop, which differs from nondeterministic JVM finalization. Parameter process-global cross-thread semantics remain pending.

Full verification: 222 passed; one unchanged Titanic CPU hull-deformation failure (20.579636). Full file/library conversion and 1:1 game parity remain incomplete and active. No playable rebuild was made for this source conversion increment.

### Toolbox property references and settings bodies

Created 24 separate Rust files under src/toolbox_references for the original Toolbox property-reference classes. Each retains the source receiver and property metadata. Boxed number conversion preserves Java narrowing and saturation behavior; wrong-type/null assignments panic by source category without implementing actual JVM exception objects. Existing references keep pointing to their original provider/vector when global objects are replaced. The layer reference requires a concrete Ship LayerReceiver adapter, which remains pending.

Added SourceToolbox toolsVisible state and toolbox_settings.rs with the original Physics, Graphics, Performance and Tool Size control bodies. Defaults for drag format/power and Sea color flags were inspected from supplied ImGui classes, not inferred from screenshots. Iteration bounds read current globals at their original points; color editing keeps the mutable original Vec4 reference. SettingsBackend operations are required. Recording checks prove passed values, aliases and ordering; they do not prove native widget rendering, clamping, or visual parity. Outer windows/tabs, file reload/coroutine, ship browser, upload/editor, Music/Advanced controls and full active Bevy integration remain pending.

Full verification of this increment: 228 passed; one unchanged Titanic CPU hull-deformation failure (20.579636). Eleven focused Toolbox checks passed. The playable package was not rebuilt.

### Original Toolbox size callback

Decompiled Toolbox$render$3.class into local reference evidence under C:/Users/Alto/Desktop/Decompile/toolbox-reference and added its separate Rust implementation in src/toolbox_render_3.rs. The callback captures padding by value and toolsHeight by a shared mutable reference, reads display and position, subtracts padding and live tool height in source order, then clamps only oversized desired components. Inspection of the bundled glm Vec2 constructor proved default mask 2 copies x into y: both minimum bounds are 50*GUI scale. glm max delegates to Java Math.max; the Rust helper preserves NaN propagation and signed-zero behavior. Native callback registration and complete Toolbox rendering remain unimplemented.

Final verification including the size callback: 13 focused Toolbox checks pass; full suite 230 passed and one unchanged Titanic CPU hull-deformation failure (20.579636). This evidence does not establish complete game parity.

### Toolbox ship-browser body and original UTF-16 search

Added src/toolbox_ship_browser.rs for the original ShipScroll contents. SourceToolbox now stores the 256 zero UTF-16 code-unit filter and supports the synthetic replacement setter by retained Rc/RefCell identity. Search copies only through the first NUL; an unterminated buffer fails rather than being silently treated as a complete string. Filtering completes before any thumbnails draw. Missing default-layer textures are skipped. Frame padding truncates via the Java float-to-int conversion; both padding components are equal, and original UV/background/tint defaults are supplied explicitly.

Selection reads the global Ship, closes it, reads the global Ship again to get its camera controller, constructs from the selected retained thumbnail, publishes the replacement, clears the list, reads the global Ship once more and adds that exact object. Construction failure stops before publication/list mutation/tooltip. The required BrowserThumbnail/BrowserBackend/BrowserShipOperations contracts expose all operations without default no-ops. They do not yet provide native Ship/ImGui/texture-size/tooltip adapters or register the child window and complete outer Toolbox render.

Decompiled the bundled Kotlin CharsKt__CharKt.class to confirm its ignore-case algorithm compares original UTF-16 units, then original uppercase units, then original lowercase units (not expanded Rust Unicode lowercase strings). A small assembled CharacterOracle.class was executed against the supplied SS2/jre to capture upper/lower for all 65,536 BMP units. Evidence lives in C:/Users/Alto/Desktop/Decompile/toolbox-reference/character-mappings.txt. src/jvm_character.rs contains the sparse mappings and corresponding Kotlin search semantics. This reproduces the bundled runtime's Unicode version, handles unpaired surrogates, and intentionally does not case-fold supplementary characters as code points. This is supporting Java Character/Kotlin text behavior, not a claim that their entire library classes have been converted.

Five added checks cover every captured BMP mapping, special UTF-16 case behavior, eager filter/draw/default argument order, shared filter replacement and missing NUL failure, live global getter/controller/list identity after close and failed construction boundaries. Sixteen focused Toolbox checks and two character checks pass. Full suite: 235 passed; one unchanged Titanic CPU hull-deformation failure (20.579636). The mapping oracle check currently reads the local captured evidence path, which is not a portable packaged test fixture. File reload and constructor coroutine, other Toolbox pages/editor/upload, native bridge/Bevy integration and full game/JAR parity remain pending. No playable rebuild was made.

### dslfix helpers and refreshed playable release

Decompiled the supplied dslfix.class and added src/dslfix.rs as its dedicated Rust implementation. Texture sizing preserves f32 source dimensions scaled by GUI scale, f64 ratios, source strict bounds and repeated live content-width getters. Default maxHeight is half the current window height (not max-width), and default padding comes from the style. Centering subtracts vertical scrollbar width only when scrollMaxY>0. Tab scopes and their default masks translate finally cleanup through Rust panic boundaries; native ImGui/JVM nullable/reflection ABI and independent tab boundary checks remain pending.

The translated Toolbox browser now calls these geometry helpers directly, removing its backend placeholder sizing and centering operations. Its recording checks exercise the actual helper with explicit padding, including the resulting 194x44 thumbnail and x=400 centering. Sixteen focused Toolbox checks pass and the current optimized release compiled successfully. No new independent dslfix geometry/tab fixture suite was added in this build increment.

At the user's request, refreshed the playable Windows build at C:/Users/Alto/Desktop/Decompile/SinkingSimulator-Rust-Playable. The executable SHA256 is 9A43B46CC1838C6F04ED65FFE8161A8477A25487C6C098BDEB175D6651D0CF1D. All 293 runtime assets were copied and hash-verified; Play.bat sets the package working directory before starting the executable. A hidden startup process ran and responded, with only a controller mapping warning and no logged asset/shader/panic errors, then was stopped. build-info.json records the evidence. Interactive gameplay and visual parity were not verified. The full prior suite remains 235 passed with one CPU Titanic deformation failure; full 1:1 conversion and native GUI integration remain active and incomplete.

### Toolbox reloadFiles algorithm and synthetic predicate/comparator

Added src/toolbox_reload.rs for original reloadFiles behavior, plus separate src/toolbox_reload_file_predicate.rs and src/toolbox_reload_name_comparator.rs for its synthetic classes. Required ReloadBackend exposes the original file/Steam/resource/thumbnail operations; ReloadState holds ordered cached map/set/list entries with retained Rc identities. Scan roots begin with ./ships, and Workshop folders are queried only when Steam is running. Top-down file predicates have no extension filter. Canonicalization of the entire collected list finishes before ordered deduplication and exclusion filtering, and its errors propagate outside resource catches. Resource reuse is by canonical File key; newly failed resources print Error: message and become permanently excluded.

Bytecode evidence at reloadFiles offset 793 shows aload_0/getfield #140 (shipFiles), followed by Map.values at 797. Grouping therefore intentionally reads the previous resource map, not newly scanned newShipFiles. This preserves one-scan appearance/removal delay and an extra failed-thumbnail retry while the new map still contains its newly excluded files. Parent folder groups preserve encounter order, then exact UTF-16 ship names form subgroups. Thumbnail construction precedes lookup in the old equality-keyed collection, and the last equal retained instance is reused. Group failures print a stack trace and exclude every resource file in that group. Stable final sorting compares Java UTF-16 names; publication assigns thumbnails then files.

Five checks verify canonical alias deduplication and resource identity, temporary thumbnail construction/equality reuse, Steam branch lookup, delayed removal, persistent parse/group exclusions and retry boundaries, folder-separated same-name ships and stable ordering, canonicalization failure without publication, and UTF-16 name ordering. All five focused checks pass. Full suite: 240 passed; one unchanged CPU Titanic hull-deformation failure (20.579636).

ReloadState is currently separate from SourceToolbox's constructor and native backing services. Concrete File identity/Windows case semantics, filesystem-walk failure behavior, Steam/native resource/thumbnail bindings, coroutine scheduling and synchronized cross-thread publication remain pending. Rust Result errors model the caught Exception operations; full JVM Throwable/type/reflection ABI is not established. This does not complete Toolbox or all-JAR parity. No additional playable rebuild was made after this source-only reload increment; the previously delivered launcher remains available.

### Source Toolbox catalog filesystem/resource integration

Added src/toolbox_reload_filesystem.rs and connected ReloadState as an owned SourceToolbox catalog with a reload_files method. FilesystemReloadBackend scans ./ships relative to its explicitly supplied game working directory and receives a required SteamFolderAccess service for live running-state/folder queries; it does not silently force an offline production service. The backend uses the translated parse_resource_path, FileShipResource and ShipThumbnail implementations, and can decode the actual supplied SS2 BASE images. The constructor reload coroutine and active Bevy/native browser integration are still pending.

Inspection of original FileShipResource/ShipResource found no equals override: file resources inherit object identity. LoadedThumbnail therefore retains original Rc resources indexed by layer/type, rather than relying solely on the translated thumbnail's current value-derived equality. This identity map covers the references used by the synthesized base/material texture resource. Cached resources and equal thumbnails retain their old Rc objects; allocating fresh resources with the same values does not make source thumbnails equal. This adapter does not yet migrate all existing ShipThumbnail/resource consumers to native source object semantics.

Decompiled bundled Kotlin top-down directory state confirms directory root is returned before listing children and missing listFiles ends children when default onFail is null. Adapter traversal preserves OS encounter order and includes directories for the existing isFile predicate. Windows cache keys use retained UTF-16 units and original captured character case mappings. Canonical verbatim drive/UNC prefixes are removed using raw OsString UTF-16 units, preserving unpaired surrogates. Full WinNT File constructor normalization, security/access edge cases, canonical paths for missing files and generic JVM File equality/exception ABI remain pending; Rust read_dir partial-enumeration errors currently map to unavailable child listing. Rust resource parsing still inherits its existing non-UTF-8/extension/layer limitations.

Four focused checks pass: real temporary image maps with cache/thumbnail identity reuse and invalid-file exclusion; source-instance equality against fresh equal-valued resource objects; Workshop folder branch plus supplied SS2 ship catalog BASE-image decoding; case-mapped keys; and raw UTF-16 canonical drive/UNC prefix handling. The Workshop service in checks is a fixture, not actual Steam SDK verification. Full suite: 244 passed; one unchanged Titanic CPU hull-deformation failure (20.579636). Native Steam access, periodic scheduling, synchronized cross-thread publication, rendering/resource creation and full game/JAR conversion remain unfinished. The previously delivered playable package was not rebuilt for this source integration increment.

### Urgent CPU Titanic deformation fix and playable refresh

Fixed the retained CPU solver failure without modifying or weakening titanic_cpu_solver_does_not_tear_without_user_damage. Its spring loop previously used a fixed 36-based stiffness and normalized material masses, unlike the source force shader. source_cpu_spring_force now applies b=0.03*fps*iterations, k=750*min(effectiveMassA,effectiveMassB)*b*rigidity, source rope softness 0.001 and b*dampening*relativeVelocity. Effective node densities already computed for buoyancy are retained for spring mass/inverse-mass. The original elastic load drives break decisions, and this iteration's force contribution is applied before severing links as in the source shader. Existing deformation bounds and the original Titanic regression assertion remain unchanged.

The previously failing Titanic check now passes. Added a separate formula check for timestep/iteration scaling, effective mass, damping and rope softness. Full suite: 246 passed, zero failed. Source parity remains incomplete: this corrects the coarse CPU fallback; the normal playable game runs the existing per-texel GPU solver, whose spring formula was already source-scaled and was not changed in this fix. Whole-game physics stability/visual parity are not proven by the CPU regression.

Built a new optimized playable package at C:/Users/Alto/Desktop/Decompile/SinkingSimulator-Rust-Playable-Fixed with all 293 runtime assets hash-verified. Executable SHA256 E0B6D5756D031FCCBBD05268B8A0FB5033B5DABBE63612EE27D405DE77AABDB4. A separate hidden startup process ran and responded without logged asset/shader/panic errors and was stopped; the user's running earlier package was left running. Play.bat launches the fixed package. build-info.json records build/test/startup evidence. The broader conversion goal remains active; Toolbox constructor/coroutine work was interrupted by the user's urgent physics fix request and has not been claimed complete.
