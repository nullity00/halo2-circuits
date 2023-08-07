use halo2_proofs::{
  plonk::{Advice, Circuit, Column, ConstraintSystem, Error, Expression, Fixed, Selector, Constraints},
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

    meta.create_gate("range_check", |meta| {
      let q = meta.query_selector(q_check);
      let value = meta.query_advice(value, Rotation::cur());
      let rc_polynomial = (1..RANGE).fold(value.clone(), |expr, i| {
        expr * (Expression::Constant(F::from(i as u64)) - value.clone())
      });

      Constraints::with_selector(q, [("range check", rc_polynomial)])
    });

    RangeConfig { value, q_check }
  }
}

