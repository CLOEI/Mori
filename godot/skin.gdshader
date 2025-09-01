shader_type canvas_item;

uniform vec4 skin_color : source_color = vec4(1.0, 1.0, 1.0, 1.0);

void fragment() {
    vec4 original_color = texture(TEXTURE, UV);
    COLOR = original_color * skin_color;
}