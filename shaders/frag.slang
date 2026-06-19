struct VSOutput {
    float4 position : SV_Position;
    float4 color : COLOR0;
    float3 normal : NORMAL0;
};

static const float3 light_dir = normalize(float3(0.5, 1.0, 0.7));

float4 main(VSOutput input) : SV_Target0 {
    float ndotl = max(dot(normalize(input.normal), light_dir), 0.0);
    return float4(input.color.rgb * ndotl, input.color.a);
}
