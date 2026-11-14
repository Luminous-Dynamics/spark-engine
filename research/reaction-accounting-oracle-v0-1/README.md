# Reaction accounting oracle v0.1

An isolated, dependency-free Rust reference implementation for fusion event-rate and reaction-power stoichiometry. It exists to independently check Spark issue #8; it is not a production reaction calculator.

## Mathematical contract

For unlike reactants:

`R = n_a n_b <sigma v>`

For identical reactants:

`R = n_a^2 <sigma v> / 2`

For a single reaction channel:

`P = R Q 1.602176634e-13`

where R is in reactions cm⁻³ s⁻¹, Q is in MeV per reaction, and P is in W cm⁻³.

D-D branches remain separate until the final sum:

`R_DD = n_D² (sv_n + sv_p) / 2`

`P_DD = n_D² (sv_n Q_n + sv_p Q_p) / 2 × 1.602176634e-13`

The oracle uses explicit channel identity and either one identical-species density or two named unlike-reactant densities. There is no universal symmetry multiplier and no API for silently reapplying branch fractions.

## Scope

- Channels: D-D neutron branch, D-D proton branch, D-T, D-He3, p-B11, p-Li6, p-Li7 and He3-He3.
- Typed density, reactivity and energy inputs.
- Positive, finite input validation.
- Explicit event-rate, reaction-power and symmetry-denominator outputs.
- D-D branch sum helper that preserves each branch's reactivity and Q value.
- Frozen known-answer and adversarial fixture corpus.
- Independent structural corpus audit that does not call the calculation functions.

The numerical reactivity and Q values are inputs. This crate does not calculate Bosch–Hale fits, screening, plasma conditions or reaction cross-sections.

## Run locally

```sh
cargo fmt --manifest-path Cargo.toml -- --check
cargo test --manifest-path Cargo.toml
cargo run --manifest-path Cargo.toml --bin verify_reaction_corpus
cargo clippy --manifest-path Cargo.toml --all-targets -- -D warnings
```

## Claim ceiling

A pass establishes only the declared algebraic stoichiometry, unit conversion and synthetic known-answer controls. It does not validate a reactivity fit, a physical plasma, fusion experiment, reactor, neutron source, net energy or hardware.
