use std::rc::Rc;

use web_sys::{WebGl2RenderingContext as GL, *};

/*
vertex vbo binds to attribute 0
color vbo binds to attribute 1
*/
pub struct MyVAO {
    gl: Rc<GL>,
    vao: WebGlVertexArrayObject,
    vbo_vtx: WebGlBuffer,
    vbo_col: WebGlBuffer,
    ibo: WebGlBuffer,
    ibo_len: i32,
}

impl MyVAO {
    pub fn new(gl: Rc<GL>, vbo_size: usize, ibo_size: usize) -> Result<Self, String> {
        let vao = gl
            .create_vertex_array()
            .ok_or("Failed to create vertex array object")?;
        gl.bind_vertex_array(Some(&vao));

        gl.enable_vertex_attrib_array(0);
        gl.enable_vertex_attrib_array(1);

        let vbo_vtx = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo_vtx));
        gl.buffer_data_with_i32(
            GL::ARRAY_BUFFER,
            4 * vbo_size as i32 * 3,
            GL::DYNAMIC_DRAW,
        );
        gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 0, 0);

        let vbo_col = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo_col));
        gl.buffer_data_with_i32(
            GL::ARRAY_BUFFER,
            4 * vbo_size as i32 * 4,
            GL::DYNAMIC_DRAW,
        );
        gl.vertex_attrib_pointer_with_i32(1, 4, GL::FLOAT, false, 0, 0);

        let ibo = gl.create_buffer().ok_or("Failed to create buffer")?;
        gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&ibo));
        gl.buffer_data_with_i32(
            GL::ELEMENT_ARRAY_BUFFER,
            2 * ibo_size as i32 * 2,
            GL::DYNAMIC_DRAW,
        );
        Ok(Self {
            gl,
            vao,
            vbo_vtx,
            vbo_col,
            ibo,
            ibo_len: 0,
        })
    }

    pub fn send_data(
        &mut self,
        vtx_data: &[f32],
        col_data: &[f32],
        idx_data: &[u16],
    ) {
        self.gl.bind_vertex_array(Some(&self.vao));

        self.gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.vbo_vtx));
        let view = unsafe { js_sys::Float32Array::view(vtx_data) };
        self.gl.buffer_sub_data_with_i32_and_array_buffer_view(GL::ARRAY_BUFFER, 0, &view);

        self.gl.bind_buffer(GL::ARRAY_BUFFER, Some(&self.vbo_col));
        let view = unsafe { js_sys::Float32Array::view(col_data) };
        self.gl.buffer_sub_data_with_i32_and_array_buffer_view(GL::ARRAY_BUFFER, 0, &view);

        self.gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&self.ibo));
        let view = unsafe { js_sys::Uint16Array::view(idx_data) };
        self.gl.buffer_sub_data_with_i32_and_array_buffer_view(GL::ELEMENT_ARRAY_BUFFER, 0, &view);

        self.ibo_len = idx_data.len() as i32;
    }

    pub fn draw_elements(&self, mode: u32) {
        self.gl.bind_vertex_array(Some(&self.vao));
        self.gl.draw_elements_with_i32(mode, self.ibo_len, GL::UNSIGNED_SHORT, 0);
    }

}
