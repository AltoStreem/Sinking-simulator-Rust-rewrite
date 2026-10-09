//! Recovered VertexShaders$none$2.class initializer.
#![allow(dead_code)]
pub(crate) const SOURCE: &str = "\n            #version 150 core\n\n            in vec3 Position;\n            in vec2 TexCoord;\n\n            out vec2 vTexCoord;\n\n            void main() {\n                gl_Position = vec4(Position, 1.0);\n                vTexCoord = TexCoord;\n            }\n        ";
pub(crate) const BINDINGS: &[&str] = &["Position", "TexCoord"];
