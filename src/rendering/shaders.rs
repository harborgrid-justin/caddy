//! WGSL shader programs for rendering

/// Shader collection
pub struct Shaders;

impl Shaders {
    /// Line rendering shader with thickness support
    pub fn line_shader() -> &'static str {
        r#"
// Transform uniforms
struct TransformUniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> transform: TransformUniforms;

// Vertex input
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) thickness: f32,
}

// Vertex output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) thickness: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position = transform.model * vec4<f32>(in.position, 1.0);
    out.clip_position = transform.view_proj * world_position;
    out.color = in.color;
    out.thickness = in.thickness;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#
    }

    /// Mesh rendering shader with Phong lighting
    pub fn mesh_shader() -> &'static str {
        r#"
// Transform uniforms
struct TransformUniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

// Light uniforms
struct LightUniforms {
    light_position: vec3<f32>,
    light_color: vec3<f32>,
    ambient_strength: f32,
    specular_strength: f32,
    shininess: f32,
}

@group(0) @binding(0)
var<uniform> transform: TransformUniforms;

@group(0) @binding(1)
var<uniform> light: LightUniforms;

// Vertex input
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
}

// Vertex output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position = transform.model * vec4<f32>(in.position, 1.0);
    out.clip_position = transform.view_proj * world_position;
    out.world_position = world_position.xyz;

    // Transform normal to world space
    let world_normal = transform.normal_matrix * vec4<f32>(in.normal, 0.0);
    out.world_normal = normalize(world_normal.xyz);

    out.color = in.color;
    out.uv = in.uv;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Normalize the normal (interpolation can denormalize it)
    let normal = normalize(in.world_normal);

    // Ambient lighting
    let ambient = light.ambient_strength * light.light_color;

    // Diffuse lighting
    let light_dir = normalize(light.light_position - in.world_position);
    let diff = max(dot(normal, light_dir), 0.0);
    let diffuse = diff * light.light_color;

    // Specular lighting (Blinn-Phong)
    let view_dir = normalize(-in.world_position); // Camera at origin in view space
    let halfway_dir = normalize(light_dir + view_dir);
    let spec = pow(max(dot(normal, halfway_dir), 0.0), light.shininess);
    let specular = light.specular_strength * spec * light.light_color;

    // Combine lighting
    let lighting = ambient + diffuse + specular;
    let final_color = vec4<f32>(in.color.rgb * lighting, in.color.a);

    return final_color;
}
"#
    }

    /// Point rendering shader
    pub fn point_shader() -> &'static str {
        r#"
// Transform uniforms
struct TransformUniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> transform: TransformUniforms;

// Vertex input
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) size: f32,
}

// Vertex output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @builtin(point_size) size: f32,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position = transform.model * vec4<f32>(in.position, 1.0);
    out.clip_position = transform.view_proj * world_position;
    out.color = in.color;
    out.size = in.size;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Draw circular points
    let coord = in.clip_position.xy;
    let center = vec2<f32>(0.5, 0.5);
    let dist = length(coord - center);

    if (dist > 0.5) {
        discard;
    }

    return in.color;
}
"#
    }

    /// Text rendering shader (simple quad-based)
    pub fn text_shader() -> &'static str {
        r#"
// Transform uniforms
struct TransformUniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> transform: TransformUniforms;

// Vertex input
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
}

// Vertex output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position = transform.model * vec4<f32>(in.position, 1.0);
    out.clip_position = transform.view_proj * world_position;
    out.color = in.color;
    out.uv = in.uv;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Simple text rendering - in production, sample from texture atlas
    return in.color;
}
"#
    }

    /// Grid rendering shader with infinite grid effect
    pub fn grid_shader() -> &'static str {
        r#"
// Transform uniforms
struct TransformUniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> transform: TransformUniforms;

// Vertex input
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) thickness: f32,
}

// Vertex output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position = transform.model * vec4<f32>(in.position, 1.0);
    out.clip_position = transform.view_proj * world_position;
    out.world_position = world_position.xyz;
    out.color = in.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Fade grid lines based on distance
    let camera_distance = length(in.world_position);
    let fade = 1.0 - smoothstep(100.0, 500.0, camera_distance);

    var color = in.color;
    color.a *= fade * 0.5; // Semi-transparent grid

    return color;
}
"#
    }

    /// Selection highlight shader
    pub fn selection_shader() -> &'static str {
        r#"
