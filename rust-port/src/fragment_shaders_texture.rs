//! Recovered FragmentShaders$texture$2.class initializer.
#![allow(dead_code)]
pub(crate) const SOURCE: &str = "\n            #version 150 core\n\n\n            in vec2 vTexCoord;\n            out vec4 FragColor;\n\n            uniform sampler2D tex;\n\n            void main() {\n                FragColor = texture(tex, vTexCoord);\n            }\n        ";
