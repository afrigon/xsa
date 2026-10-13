import numpy as np

from operations.hapke_model import Geometry, HapkeModel

STANDARD = Geometry(incidence=np.radians(60), emission=0.0, phase=np.radians(60))


def model(size: int) -> HapkeModel:
    return HapkeModel(
        lobe_width=np.full(size, 0.25),
        lobe_balance=np.full(size, 0.4),
        shadow_hiding_amplitude=np.full(size, 1.7),
        shadow_hiding_width=np.full(size, 0.06),
        backscatter_amplitude=0.0,
        backscatter_width=1.0,
        porosity=1.0,
        roughness=np.radians(23.657),
    )


def test_solving_the_albedo_reproduces_the_measurement():
    albedo = np.array([0.05, 0.2, 0.45, 0.8])
    hapke = model(albedo.size)
    radiance_factor = np.pi * hapke.reflectance(albedo, STANDARD)

    assert np.allclose(hapke.single_scattering_albedo(radiance_factor, STANDARD), albedo, atol=1e-6)


def test_missing_measurements_stay_missing():
    assert np.isnan(model(1).single_scattering_albedo(np.array([np.nan]), STANDARD)[0])


def test_diffusive_reflectance_grows_with_the_albedo_up_to_one():
    reflectance = HapkeModel.diffusive_reflectance(np.array([0.0, 0.5, 1.0]))

    assert reflectance[0] == 0 and 0 < reflectance[1] < 1 and reflectance[2] == 1
