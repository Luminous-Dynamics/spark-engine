# Materials-Mediated Fusion Benchmark Data Contract

**Status:** Executable schema foundation; pointwise paper data not yet ingested  
**Scope:** Spark Engine materials-mediated D–D benchmark corpus  
**Date:** 2026-11-14

## Purpose

The benchmark corpus must preserve what a source actually measured, the experiment class, the measurement unit and frame, the normalization reference, uncertainty status, data rights and exact source location.

This contract deliberately does not turn a published figure into fabricated pointwise data. A record is admitted only when its value can be tied to a source locator and its measurement semantics pass validation.

Implementation: src/benchmark.rs.

## Why the experiment classes are separate

The initial literature includes materially different experiment types:

1. Beam-driven metal-target measurements.
2. X-ray-triggered lattice experiments.
3. Electrochemical loading controls without a declared incident-ion beam.
4. Other experiment classes.

These cannot be combined into one undifferentiated "LCF yield" dataset. In particular, the beam-driven metal-hydride studies published in 2025 and 2026 are not replications of a specific X-ray-triggered NASA LCF configuration.

## Measurement semantics

| Quantity | Required unit | Required basis / context |
|---|---|---|
| Incident energy | eV or keV | Explicit laboratory or centre-of-mass frame |
| Relative yield | Dimensionless | Named normalization reference |
| Hydride-to-metal yield ratio | Dimensionless | Named control |
| Absolute yield | Events per incident ion | Absolute |
| Total detector count rate | Counts per second | Absolute |
| Areal detector count rate | Counts per second per cm² | Absolute |
| Total reaction rate | Reactions per second | Absolute |
| Volumetric reaction rate | Reactions per second per cm³ | Absolute |
| Calorimetric power | Watt | Absolute; calorimetry channel only |
| Loading ratio | Deuterium per host atom | Absolute |
| Enhancement ratio | Dimensionless | Named model/reference calculation |

A model-relative enhancement ratio is not a reaction rate, thermal power, Q-factor or net-electric output. Plant energy boundaries are owned separately by Symtropy's typed energy-port work.

## Provenance and uncertainty

Every record carries:
- a citation and DOI, stable URL or content digest;
- a locator identifying the exact figure, table, page or artifact;
- content SHA-256 for raw datasets and digitized-figure artifacts;
- data-rights status;
- experiment class, host material, isotope pair and loading state;
- detection channel;
- explicit uncertainty status (Reported, NotReported, NotApplicable);
- uncertainty magnitude, interpretation and coverage probability where required.

Unknown uncertainty remains unknown. It is not zero. A relative number is never admitted without its reference identifier. An incident energy is never admitted without its frame.

## Current source anchors

- Chen et al., *Nature* (2025), [10.1038/s41586-025-09042-7](https://doi.org/10.1038/s41586-025-09042-7). The publication reports a 15(2)% electrochemical-loading-related increase in beam-driven D–D fusion rate in a palladium target. The paper is a literature source, not a Spark replication.
- Karahadian et al., *Nature Communications* (2026), [10.1038/s41467-026-74421-1](https://doi.org/10.1038/s41467-026-74421-1). The paper reports a low-energy yield plateau in palladium and titanium hydrides. The reported enhancement above 10¹⁸ is relative to bare-nucleus expectations, not net energy gain.

No raw pointwise values are embedded by this change. They must be ingested from an admissible source artifact or digitized with a versioned, hashed extraction artifact and rights review.

## Schema acceptance cases

The Rust unit tests require the contract to reject:
- beam-driven records without incident energy or loading state;
- incident energy without a reference frame;
- dimensionally invalid units;
- relative yield without a named normalization reference;
- hydride-to-metal ratio without a named control;
- model-relative enhancement represented as a power unit;
- areal detector count rate confused with total detector count rate;
- volumetric reaction rate confused with total reaction rate;
- calorimetric power assigned to a particle detector channel;
- model output without explicit model/revision identity;
- raw dataset or digitized figure without a content digest;
- non-finite values;
- uncertainty payload/status disagreement;
- incomplete expanded-uncertainty coverage metadata.

The tests use synthetic fixtures only. Passing them validates the data contract, not the underlying physics model or a physical reactor.

## Next benchmark increments

1. Admit the two papers as externally reported source records with exact figure/table locators.
2. Obtain/review raw datasets or digitize selected figures into content-addressed artifacts.
3. Add independent reference curves and unit/frame conversion tests.
4. Freeze train/validation/hold-out splits before fitting.
5. Report held-out log-yield error, residuals, interval coverage and model complexity.
6. Add explicit negative controls and model-misspecification cases.
