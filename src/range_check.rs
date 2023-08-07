use halo2_proofs::{
  plonk::{
    Advice,
    Circuit,
    Column,
    ConstraintSystem,
    Error,
    Expression,
    Selector,
    Constraints,
    Assigned,
  },
  circuit::*,
  poly::Rotation,
};

use group::ff::PrimeField;

use std::marker::PhantomData;

#[derive(Clone, Debug)]
struct RangeConfig {
  value: Column<Advice>,
  q_check: Selector,
}

#[derive(Debug, Clone)]
struct RangeChip<F: PrimeField, const RANGE: usize> {
  config: RangeConfig,
  _marker: PhantomData<F>,
}

impl<F: PrimeField, const RANGE: usize> RangeChip<F, RANGE> {
  pub fn construct(config: RangeConfig) -> Self {
    Self {
      config,
      _marker: PhantomData,
    }
  }

  pub fn configure(meta: &mut ConstraintSystem<F>) -> RangeConfig {
    let value = meta.advice_column();
    let q_check = meta.selector();

    meta.enable_equality(value);

    meta.create_gate("range_check_gate", |meta| {
      let q = meta.query_selector(q_check);
      let value = meta.query_advice(value, Rotation::cur());
      let range_check_poly = (1..RANGE).fold(value.clone(), |expr, i| {
        expr * (Expression::Constant(F::from(i as u64)) - value.clone())
      });

      Constraints::with_selector(q, [("range check constraint", range_check_poly)])
    });

    RangeConfig { value, q_check }
  }

  pub fn assign(
    &self,
    mut layouter: impl Layouter<F>,
    value: Value<Assigned<F>>
  ) -> Result<(), Error> {
    layouter.assign_region(
      || "range check region",
      |mut region| {
        self.config.q_check.enable(&mut region, 0)?;

        region.assign_advice(
          || "value",
          self.config.value,
          0,
          || value
        )
      }
    )?;

    Ok(())
  }
}

#[derive(Default)]
struct RangeCircuit<F: PrimeField, const RANGE: usize> {
  assigned_value: Value<Assigned<F>>,
  _marker: PhantomData<F>,
}

impl<F: PrimeField, const RANGE: usize> Circuit<F> for RangeCircuit<F, RANGE> {
  type Config = RangeConfig;
  type FloorPlanner = SimpleFloorPlanner;

  fn without_witnesses(&self) -> Self {
    Self::default()
  }

  fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
    RangeChip::<F, RANGE>::configure(meta)
  }

  fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
    let chip = RangeChip::<F, RANGE>::construct(config);
    chip.assign(
      layouter.namespace(|| "value"),
      self.assigned_value
    )?;

    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use halo2_proofs::{ dev::{ FailureLocation, MockProver, VerifyFailure }, pasta::Fp, plonk::* };

  use super::*;

  #[test]
  fn test_range_check_1() {
    let k = 4;
    const RANGE: usize = 8;

    // Successful cases
    for i in 0..RANGE {
      let circuit = RangeCircuit::<Fp, RANGE> {
        assigned_value: Value::known(Fp::from(i as u64).into()),
        _marker: PhantomData,
      };

      let prover = MockProver::run(k, &circuit, vec![]).unwrap();
      prover.assert_satisfied();
    }
  }

  #[test]
  fn test_range_check_fail() {
    let k = 4;
    const RANGE: usize = 8;
    let testvalue: u64 = 22;

    // Out-of-range `value = 8`
    {
      let circuit = RangeCircuit::<Fp, RANGE> {
        assigned_value: Value::known(Fp::from(testvalue).into()),
        _marker: PhantomData,
      };
      let prover = MockProver::run(k, &circuit, vec![]).unwrap();
      assert_eq!(
        prover.verify(),
        Err(
          vec![VerifyFailure::ConstraintNotSatisfied {
            constraint: ((0, "range_check_gate").into(), 0, "range check constraint").into(),
            location: FailureLocation::InRegion {
              region: (0, "range check region").into(),
              offset: 0,
            },
            cell_values: vec![(((Any::Advice, 0).into(), 0).into(), "0x16".to_string())],
          }]
        )
      );
    }
  }
}
