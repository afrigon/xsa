use glam::{Mat3, Quat, Vec3, Vec4};

const SNORM16_MAXIMUM: f32 = i16::MAX as f32;
// The smallest |w| a quantized quaternion keeps, so its sign survives rounding.
const MINIMUM_W: f32 = 1.0 / SNORM16_MAXIMUM;

/// A vertex's tangent frame as a QTangent (Frey and Herzog, "Spherical Skinning with Dual-Quaternions and QTangents",
/// SIGGRAPH 2011): the rotation from tangent space to the mesh's frame, a unit quaternion in four snorm16 components,
/// whose w carries the sign of the bitangent.
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TangentFrame {
    pub quaternion: [i16; 4],
}

impl TangentFrame {
    /// The frame of a normal and a glTF tangent, whose w (±1) says whether the bitangent is cross(normal, tangent) or
    /// its opposite. The tangent is made orthogonal to the normal first.
    pub fn new(normal: Vec3, tangent: Vec4) -> anyhow::Result<TangentFrame> {
        let normal = normal
            .try_normalize()
            .ok_or_else(|| anyhow::anyhow!("a vertex has a zero normal"))?;
        let mirrored = tangent.w < 0.0;
        let tangent = tangent
            .truncate()
            .reject_from_normalized(normal)
            .try_normalize()
            .ok_or_else(|| anyhow::anyhow!("a vertex has a tangent parallel to its normal"))?;
        let bitangent = normal.cross(tangent);
        let mut rotation = Quat::from_mat3(&Mat3::from_cols(tangent, bitangent, normal)).normalize();

        if rotation.w < 0.0 {
            rotation = -rotation;
        }

        if rotation.w < MINIMUM_W {
            let scale = (1.0 - MINIMUM_W * MINIMUM_W).sqrt() / rotation.xyz().length();
            let axis = rotation.xyz() * scale;
            rotation = Quat::from_xyzw(axis.x, axis.y, axis.z, MINIMUM_W);
        }

        if mirrored {
            rotation = -rotation;
        }

        Ok(TangentFrame {
            quaternion: rotation
                .to_array()
                .map(|component| (component * SNORM16_MAXIMUM).round() as i16),
        })
    }

    pub fn normal(self) -> Vec3 {
        self.rotation() * Vec3::Z
    }

    pub fn tangent(self) -> Vec3 {
        self.rotation() * Vec3::X
    }

    pub fn bitangent(self) -> Vec3 {
        self.rotation() * Vec3::Y * self.bitangent_sign()
    }

    /// 1 when the bitangent is cross(normal, tangent), −1 when it is the opposite.
    pub fn bitangent_sign(self) -> f32 {
        if self.quaternion[3] < 0 { -1.0 } else { 1.0 }
    }

    fn rotation(self) -> Quat {
        Quat::from_array(self.quaternion.map(|component| f32::from(component) / SNORM16_MAXIMUM)).normalize()
    }
}
