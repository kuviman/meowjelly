varying vec2 v_uv;
varying vec3 v_camera_pos;
varying vec3 v_world_pos;

#ifdef VERTEX_SHADER
attribute vec3 a_pos;
attribute vec2 a_uv;

uniform mat3 u_uv_matrix;
uniform mat4 u_model_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_projection_matrix;
void main() {
  v_uv = (u_uv_matrix * vec3(a_uv, 1.0)).xy;
  v_world_pos = (u_model_matrix * vec4(a_pos, 1.0)).xyz;
  v_camera_pos = (u_view_matrix * vec4(v_world_pos, 1.0)).xyz;
  gl_Position = u_projection_matrix * vec4(v_camera_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform vec4 u_color;
uniform sampler2D u_texture;
uniform ivec2 u_texture_size;
uniform float u_fog_distance;
uniform vec4 u_fog_color;
void main() {
  float fog = clamp(-v_camera_pos.z / u_fog_distance, 0.0, 1.0);
  vec4 color = smoothTexture2D(v_uv, u_texture, u_texture_size) * u_color;
  gl_FragColor = color * (1.0 - fog) + vec4(u_fog_color.rgb, color.a) * fog;
  if (gl_FragColor.a == 0.0) {
    discard;
  }
}
#endif
