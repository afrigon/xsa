from dataclasses import dataclass

import numpy as np

MAXIMUM_ALBEDO = 0.999
MINIMUM_COSINE = 1e-4
# The inversion table's resolution along each of its axes, and the albedo samples its curves are interpolated from.
ALBEDO_SAMPLES = 8192
SINGLE_SCATTERING_SAMPLES = 64
RADIANCE_FACTOR_SAMPLES = 4096


@dataclass(frozen=True)
class Geometry:
    incidence: float
    emission: float
    phase: float


# Hapke's bidirectional reflectance (Hapke 2012, eq. 12.55), the same model as hapkeReflectance in the base pack's
# material/light/hapke.slang, so a value solved here renders back to the measurement it came from. Angles in radians.
@dataclass(frozen=True)
class HapkeModel:
    lobe_width: np.ndarray
    lobe_balance: np.ndarray
    shadow_hiding_amplitude: np.ndarray
    shadow_hiding_width: np.ndarray
    backscatter_amplitude: float
    backscatter_width: float
    porosity: float
    roughness: float

    def reflectance(self, albedo: np.ndarray, geometry: Geometry) -> np.ndarray:
        factor, multiple = self._albedo_terms(albedo, geometry)
        return factor * (self._single_scattering(geometry.phase) + multiple)

    # The single-scattering albedo whose radiance factor (I/F, π times the reflectance) matches the measurement, read
    # from a table over the per-pixel single-scattering term and the radiance factor. Reflectance grows with the
    # albedo, so each of the table's rows inverts one increasing curve.
    def single_scattering_albedo(self, radiance_factor: np.ndarray, geometry: Geometry) -> np.ndarray:
        single = np.broadcast_to(self._single_scattering(geometry.phase), radiance_factor.shape)
        albedos = np.linspace(0, MAXIMUM_ALBEDO, ALBEDO_SAMPLES)
        factor, multiple = self._albedo_terms(albedos, geometry)
        singles = np.linspace(np.min(single), np.max(single), SINGLE_SCATTERING_SAMPLES)
        curves = np.pi * factor * (singles[:, None] + multiple)
        radiance_factors = np.linspace(0, curves[:, -1].max(), RADIANCE_FACTOR_SAMPLES)
        table = np.stack([np.interp(radiance_factors, curve, albedos) for curve in curves])
        measured = np.isfinite(radiance_factor)
        rows = HapkeModel._fractional_index(single, singles)
        columns = HapkeModel._fractional_index(np.where(measured, radiance_factor, 0), radiance_factors)
        return np.where(measured, HapkeModel._bilinear(table, rows, columns), np.nan)

    # Where each value falls along evenly spaced samples, in samples, held to the samples' span.
    @staticmethod
    def _fractional_index(values: np.ndarray, samples: np.ndarray) -> np.ndarray:
        span = samples[-1] - samples[0]

        if span == 0:
            return np.zeros_like(values)

        return np.clip((values - samples[0]) / span * (len(samples) - 1), 0, len(samples) - 1)

    @staticmethod
    def _bilinear(table: np.ndarray, rows: np.ndarray, columns: np.ndarray) -> np.ndarray:
        row = np.minimum(rows.astype(int), table.shape[0] - 2)
        column = np.minimum(columns.astype(int), table.shape[1] - 2)
        down, right = rows - row, columns - column
        top = table[row, column] * (1 - right) + table[row, column + 1] * right
        bottom = table[row + 1, column] * (1 - right) + table[row + 1, column + 1] * right
        return top * (1 - down) + bottom * down

    # The single-scattering term, phase function with the shadow-hiding surge: the only one holding the parameters
    # that vary per pixel.
    def _single_scattering(self, phase: float) -> np.ndarray:
        # A zero surge width divides by zero; the limit, no surge away from opposition, is exact.
        with np.errstate(divide="ignore"):
            shadow_hiding = 1 / (1 + np.tan(phase / 2) / self.shadow_hiding_width)

        return self._phase_function(phase) * (1 + self.shadow_hiding_amplitude * shadow_hiding)

    # The reflectance is factor × (single scattering + multiple), both depending only on the albedo and the
    # parameters held constant over the body.
    def _albedo_terms(self, albedo: np.ndarray, geometry: Geometry) -> tuple[np.ndarray, np.ndarray]:
        w = np.clip(albedo, 0.0, MAXIMUM_ALBEDO)
        r0 = HapkeModel.diffusive_reflectance(w)
        incidence_cosine, emission_cosine, shadowing = self._roughness_correction(geometry)
        mu0 = max(incidence_cosine, MINIMUM_COSINE)
        mu = max(emission_cosine, MINIMUM_COSINE)
        multiple = self._chandrasekhar(mu0 / self.porosity, w, r0) * self._chandrasekhar(mu / self.porosity, w, r0) - 1
        backscatter = 1 + self.backscatter_amplitude * self._coherent_backscatter(geometry.phase)
        factor = self.porosity * w / (4 * np.pi) * mu0 / (mu0 + mu) * backscatter * shadowing
        return factor, multiple

    # Hapke's diffusive reflectance r₀ (eq. 8.25), the albedo a body shows under light from every direction.
    @staticmethod
    def diffusive_reflectance(albedo: np.ndarray) -> np.ndarray:
        gamma = np.sqrt(1 - albedo)
        return (1 - gamma) / (1 + gamma)

    # Coherent-backscatter opposition effect (eq. 9.43); its limit at opposition is 1.
    def _coherent_backscatter(self, phase: float) -> float:
        zeta = np.tan(phase / 2) / self.backscatter_width

        if zeta == 0:
            return 1.0

        return (1 + (1 - np.exp(-zeta)) / zeta) / (2 * (1 + zeta) ** 2)

    # Double-lobed Henyey-Greenstein phase function (eq. 6.7a).
    def _phase_function(self, phase: float) -> np.ndarray:
        b2 = self.lobe_width**2
        cosine = np.cos(phase)
        backward = (1 + self.lobe_balance) / 2 * (1 - b2) / (1 - 2 * self.lobe_width * cosine + b2) ** 1.5
        forward = (1 - self.lobe_balance) / 2 * (1 - b2) / (1 + 2 * self.lobe_width * cosine + b2) ** 1.5
        return backward + forward

    # Ambartsumian-Chandrasekhar H function, Hapke's approximation (eq. 8.56).
    @staticmethod
    def _chandrasekhar(x: float, albedo: np.ndarray, r0: np.ndarray) -> np.ndarray:
        return 1 / (1 - albedo * x * (r0 + (1 - 2 * r0 * x) / 2 * np.log((1 + x) / x)))

    # Macroscopic roughness (eq. 12.46-12.54): effective cosines and the shadowing function.
    def _roughness_correction(self, geometry: Geometry) -> tuple[float, float, float]:
        i, e = geometry.incidence, geometry.emission
        tangent = np.tan(self.roughness)
        azimuth = self._azimuth(geometry)
        chi = self._chi()
        f = np.exp(-2 * np.tan(azimuth / 2))
        sin_half_azimuth2 = np.sin(azimuth / 2) ** 2
        e1i, e2i, e1e, e2e = self._e1(i), self._e2(i), self._e1(e), self._e2(e)
        eta_i, eta_e = self._eta(i), self._eta(e)

        if i <= e:
            denominator = 2 - e1e - azimuth / np.pi * e1i
            incidence_cosine = chi * (
                np.cos(i) + np.sin(i) * tangent * (np.cos(azimuth) * e2e + sin_half_azimuth2 * e2i) / denominator
            )
            emission_cosine = chi * (np.cos(e) + np.sin(e) * tangent * (e2e - sin_half_azimuth2 * e2i) / denominator)
            shadowing = emission_cosine / eta_e * np.cos(i) / eta_i * chi / (1 - f + f * chi * np.cos(i) / eta_i)
        else:
            denominator = 2 - e1i - azimuth / np.pi * e1e
            incidence_cosine = chi * (np.cos(i) + np.sin(i) * tangent * (e2i - sin_half_azimuth2 * e2e) / denominator)
            emission_cosine = chi * (
                np.cos(e) + np.sin(e) * tangent * (np.cos(azimuth) * e2i + sin_half_azimuth2 * e2e) / denominator
            )
            shadowing = emission_cosine / eta_e * np.cos(i) / eta_i * chi / (1 - f + f * chi * np.cos(e) / eta_e)

        return float(incidence_cosine), float(emission_cosine), float(shadowing)

    def _chi(self) -> float:
        return 1 / np.sqrt(1 + np.pi * np.tan(self.roughness) ** 2)

    # At a zero angle the cotangent is infinite and both exponentials are exactly 0.
    def _cotangents(self, angle: float) -> float:
        with np.errstate(divide="ignore"):
            return 1 / (np.tan(self.roughness) * np.tan(angle))

    def _e1(self, angle: float) -> float:
        return np.exp(-2 / np.pi * self._cotangents(angle))

    def _e2(self, angle: float) -> float:
        return np.exp(-(self._cotangents(angle) ** 2) / np.pi)

    # Effective cosine of a smooth-surface angle (eq. 12.48).
    def _eta(self, angle: float) -> float:
        return self._chi() * (
            np.cos(angle) + np.sin(angle) * np.tan(self.roughness) * self._e2(angle) / (2 - self._e1(angle))
        )

    @staticmethod
    def _azimuth(geometry: Geometry) -> float:
        sines = np.sin(geometry.incidence) * np.sin(geometry.emission)

        if sines < 1e-6:
            return 0.0

        cosine = (np.cos(geometry.phase) - np.cos(geometry.incidence) * np.cos(geometry.emission)) / sines
        return float(np.arccos(np.clip(cosine, -1, 1)))
