#ifdef VERTEX_SHADER
attribute vec3 a_pos;
void main() {
  gl_Position = vec4(a_pos, 1.0);
}
#endif

#ifdef FRAGMENT_SHADER
uniform vec4 u_color;
void main() {
  gl_FragColor = u_color;
}
#endif
