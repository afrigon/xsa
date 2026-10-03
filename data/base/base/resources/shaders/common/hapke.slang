// Hapke reflectance for particulate surfaces (regolith, frost), equation numbers from Hapke, "Theory of Reflectance
// and Emittance Spectroscopy" (2nd edition, 2012). hapkeSol and the helpers it shares are ported from the
// Custom/HapkeScaled shader of CharonSSS/AdvancedPQSTools (MIT, Copyright (c) 2022 Niako; Hapke shaders by
// ballisticfox), see data/base/LICENSE.

static const float hapkePi = 3.14159265358979;

struct HapkeSurface
{
    float3 albedo;
    float singleScatteringAlbedo;
    float lobeWidth;
    float lobeBalance;
    float shadowHidingAmplitude;
    float shadowHidingWidth;
    float backscatterAmplitude;
    float backscatterWidth;
    float porosity;
    float roughness;
};

struct HapkeGeometry
{
    float incidenceCosine;
    float emissionCosine;
    float phaseAngle;
};

// Double-lobed Henyey-Greenstein phase function (6.7a).
float phaseFunction(float phaseAngle, float lobeWidth, float lobeBalance)
{
    float b2 = lobeWidth * lobeWidth;
    float cosine = cos(phaseAngle);
    float backward = (1.0 + lobeBalance) * 0.5 * (1.0 - b2) / pow(1.0 - 2.0 * lobeWidth * cosine + b2, 1.5);
    float forward = (1.0 - lobeBalance) * 0.5 * (1.0 - b2) / pow(1.0 + 2.0 * lobeWidth * cosine + b2, 1.5);
    return backward + forward;
}

// Shadow-hiding opposition effect (9.22).
float shadowHiding(float phaseAngle, float width)
{
    return 1.0 / (1.0 + tan(phaseAngle * 0.5) / width);
}

// Coherent-backscatter opposition effect (9.43).
float coherentBackscatter(float phaseAngle, float width, float porosity)
{
    float zeta = tan(phaseAngle * 0.5) / width;
    return 1.0 / (1.0 + (1.3 + porosity) * (zeta + zeta * zeta));
}

// Ambartsumian-Chandrasekhar H function, Hapke's approximation (8.56).
float3 chandrasekharH(float x, float3 albedo, float3 diffusiveReflectance)
{
    return 1.0 / (1.0 - albedo * x * (diffusiveReflectance + (1.0 - 2.0 * diffusiveReflectance * x) * 0.5 * log((1.0 + x) / x)));
}

float roughnessE1(float angle, float roughness)
{
    return exp(-2.0 / hapkePi / (tan(roughness) * tan(angle)));
}

float roughnessE2(float angle, float roughness)
{
    float cotangents = 1.0 / (tan(roughness) * tan(angle));
    return exp(-cotangents * cotangents / hapkePi);
}

float roughnessChi(float roughness)
{
    float tangent = tan(roughness);
    return 1.0 / sqrt(1.0 + hapkePi * tangent * tangent);
}

// Effective cosine of a smooth-surface angle (12.48).
float roughnessEta(float angle, float roughness)
{
    return roughnessChi(roughness)
        * (cos(angle) + sin(angle) * tan(roughness) * roughnessE2(angle, roughness) / (2.0 - roughnessE1(angle, roughness)));
}

struct RoughSurface
{
    float incidenceCosine;
    float emissionCosine;
    float shadowing;
};

// Macroscopic roughness (12.46-12.54): effective cosines of incidence and emission and the shadowing function S, for
// mean slope angle `roughness` and azimuth `azimuth` between the planes of incidence and emission.
RoughSurface roughSurface(float incidenceCosine, float emissionCosine, float azimuth, float roughness)
{
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

    RoughSurface surface;
    if (incidence <= emission)
    {
        float denominator = 2.0 - e1e - azimuth / hapkePi * e1i;
        surface.incidenceCosine = chi * (incidenceCosine + sin(incidence) * tangent * (cos(azimuth) * e2e + sinHalfAzimuth2 * e2i) / denominator);
        surface.emissionCosine = chi * (emissionCosine + sin(emission) * tangent * (e2e - sinHalfAzimuth2 * e2i) / denominator);
        surface.shadowing = surface.emissionCosine / etaEmission * incidenceCosine / etaIncidence
            * chi / (1.0 - f + f * chi * incidenceCosine / etaIncidence);
    }
    else
    {
        float denominator = 2.0 - e1i - azimuth / hapkePi * e1e;
        surface.incidenceCosine = chi * (incidenceCosine + sin(incidence) * tangent * (e2i - sinHalfAzimuth2 * e2e) / denominator);
        surface.emissionCosine = chi * (emissionCosine + sin(emission) * tangent * (cos(azimuth) * e2i + sinHalfAzimuth2 * e2e) / denominator);
        surface.shadowing = surface.emissionCosine / etaEmission * incidenceCosine / etaIncidence
            * chi / (1.0 - f + f * chi * emissionCosine / etaEmission);
    }
    return surface;
}