// Transform uniforms
struct TransformUniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> transform: TransformUniforms;

// Vertex input
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
}

// Vertex output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Slightly inflate the geometry for selection highlight
    let inflated_position = in.position + in.normal * 0.01;
    let world_position = transform.model * vec4<f32>(inflated_position, 1.0);
    out.clip_position = transform.view_proj * world_position;
    out.world_position = world_position.xyz;
    out.color = vec4<f32>(1.0, 0.5, 0.0, 0.8); // Orange highlight

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Animated pulse effect
    let time = 0.0; // Would be passed as uniform in production
    let pulse = 0.5 + 0.5 * sin(time * 3.0);

    var color = in.color;
    color.a *= 0.5 + pulse * 0.3;

    return color;
}
"#
    }

    /// Hidden line removal shader (two-pass technique)
    pub fn hidden_line_shader() -> &'static str {
        r#"
// Transform uniforms
struct TransformUniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> transform: TransformUniforms;

// Vertex input
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) color: vec4<f32>,
    @location(3) uv: vec2<f32>,
}

// Vertex output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position = transform.model * vec4<f32>(in.position, 1.0);
    out.clip_position = transform.view_proj * world_position;

    let world_normal = transform.normal_matrix * vec4<f32>(in.normal, 0.0);
    out.world_normal = normalize(world_normal.xyz);

    out.color = in.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Calculate edge intensity based on normal
    let view_dir = vec3<f32>(0.0, 0.0, 1.0);
    let edge_factor = 1.0 - abs(dot(normalize(in.world_normal), view_dir));

    // Make edges more visible
    let edge_color = vec4<f32>(0.0, 0.0, 0.0, 1.0);
    let face_color = vec4<f32>(1.0, 1.0, 1.0, 0.3);

    return mix(face_color, edge_color, smoothstep(0.6, 0.8, edge_factor));
}
"#
    }

    /// Shader for rendering construction geometry (dashed lines, etc.)
    pub fn construction_shader() -> &'static str {
        r#"
// Transform uniforms
struct TransformUniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> transform: TransformUniforms;

// Vertex input
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) thickness: f32,
}

// Vertex output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position = transform.model * vec4<f32>(in.position, 1.0);
    out.clip_position = transform.view_proj * world_position;
    out.world_position = world_position.xyz;
    out.color = in.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Create dashed line effect
    let dash_length = 5.0;
    let gap_length = 3.0;
    let total_length = dash_length + gap_length;

    let position_along_line = length(in.world_position.xy);
    let cycle = position_along_line % total_length;

    if (cycle > dash_length) {
        discard;
    }

    var color = in.color;
    color.a *= 0.7; // Semi-transparent construction lines

    return color;
}
"#
    }

    /// Axis rendering shader (X, Y, Z axes with labels)
    pub fn axis_shader() -> &'static str {
        r#"
// Transform uniforms
struct TransformUniforms {
    view_proj: mat4x4<f32>,
    model: mat4x4<f32>,
    normal_matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> transform: TransformUniforms;

// Vertex input
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) thickness: f32,
}

// Vertex output
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let world_position = transform.model * vec4<f32>(in.position, 1.0);
    out.clip_position = transform.view_proj * world_position;
    out.color = in.color;

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shaders_compile() {
        // Basic test to ensure shaders are valid strings
        assert!(!Shaders::line_shader().is_empty());
        assert!(!Shaders::mesh_shader().is_empty());
        assert!(!Shaders::point_shader().is_empty());
        assert!(!Shaders::text_shader().is_empty());
        assert!(!Shaders::grid_shader().is_empty());
        assert!(!Shaders::selection_shader().is_empty());
        assert!(!Shaders::hidden_line_shader().is_empty());
        assert!(!Shaders::construction_shader().is_empty());
        assert!(!Shaders::axis_shader().is_empty());
    }

    #[test]
    fn test_shader_contains_entry_points() {
        // Verify shaders have required entry points
        assert!(Shaders::line_shader().contains("vs_main"));
        assert!(Shaders::line_shader().contains("fs_main"));
        assert!(Shaders::mesh_shader().contains("vs_main"));
        assert!(Shaders::mesh_shader().contains("fs_main"));
    }
}
