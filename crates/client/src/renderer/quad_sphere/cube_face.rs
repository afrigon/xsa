use glam::DVec3;

// A face of the cube spanning -1 to 1 on each axis; a point on it is normal + u * horizontal + v * vertical.
pub(in crate::renderer) struct CubeFace {
    pub normal: DVec3,
    pub horizontal: DVec3,
    pub vertical: DVec3,
}

impl CubeFace {
    pub const ALL: [CubeFace; 6] = [
        CubeFace {
            normal: DVec3::X,
            horizontal: DVec3::Y,
            vertical: DVec3::Z,
        },
        CubeFace {
            normal: DVec3::NEG_X,
            horizontal: DVec3::NEG_Y,
            vertical: DVec3::Z,
        },
        CubeFace {
            normal: DVec3::Y,
            horizontal: DVec3::NEG_X,
            vertical: DVec3::Z,
        },
        CubeFace {
            normal: DVec3::NEG_Y,
            horizontal: DVec3::X,
            vertical: DVec3::Z,
        },
        CubeFace {
            normal: DVec3::Z,
            horizontal: DVec3::X,
            vertical: DVec3::Y,
        },
        CubeFace {
            normal: DVec3::NEG_Z,
            horizontal: DVec3::X,
            vertical: DVec3::NEG_Y,
        },
    ];

    pub fn point(&self, u: f64, v: f64) -> DVec3 {
        self.normal + u * self.horizontal + v * self.vertical
    }
}
