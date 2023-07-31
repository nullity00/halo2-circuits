use halo2_proofs::{circuit::*, plonk::*, poly::Rotation, arithmetic::Field};
use std::marker::PhantomData;

#[derive(Clone, Copy, Debug)]
struct FibonacciConfig {
  pub col_a : Column<Advice>,
  pub col_b : Column<Advice>,
  pub col_c : Column<Advice>,
  pub selector : Selector,
  pub instance : Column<Instance>
}

#[derive(Clone, Debug)]
struct FibonacciChip<F: Field> {
  config : FibonacciConfig,
  _marker : PhantomData<F>
}

impl<F: Field> FibonacciChip<F> {
    pub fn construct(config: FibonacciConfig) -> Self {
        Self {
            config,
            _marker: PhantomData,
        }
    }

    pub fn configure(meta: &mut ConstraintSystem<F>) -> FibonacciConfig{
      let col_a = meta.advice_column();
      let col_b = meta.advice_column();
      let col_c = meta.advice_column();
      let selector = meta.selector();
      let instance = meta.instance_column();

      meta.enable_equality(col_a);
      meta.enable_equality(col_b);
      meta.enable_equality(col_c);
      meta.enable_equality(instance);

      meta.create_gate("add", |meta|{

        let s = meta.query_selector(selector);
        let a = meta.query_advice(col_a, Rotation::cur());
        let b = meta.query_advice(col_b, Rotation::cur());
        let c = meta.query_advice(col_c, Rotation::cur());
        vec![s * ( a + b - c)]
      });

      FibonacciConfig { col_a, col_b, col_c, selector, instance}
    }

    pub fn assign_row(&self, mut layouter: impl Layouter<F>, nrows: usize)-> Result<AssignedCell<F, F>, Error>{
      layouter.assign_region(|| "fibo table", |mut region|{
        self.config.selector.enable(&mut region, 0)?;
        self.config.selector.enable(&mut region, 1)?;

        let mut a_cell = region.assign_advice(
          || "a",
          self.config.col_a,
          0,
          || Value::known(F::ZERO),
        )?;

        let mut b_cell = region.assign_advice(
          || "b",
          self.config.col_b,
          0,
          || Value::known(F::ONE),
        )?;

        for _i in 2..nrows {
          let c_cell = region.assign_advice(
            || "c",
            self.config.col_c,
            0,
            || a_cell.value().copied() + b_cell.value(),
          )?;
          a_cell = b_cell;
          b_cell = c_cell;
        }

        Ok(b_cell)
      },)
    }

    pub fn expose_public(
      &self,
      mut layouter: impl Layouter<F>,
      cell: AssignedCell<F, F>,
      row: usize,
    ) -> Result<(), Error> {
        layouter.constrain_instance(cell.cell(), self.config.instance, row)
    }
}


#[derive(Default)]

struct FiboCircuit<F>(PhantomData<F>);

impl<F: Field> Circuit<F> for FiboCircuit<F> {
    type Config = FibonacciConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        FibonacciChip::configure(meta)
    }

    fn synthesize(
        &self,
        config: Self::Config,
        mut layouter: impl Layouter<F>,
    ) -> Result<(), Error> {
      let chip  = FibonacciChip::construct(config);

      let out_cell = chip.assign_row(layouter.namespace(|| "entire table"), 10)?;

      chip.expose_public(layouter.namespace(|| "out"), out_cell, 2)?;

      Ok(())    }
}

#[cfg(test)]
mod tests {
    use std::marker::PhantomData;

    use super::FiboCircuit;
    use halo2_proofs::{dev::MockProver, pasta::Fp};

    #[test]
    fn fibonacci_example1() {
        let k = 4;

        let a = Fp::from(1); // F[0]
        let b = Fp::from(1); // F[1]
        let out = Fp::from(55); // F[9]

        let circuit = FiboCircuit(PhantomData);

        let mut public_input = vec![a, b, out];

        let prover = MockProver::run(k, &circuit, vec![public_input.clone()]).unwrap();
        prover.assert_satisfied();

        public_input[2] += Fp::one();
        let _prover = MockProver::run(k, &circuit, vec![public_input]).unwrap();
        // uncomment the following line and the assert will fail
        // _prover.assert_satisfied();
    }

    #[cfg(feature = "dev-graph")]
    #[test]
    fn plot_fibonacci1() {
        use plotters::prelude::*;

        let root = BitMapBackend::new("fib-1-layout.png", (1024, 3096)).into_drawing_area();
        root.fill(&WHITE).unwrap();
        let root = root.titled("Fib 1 Layout", ("sans-serif", 60)).unwrap();

        let circuit = MyCircuit::<Fp>(PhantomData);
        halo2_proofs::dev::CircuitLayout::default()
            .render(4, &circuit, &root)
            .unwrap();
    }
}