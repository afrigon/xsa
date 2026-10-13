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
UNSIGNED_BYTE_MAX = 255
BLOCK_SAMPLE_MAX = 0xFFFFFFFF


@dataclass(frozen=True)
class Sample:
    bit_offset: int
    bit_length: int
    channel: int
    upper: int


@dataclass(frozen=True)
class FormatDescription:
    vk_format: int
    model: int
    transfer: int
    block_size: int
    block_bytes: int
    samples: tuple[Sample, ...]


def _rgba8_samples(transfer: int) -> tuple[Sample, ...]:
    alpha = CHANNEL_ALPHA | (QUALIFIER_LINEAR if transfer == TRANSFER_SRGB else 0)
    channels = (0, 1, 2, alpha)
    return tuple(Sample(index * 8, 8, channel, UNSIGNED_BYTE_MAX) for index, channel in enumerate(channels))


class TextureFormat(Enum):
    RGBA8_SRGB = FormatDescription(43, MODEL_RGBSDA, TRANSFER_SRGB, 1, 4, _rgba8_samples(TRANSFER_SRGB))
    BC4_UNORM = FormatDescription(139, MODEL_BC4, TRANSFER_LINEAR, 4, 8, (Sample(0, 64, 0, BLOCK_SAMPLE_MAX),))
    BC5_UNORM = FormatDescription(
        141, MODEL_BC5, TRANSFER_LINEAR, 4, 16, (Sample(0, 64, 0, BLOCK_SAMPLE_MAX), Sample(64, 64, 1, BLOCK_SAMPLE_MAX))
    )
    BC7_SRGB = FormatDescription(146, MODEL_BC7, TRANSFER_SRGB, 4, 16, (Sample(0, 128, 0, BLOCK_SAMPLE_MAX),))

    def level_size(self, width: int, height: int) -> int:
        size = self.value.block_size
        return -(-width // size) * -(-height // size) * self.value.block_bytes
