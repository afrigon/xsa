from dataclasses import dataclass
from enum import Enum

# Khronos Data Format Specification values written into each KTX2 file's data format descriptor.
MODEL_RGBSDA = 1
MODEL_BC4 = 131
MODEL_BC5 = 132
MODEL_BC7 = 134
TRANSFER_LINEAR = 1
TRANSFER_SRGB = 2
CHANNEL_ALPHA = 15
QUALIFIER_LINEAR = 0x10
QUALIFIER_SIGNED = 0x40
QUALIFIER_FLOAT = 0x80
UNSIGNED_BYTE_MAX = 255
UNSIGNED_SHORT_MAX = 65535
BLOCK_SAMPLE_MAX = 0xFFFFFFFF
# A float sample's bounds are the bit patterns of -1.0 and 1.0.
FLOAT_SAMPLE_LOWER = 0xBF800000
FLOAT_SAMPLE_UPPER = 0x3F800000
HALF_BITS = 16
FLOAT_BITS = 32


@dataclass(frozen=True)
class Sample:
    bit_offset: int
    bit_length: int
    channel: int
    upper: int
    lower: int = 0


@dataclass(frozen=True)
class FormatDescription:
    vk_format: int
    model: int
    transfer: int
    block_size: int
    block_bytes: int
    samples: tuple[Sample, ...]

    # KTX2's typeSize: the size of the type each channel is stored as, 1 for block-compressed formats.
    @property
    def type_size(self) -> int:
        return self.samples[0].bit_length // 8 if self.block_size == 1 else 1


def _rgba8_samples(transfer: int) -> tuple[Sample, ...]:
    alpha = CHANNEL_ALPHA | (QUALIFIER_LINEAR if transfer == TRANSFER_SRGB else 0)
    channels = (0, 1, 2, alpha)
    return tuple(Sample(index * 8, 8, channel, UNSIGNED_BYTE_MAX) for index, channel in enumerate(channels))


def _rgba16_float_samples() -> tuple[Sample, ...]:
    qualifiers = QUALIFIER_FLOAT | QUALIFIER_SIGNED
    channels = (0, 1, 2, CHANNEL_ALPHA)
    return tuple(
        Sample(index * HALF_BITS, HALF_BITS, channel | qualifiers, FLOAT_SAMPLE_UPPER, FLOAT_SAMPLE_LOWER)
        for index, channel in enumerate(channels)
    )


class TextureFormat(Enum):
    RGBA8_SRGB = FormatDescription(43, MODEL_RGBSDA, TRANSFER_SRGB, 1, 4, _rgba8_samples(TRANSFER_SRGB))
    RGBA16_SFLOAT = FormatDescription(97, MODEL_RGBSDA, TRANSFER_LINEAR, 1, 8, _rgba16_float_samples())
    RG8_UNORM = FormatDescription(
        16,
        MODEL_RGBSDA,
        TRANSFER_LINEAR,
        1,
        2,
        (Sample(0, 8, 0, UNSIGNED_BYTE_MAX), Sample(8, 8, 1, UNSIGNED_BYTE_MAX)),
    )
    R16_UNORM = FormatDescription(70, MODEL_RGBSDA, TRANSFER_LINEAR, 1, 2, (Sample(0, 16, 0, UNSIGNED_SHORT_MAX),))
    R32_SFLOAT = FormatDescription(
        100,
        MODEL_RGBSDA,
        TRANSFER_LINEAR,
        1,
        4,
        (Sample(0, FLOAT_BITS, QUALIFIER_FLOAT | QUALIFIER_SIGNED, FLOAT_SAMPLE_UPPER, FLOAT_SAMPLE_LOWER),),
    )
    BC4_UNORM = FormatDescription(139, MODEL_BC4, TRANSFER_LINEAR, 4, 8, (Sample(0, 64, 0, BLOCK_SAMPLE_MAX),))
    BC5_UNORM = FormatDescription(
        141, MODEL_BC5, TRANSFER_LINEAR, 4, 16, (Sample(0, 64, 0, BLOCK_SAMPLE_MAX), Sample(64, 64, 1, BLOCK_SAMPLE_MAX))
    )
    BC7_SRGB = FormatDescription(146, MODEL_BC7, TRANSFER_SRGB, 4, 16, (Sample(0, 128, 0, BLOCK_SAMPLE_MAX),))

    def level_size(self, width: int, height: int) -> int:
        size = self.value.block_size
        return -(-width // size) * -(-height // size) * self.value.block_bytes
