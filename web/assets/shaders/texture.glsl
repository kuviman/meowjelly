varying vec2 v_uv;
varying vec3 v_camera_pos;
varying vec3 v_world_pos;
varying float v_edge;

#ifdef VERTEX_SHADER
attribute vec3 a_pos;
attribute vec2 a_uv;
attribute vec3 a_normal;

uniform mat3 u_uv_matrix;
uniform mat4 u_model_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_projection_matrix;
void main() {
  v_edge = 1.0 - abs(a_normal.z);
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
uniform vec3 u_player_pos;
uniform float u_player_radius;
void main() {
  float fog = clamp(-v_camera_pos.z / u_fog_distance, 0.0, 1.0);
  float light = 1.0;
  if (length(v_world_pos.xy) >= 9.9) {
    if (length(cross(normalize(v_world_pos - vec3(0.0, 0.0, u_player_pos.z)), vec3(u_player_pos.xy, 0.0))) < u_player_radius && dot(v_world_pos.xy, u_player_pos.xy) > 0.0) {
      light *= 1.0 - clamp((length(u_player_pos.xy) - 3.0) / 6.0 * 0.5, 0.0, 1.0);
    }
  } else if (u_player_pos.z > v_world_pos.z + 1.0) {
    float k = clamp((u_player_pos.z - v_world_pos.z) / 200.0, 0.0, 1.0);
    if (length(u_player_pos.xy - v_world_pos.xy) < u_player_radius * (1.0 + k * 4.0)) {
      light *= k + (1.0 - k) * 0.5;
    }
  }
  vec4 color = smoothTexture2D(v_uv, u_texture, u_texture_size) * u_color * vec4(light, light, light, 1.0);
  color.rgb *= (1.0 - v_edge * 0.25);
  gl_FragColor = color * (1.0 - fog) + vec4(u_fog_color.rgb, color.a) * fog;
  if (gl_FragColor.a == 0.0) {
    discard;
  }
}
#endif
