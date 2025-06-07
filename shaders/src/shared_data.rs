use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
#[allow(unused_attributes)]
pub struct ShaderConstants {
    pub width: u32,
    pub height: u32,
    pub time: f32,

    // UI state
    /// Boolean value indicating whether all shaders are rendered in a grid layout.
    pub grid_mode: u32,
    pub shader_to_show: u32,

    // Mouse state.
    pub cursor_x: f32,
    pub cursor_y: f32,
    pub drag_start_x: f32,
    pub drag_start_y: f32,
    pub drag_end_x: f32,
    pub drag_end_y: f32,
    pub mouse_left_pressed: u32,
    pub mouse_left_clicked: u32,
}
