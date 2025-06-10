use bytemuck::{NoUninit, Zeroable};

#[repr(C, u32)]
#[derive(Copy, Clone, NoUninit, Zeroable)]
pub enum DisplayMode {
    Grid { _padding: u32 },
    SingleShader(u32),
}

#[repr(C)]
#[derive(Copy, Clone, NoUninit, Zeroable)]
pub struct ShaderConstants {
    pub width: u32,
    pub height: u32,
    pub time: f32,

    // UI state
    pub shader_display_mode: DisplayMode,

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