// Angle between the planes of incidence and emission, from the phase angle and both cosines.
float azimuthAngle(HapkeGeometry geometry)
{
    float sines = sqrt(max((1.0 - geometry.incidenceCosine * geometry.incidenceCosine)
        * (1.0 - geometry.emissionCosine * geometry.emissionCosine), 0.0));
    if (sines < 1e-6)
    {
        return 0.0;
    }
    float cosine = (cos(geometry.phaseAngle) - geometry.incidenceCosine * geometry.emissionCosine) / sines;
    return acos(clamp(cosine, -1.0, 1.0));
}

// Reflectance factor per unit illuminance, as Sol renders it: the colour map multiplies the result, w only enters
// the multiple scattering, r0 is written 2 / (1 + γ), the roughness term receives the phase angle as its azimuth,
// and the result passes Sol's look trims. Returns cd/m² per lux.
float3 hapkeSol(HapkeSurface surface, HapkeGeometry geometry, float blend, float lightBoost, float gammaBoost)
{
    float w = surface.singleScatteringAlbedo;
    float diffusiveReflectance = 2.0 / (1.0 + sqrt(1.0 - w));
    float porosity = surface.porosity;
    float mu0 = geometry.incidenceCosine;
    float mu = geometry.emissionCosine;
    float g = geometry.phaseAngle;

    float lighting = blend * mu0 / (mu0 + mu) + (1.0 - blend) * mu0;
    float phaseAndShadowHiding = phaseFunction(g, surface.lobeWidth, surface.lobeBalance)
        * (1.0 + surface.shadowHidingAmplitude * shadowHiding(g, surface.shadowHidingWidth));
    float multipleScattering = (chandrasekharH(mu0 / porosity, float3(w), float3(diffusiveReflectance))
        * chandrasekharH(mu / porosity, float3(w), float3(diffusiveReflectance)) - 1.0).x;
    float backscatter = 1.0 + surface.backscatterAmplitude * coherentBackscatter(g, surface.backscatterWidth, porosity);
    float shadowing = roughSurface(mu0, mu, g, surface.roughness).shadowing;

    float3 reflectance = surface.albedo * 0.25
        * (porosity * lighting * (phaseAndShadowHiding + multipleScattering) * backscatter * shadowing);
    // Sol's values are tuned for a renderer where Lambert reflects albedo × cos(i) of the light; ours reflects
    // albedo / π × cos(i) per lux.
    return pow(max(reflectance, 0.0), gammaBoost) * lightBoost / hapkePi;
}

// Hapke's bidirectional reflectance (12.55) with porosity, both opposition effects and macroscopic roughness, per
// colour channel. Returns cd/m² per lux.
float3 hapkeTextbook(HapkeSurface surface, HapkeGeometry geometry)
{
    float3 w = clamp(surface.albedo, 0.0, 0.999);
    float3 gamma = sqrt(1.0 - w);
    float3 diffusiveReflectance = (1.0 - gamma) / (1.0 + gamma);
    float porosity = surface.porosity;
    float g = geometry.phaseAngle;

    RoughSurface rough = roughSurface(geometry.incidenceCosine, geometry.emissionCosine, azimuthAngle(geometry), surface.roughness);
    float mu0 = max(rough.incidenceCosine, 1e-4);
    float mu = max(rough.emissionCosine, 1e-4);

    float phaseAndShadowHiding = phaseFunction(g, surface.lobeWidth, surface.lobeBalance)
        * (1.0 + surface.shadowHidingAmplitude * shadowHiding(g, surface.shadowHidingWidth));
    float3 multipleScattering = chandrasekharH(mu0 / porosity, w, diffusiveReflectance)
        * chandrasekharH(mu / porosity, w, diffusiveReflectance) - 1.0;
    float backscatter = 1.0 + surface.backscatterAmplitude * coherentBackscatter(g, surface.backscatterWidth, porosity);

    return porosity * w / (4.0 * hapkePi) * mu0 / (mu0 + mu) * (phaseAndShadowHiding + multipleScattering) * backscatter
        * rough.shadowing;
}
