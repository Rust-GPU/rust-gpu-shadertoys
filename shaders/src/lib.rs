#![cfg_attr(target_arch = "spirv", no_std)]

pub mod shader_prelude;
use shader_prelude::*;
pub mod shaders;
pub mod shared_data;

// Compute optimal grid layout (rows, cols) for cell count while attempting to keep the aspect ratio close to the provided aspect ratio.
fn optimal_grid(cell_count: usize, aspect: Vec2) -> (usize, usize) {
    // Handle edge cases for 0 or 1 cells.
    if cell_count == 0 {
        return (0, 0);
    }
    if cell_count == 1 {
        return (1, 1);
    }

    // The target aspect ratio (width / height). Add a small epsilon to avoid division by zero.
    let target_aspect = aspect.x / (aspect.y + f32::EPSILON);

    let mut best_layout = (1, cell_count);
    let mut min_aspect_diff = f32::INFINITY;

    // Iterate through all possible row counts from 1 to cell_count.
    // This is a simple and robust way to find the global optimum.
    for rows in 1..=cell_count {
        // Calculate the number of columns needed to fit all cells for the current row count.
        // This is equivalent to `ceil(cell_count / rows)`.
        let cols = cell_count.div_ceil(rows);

        // The aspect ratio of the current grid layout.
        let grid_aspect = cols as f32 / rows as f32;

        // Calculate the difference from the target aspect ratio.
        let diff = (grid_aspect - target_aspect).abs();

        // If this layout is better than the best one we've found so far, update it.
        if diff < min_aspect_diff {
            min_aspect_diff = diff;
            best_layout = (rows, cols);
        }
    }

    best_layout
}

#[inline(always)]
#[must_use]
pub fn fs(constants: &ShaderConstants, mut frag_coord: Vec2) -> Vec4 {
    let resolution = vec3(constants.width as f32, constants.height as f32, 0.0);
    let time = constants.time;
    let mut mouse = vec4(
        constants.drag_end_x,
        constants.drag_end_y,
        constants.drag_start_x,
        constants.drag_start_y,
    );
    if mouse != Vec4::ZERO {
        mouse.y = resolution.y - mouse.y;
        mouse.w = resolution.y - mouse.w;
    }
    if constants.mouse_left_pressed != 1 {
        mouse.z *= -1.0;
    }
    if constants.mouse_left_clicked != 1 {
        mouse.w *= -1.0;
    }

    frag_coord.x %= resolution.x;
    frag_coord.y = resolution.y - frag_coord.y % resolution.y;

    let shader_count = shaders::SHADER_DEFINITIONS.len();

    let shader_index;
    let shader_input: ShaderInput;
    let shader_output = &mut ShaderResult { color: Vec4::ZERO };

    if constants.grid == 0 {
        shader_input = ShaderInput {
            resolution,
            time,
            frag_coord,
            mouse,
        };
        shader_index = constants.shader_to_show as usize;
    } else {
        // Render all shaders in a grid layout
        // ignore shader_to_show
        let (rows, cols) = optimal_grid(shader_count, vec2(resolution.x, resolution.y));

        let cell_width = resolution.x / cols as f32;
        let cell_height = resolution.y / rows as f32;

        #[expect(clippy::cast_sign_loss)]
        let col = (frag_coord.x / cell_width).floor() as usize;
        #[expect(clippy::cast_sign_loss)]
        let row = (frag_coord.y / cell_height).floor() as usize;
        shader_index = row + col * rows;

        let cell_resolution = vec3(cell_width, cell_height, 0.0);
        let cell_frag_coord = vec2(
            (col as f32).mul_add(-cell_width, frag_coord.x),
            (row as f32).mul_add(-cell_height, frag_coord.y),
        );
        let cell_mouse = mouse / vec4(cols as f32, rows as f32, cols as f32, rows as f32);

        shader_input = ShaderInput {
            resolution: cell_resolution,
            time,
            frag_coord: cell_frag_coord,
            mouse: cell_mouse,
        };
    }

    if shader_index < shader_count {
        shaders::render_shader(shader_index as u32, &shader_input, shader_output);
    } else {
        // If the shader index is out of bounds, just return a default color
        shader_output.color = Vec4::new(0.0, 0.0, 0.0, 1.0);
    }

    let color = shader_output.color;
    Vec3::powf(color.truncate(), 2.2).extend(color.w)
}

#[allow(unused_attributes)]
#[spirv(fragment)]
pub fn main_fs(
    #[spirv(frag_coord)] in_frag_coord: Vec4,
    #[spirv(push_constant)] constants: &ShaderConstants,
    output: &mut Vec4,
) {
    let frag_coord = vec2(in_frag_coord.x, in_frag_coord.y);
    let color = fs(constants, frag_coord);
    *output = color;
}

#[allow(unused_attributes)]
#[spirv(vertex)]
pub fn main_vs(#[spirv(vertex_index)] vert_idx: i32, #[spirv(position)] builtin_pos: &mut Vec4) {
    // Create a "full screen triangle" by mapping the vertex index.
    // ported from https://www.saschawillems.de/blog/2016/08/13/vulkan-tutorial-on-rendering-a-fullscreen-quad-without-buffers/
    let uv = vec2(((vert_idx << 1) & 2) as f32, (vert_idx & 2) as f32);
    let pos = 2.0 * uv - Vec2::ONE;

    *builtin_pos = pos.extend(0.0).extend(1.0);
}
