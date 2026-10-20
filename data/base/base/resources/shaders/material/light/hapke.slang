// Hapke reflectance for particulate surfaces (regolith, frost), equation numbers from Hapke, "Theory of Reflectance
// and Emittance Spectroscopy" (2nd edition, 2012). The helpers are ported from the Custom/HapkeScaled shader of
// CharonSSS/AdvancedPQSTools (MIT, Copyright (c) 2022 Niako; Hapke shaders by ballisticfox), see data/base/LICENSE.

static const float hapkePi = 3.14159265358979;

struct HapkeGeometry {
    float incidenceCosine;
    float emissionCosine;
    float phaseAngle;
};

// Double-lobed Henyey-Greenstein phase function (6.7a).
float phaseFunction(float phaseAngle, float lobeWidth, float lobeBalance) {
    float b2 = lobeWidth * lobeWidth;
    float cosine = cos(phaseAngle);
    float backward = (1.0 + lobeBalance) * 0.5 * (1.0 - b2) / pow(1.0 - 2.0 * lobeWidth * cosine + b2, 1.5);
    float forward = (1.0 - lobeBalance) * 0.5 * (1.0 - b2) / pow(1.0 + 2.0 * lobeWidth * cosine + b2, 1.5);
    return backward + forward;
}

// Shadow-hiding opposition effect (9.22).
float shadowHiding(float phaseAngle, float width) {
    return 1.0 / (1.0 + tan(phaseAngle * 0.5) / width);
}

// Ambartsumian-Chandrasekhar H function, Hapke's approximation (8.56).
float3 chandrasekharH(float x, float3 albedo, float3 diffusiveReflectance) {
    return 1.0 / (1.0 - albedo * x *
                            (diffusiveReflectance + (1.0 - 2.0 * diffusiveReflectance * x) * 0.5 * log((1.0 + x) / x)));
}

float roughnessE1(float angle, float roughness) {
    return exp(-2.0 / hapkePi / (tan(roughness) * tan(angle)));
}

float roughnessE2(float angle, float roughness) {
    float cotangents = 1.0 / (tan(roughness) * tan(angle));
    return exp(-cotangents * cotangents / hapkePi);
}

float roughnessChi(float roughness) {
    float tangent = tan(roughness);
    return 1.0 / sqrt(1.0 + hapkePi * tangent * tangent);
}

// Effective cosine of a smooth-surface angle (12.48).
float roughnessEta(float angle, float roughness) {
    return roughnessChi(roughness) * (cos(angle) + sin(angle) * tan(roughness) * roughnessE2(angle, roughness) /
                                                       (2.0 - roughnessE1(angle, roughness)));
}

struct RoughnessCorrection {
    float incidenceCosine;
    float emissionCosine;
    float shadowing;
};

// Macroscopic roughness (12.46-12.54): effective cosines of incidence and emission and the shadowing function S, for
// mean slope angle `roughness` and azimuth `azimuth` between the planes of incidence and emission.
RoughnessCorrection roughnessCorrection(float incidenceCosine, float emissionCosine, float azimuth, float roughness) {
    float incidence = acos(incidenceCosine);
    float emission = acos(emissionCosine);
    float chi = roughnessChi(roughness);
    float tangent = tan(roughness);
    float f = exp(-2.0 * tan(azimuth * 0.5));
    float sinHalfAzimuth2 = sin(azimuth * 0.5) * sin(azimuth * 0.5);
    float etaIncidence = roughnessEta(incidence, roughness);
    float etaEmission = roughnessEta(emission, roughness);
    float e1i = roughnessE1(incidence, roughness);
    float e2i = roughnessE2(incidence, roughness);
    float e1e = roughnessE1(emission, roughness);
    float e2e = roughnessE2(emission, roughness);

    RoughnessCorrection correction;
    if (incidence <= emission) {
        float denominator = 2.0 - e1e - azimuth / hapkePi * e1i;
        correction.incidenceCosine =
            chi *
            (incidenceCosine + sin(incidence) * tangent * (cos(azimuth) * e2e + sinHalfAzimuth2 * e2i) / denominator);
        correction.emissionCosine =
            chi * (emissionCosine + sin(emission) * tangent * (e2e - sinHalfAzimuth2 * e2i) / denominator);
        correction.shadowing = correction.emissionCosine / etaEmission * incidenceCosine / etaIncidence * chi /
                               (1.0 - f + f * chi * incidenceCosine / etaIncidence);
    } else {
        float denominator = 2.0 - e1i - azimuth / hapkePi * e1e;
        correction.incidenceCosine =
            chi * (incidenceCosine + sin(incidence) * tangent * (e2i - sinHalfAzimuth2 * e2e) / denominator);
        correction.emissionCosine =
            chi *
            (emissionCosine + sin(emission) * tangent * (cos(azimuth) * e2i + sinHalfAzimuth2 * e2e) / denominator);
        correction.shadowing = correction.emissionCosine / etaEmission * incidenceCosine / etaIncidence * chi /
                               (1.0 - f + f * chi * emissionCosine / etaEmission);
    }
    return correction;
}

