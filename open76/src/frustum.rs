
// Matrix will receive the calculated perspective matrix.
// You would have to upload to your shader
// or use glLoadMatrixf if you aren't using shaders.
pub fn glh_perspectivef2(
    matrix: &mut [f32; 16],
    fovy_in_degrees: f32,
    aspect_ratio: f32,
    znear: f32,
    zfar: f32,
) {
    let ymax = znear * (fovy_in_degrees * std::f32::consts::PI / 360.0).tan();
    let xmax = ymax * aspect_ratio;
    glh_frustumf2(matrix, -xmax, xmax, -ymax, ymax, znear, zfar);
}

fn glh_frustumf2(
    matrix: &mut [f32],
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    znear: f32,
    zfar: f32,
) {
    let temp = 2.0 * znear;
    let temp2 = right - left;
    let temp3 = top - bottom;
    let temp4 = zfar - znear;
    matrix[0] = temp / temp2;
    matrix[1] = 0.0;
    matrix[2] = 0.0;
    matrix[3] = 0.0;
    matrix[4] = 0.0;
    matrix[5] = temp / temp3;
    matrix[6] = 0.0;
    matrix[7] = 0.0;
    matrix[8] = (right + left) / temp2;
    matrix[9] = (top + bottom) / temp3;
    matrix[10] = (-zfar - znear) / temp4;
    matrix[11] = -1.0;
    matrix[12] = 0.0;
    matrix[13] = 0.0;
    matrix[14] = (-temp * zfar) / temp4;
    matrix[15] = 0.0;
}