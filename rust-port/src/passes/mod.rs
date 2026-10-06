//! Source physics pass interfaces and ordered composite implementations.
#![allow(dead_code)]
pub mod custom_pass;
pub mod direct_pass;
pub mod initializable_pass;
pub mod initializable_stateful_pass;
pub mod pass;
pub mod pass_builder;
pub mod provider_pass;
pub mod shader_pass_backend;
pub mod standard_pass;
pub mod stateful_pass;
pub mod stencil_pass;
pub mod target_pass;
