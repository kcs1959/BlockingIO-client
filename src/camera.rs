use crate::types::*;
use re::interpolation::{Time, TimeSpan};
use re::Interpolation;

pub struct Camera {
    pos: Point3,
    interpolation_x: Interpolation<f32>,
    interpolation_y: Interpolation<f32>,
    interpolation_z: Interpolation<f32>,
}

impl Camera {
    pub fn new(player_pos: Point3) -> Self {
        Self {
            pos: player_pos,
            interpolation_x: Interpolation::<f32>::new_constant(player_pos.x),
            interpolation_y: Interpolation::<f32>::new_constant(player_pos.y),
            interpolation_z: Interpolation::<f32>::new_constant(player_pos.z),
        }
    }

    pub fn pos(&self) -> &Point3 {
        &self.pos
    }

    /// 現在の場所から指定した場所へ、フレーム数`shade_frames`で滑らかに移動するように設定する
    ///  - `current_frames` - ゲーム開始からの総フレーム数
    pub fn shade_to_new_position(&mut self, pos: Point3, current_frames: u64, shade_frames: u64) {
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

    pub fn view_matrix(&self, scale: f32) -> Matrix4 {
        const CAM_HEIGHT: Vector3 = Vector3::new(0.0, 5.0, 0.0);
        // カメラが見下ろす方向は-y方向
        const DOWN: Vector3 = Vector3::new(0.0, -1.0, 0.0);
        // 画面の上方向は+x方向
        const X_POSITIVE: Vector3 = Vector3::new(1.0, 0.0, 0.0);

        Matrix4::look_at_rh(
            &(self.pos * scale + CAM_HEIGHT),
            &((self.pos + DOWN) * scale),
            &X_POSITIVE,
        )
    }
}
