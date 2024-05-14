varying float v_t;
varying vec2 v_uv;
varying vec4 v_color;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
attribute vec2 a_uv;

attribute vec3 i_pos;
attribute vec3 i_vel;
attribute float i_size;
attribute float i_start_time;
attribute float i_end_time;
attribute vec4 i_color;

uniform float u_time;
uniform mat4 u_view_matrix;
uniform mat4 u_projection_matrix;
void main() {
  v_uv = a_uv;
  v_t = (u_time - i_start_time) / (i_end_time - i_start_time);
  v_color = i_color;
  v_color.a *= (1.0 - v_t);
  vec3 world_pos = i_pos + i_vel * (u_time - i_start_time) + vec3(a_pos, 0.0) * i_size;
  gl_Position = u_projection_matrix * u_view_matrix * vec4(world_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
uniform ivec2 u_texture_size;
void main() {
  gl_FragColor = smoothTexture2D(v_uv, u_texture, u_texture_size) * v_color;
}
#endif
