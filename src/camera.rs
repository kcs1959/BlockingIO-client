use nalgebra as na;
use reverie_engine::interpolation::{
    types::{Time, TimeSpan},
    Interpolation,
};

pub struct Camera {
    pos: na::Point3<f32>,
    interpolation_x: Interpolation<f32>,
    interpolation_y: Interpolation<f32>,
    interpolation_z: Interpolation<f32>,
}

impl Camera {
    pub fn new(player_pos: na::Point3<f32>) -> Self {
        Self {
            pos: player_pos,
            interpolation_x: Interpolation::<f32>::new_lerp(0.0, 0.0, 0, 1),
            interpolation_y: Interpolation::<f32>::new_lerp(0.0, 0.0, 0, 1),
            interpolation_z: Interpolation::<f32>::new_lerp(0.0, 0.0, 0, 1),
        }
    }

    /// 現在の場所から指定した場所へ、フレーム数`shade_frames`で滑らかに移動するように設定する
    ///  - `current_frames` - ゲーム開始からの総フレーム数
    pub fn shade_to_new_position(
        &mut self,
        pos: na::Point3<f32>,
        current_frames: u64,
        shade_frames: u64,
    ) {
        self.interpolation_x = Interpolation::new_cubic_ease_in_out(
            self.pos.x,
            pos.x,
            current_frames as Time,
            shade_frames as TimeSpan,
        );
        self.interpolation_y = Interpolation::new_cubic_ease_in_out(
            self.pos.y,
            pos.y,
            current_frames as Time,
            shade_frames as TimeSpan,
        );
        self.interpolation_z = Interpolation::new_cubic_ease_in_out(
            self.pos.z,
            pos.z,
            current_frames as Time,
            shade_frames as TimeSpan,
        );
    }

    pub fn update_position(&mut self, current_frames: u64) {
        self.pos.x = self.interpolation_x.value(current_frames as Time);
        self.pos.y = self.interpolation_y.value(current_frames as Time);
        self.pos.z = self.interpolation_z.value(current_frames as Time);
    }

    pub fn view_matrix(&self, scale: f32) -> na::Matrix4<f32> {
        const CAM_HEIGHT: na::Vector3<f32> = na::Vector3::new(0.0, 5.0, 0.0);
        // カメラが見下ろす方向は-y方向
        const DOWN: na::Vector3<f32> = na::Vector3::new(0.0, -1.0, 0.0);
        // 画面の上方向は+x方向
        const X_POSITIVE: na::Vector3<f32> = na::Vector3::new(1.0, 0.0, 0.0);

        na::Matrix4::look_at_rh(
            &(self.pos * scale + CAM_HEIGHT),
            &((self.pos + DOWN) * scale),
            &X_POSITIVE,
        )
    }
}