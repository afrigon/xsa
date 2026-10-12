# Blurs applied through the Fourier transform, channel by channel.
import numpy as np

def disk_blur(image, radius):
    """Convolves each channel with a uniform disk of `radius` pixels."""
    if radius < 0.5:
        return image
    h, w = image.shape[:2]
    y = np.fft.fftfreq(h)[:, None] * h
    x = np.fft.fftfreq(w)[None, :] * w
    kernel = (x ** 2 + y ** 2 <= radius ** 2).astype(float)
    kernel /= kernel.sum()
    spectrum = np.fft.rfft2(kernel)
    return np.stack([np.fft.irfft2(np.fft.rfft2(image[..., c]) * spectrum, s=(h, w)) for c in range(image.shape[2])], -1)

def gaussian_blur(image, sigma):
    """Convolves each channel with a Gaussian of standard deviation `sigma` pixels."""
    fy = np.fft.fftfreq(image.shape[0])[:, None]
    fx = np.fft.fftfreq(image.shape[1])[None, :]
    kernel = np.exp(-2 * (np.pi * sigma) ** 2 * (fx ** 2 + fy ** 2))
    return np.stack([np.real(np.fft.ifft2(np.fft.fft2(image[..., c]) * kernel)) for c in range(image.shape[2])], -1)