// Angle between the planes of incidence and emission, from the phase angle and both cosines.
float azimuthAngle(HapkeGeometry geometry) {
    float sines = sqrt(max((1.0 - geometry.incidenceCosine * geometry.incidenceCosine) *
                               (1.0 - geometry.emissionCosine * geometry.emissionCosine),
                           0.0));
    if (sines < 1e-6) {
        return 0.0;
    }
    float cosine = (cos(geometry.phaseAngle) - geometry.incidenceCosine * geometry.emissionCosine) / sines;
    return acos(clamp(cosine, -1.0, 1.0));
}

// Hapke 2012's inputs for a particulate medium, everything but the viewing geometry.
struct HapkeParameters {
    // w, per color channel.
    float3 singleScatteringAlbedo;
    // b and c of the double-lobed phase function; c > 0 scatters back toward the light.
    float lobeWidth;
    float lobeBalance;
    // θ̄, in radians.
    float roughness;
    // K.
    float porosity;
    float shadowHidingAmplitude;
    float shadowHidingWidth;
    float backscatterAmplitude;
    float backscatterWidth;
};

// w from the diffusive reflectance r0, inverting r0 = (1 - γ) / (1 + γ), γ = √(1 - w) (8.25).
float3 singleScatteringAlbedo(float3 diffusiveReflectance) {
    float3 gamma = (1.0 - diffusiveReflectance) / (1.0 + diffusiveReflectance);
    return 1.0 - gamma * gamma;
}

// Coherent-backscatter opposition effect (9.43); it tends to 1 at opposition.
float coherentBackscatterOpposition(float phaseAngle, float width) {
    float zeta = tan(phaseAngle * 0.5) / width;
    if (zeta < 1e-6) {
        return 1.0;
    }
    return (1.0 + (1.0 - exp(-zeta)) / zeta) / (2.0 * (1.0 + zeta) * (1.0 + zeta));
}

// Hapke's bidirectional reflectance with porosity, both opposition effects and macroscopic roughness (12.55), per
// color channel. Returns cd/m² per lux.
float3 hapkeReflectance(HapkeParameters parameters, HapkeGeometry geometry) {
    float3 w = clamp(parameters.singleScatteringAlbedo, 0.0, 0.999);
    float3 gamma = sqrt(1.0 - w);
    float3 diffusiveReflectance = (1.0 - gamma) / (1.0 + gamma);
    float porosity = parameters.porosity;
    float g = geometry.phaseAngle;

    RoughnessCorrection rough = roughnessCorrection(
        geometry.incidenceCosine, geometry.emissionCosine, azimuthAngle(geometry), parameters.roughness);
    float mu0 = max(rough.incidenceCosine, 1e-4);
    float mu = max(rough.emissionCosine, 1e-4);

    float shadowHidingOpposition =
        1.0 + parameters.shadowHidingAmplitude * shadowHiding(g, parameters.shadowHidingWidth);
    float singleScattering = phaseFunction(g, parameters.lobeWidth, parameters.lobeBalance) * shadowHidingOpposition;
    float3 multipleScattering = chandrasekharH(mu0 / porosity, w, diffusiveReflectance) *
                                    chandrasekharH(mu / porosity, w, diffusiveReflectance) -
                                1.0;
    float backscatterOpposition =
        1.0 + parameters.backscatterAmplitude * coherentBackscatterOpposition(g, parameters.backscatterWidth);

    return porosity * w / (4.0 * hapkePi) * mu0 / (mu0 + mu) * (singleScattering + multipleScattering) *
           backscatterOpposition * rough.shadowing;
}
