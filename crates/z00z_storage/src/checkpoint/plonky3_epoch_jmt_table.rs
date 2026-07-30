//! Batch-STARK table registration for the canonical epoch JMT AIR.

use p3_circuit::ops::NpoTypeId;
use p3_circuit::tables::Traces;
use p3_field::extension::BinomialExtensionField;
use p3_field::PrimeCharacteristicRing;
use p3_koala_bear::KoalaBear;
use z00z_plonky3_circuit_prover::batch_stark_prover::{
    BatchTableInstance, DynamicAirEntry, NonPrimitiveTableEntry, TablePacking, TableProver,
};

use super::plonky3_epoch_jmt_air::{
    jmt_npo_type, JmtAirV2, JmtTraceV2, CALL_FIELDS_V2, PUBLIC_FIELDS_V2,
};
use super::Plonky3StarkConfigV2;

#[derive(Clone, Copy, Debug)]
pub(super) struct JmtProverV2;

impl JmtProverV2 {
    fn batch_instance(
        &self,
        packing: &TablePacking,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        let trace = traces.non_primitive_trace::<JmtTraceV2>(&jmt_npo_type())?;
        if trace.rows.is_empty()
            || trace
                .rows
                .iter()
                .any(|row| row.values.len() != CALL_FIELDS_V2)
        {
            return None;
        }
        let preprocessed = vec![KoalaBear::ONE; trace.rows.len()];
        let min_height = packing.min_trace_height().max(trace.rows.len());
        let air = JmtAirV2::<KoalaBear, 1>::new(preprocessed, min_height);
        Some(BatchTableInstance {
            op_type: jmt_npo_type(),
            air: DynamicAirEntry::new(Box::new(air)),
            trace: JmtAirV2::<KoalaBear, 1>::trace_to_matrix(&trace.rows, min_height),
            public_values: trace.rows[0].values[..PUBLIC_FIELDS_V2].to_vec(),
            rows: trace.rows.len(),
            lanes: 1,
        })
    }
}

impl TableProver<Plonky3StarkConfigV2> for JmtProverV2 {
    fn op_type(&self) -> NpoTypeId {
        jmt_npo_type()
    }

    fn batch_instance_d1(
        &self,
        _config: &Plonky3StarkConfigV2,
        packing: &TablePacking,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        self.batch_instance(packing, traces)
    }

    fn batch_instance_d2(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 2>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_instance_d4(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 4>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_instance_d6(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 6>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_instance_d8(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        _traces: &Traces<BinomialExtensionField<KoalaBear, 8>>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        None
    }

    fn batch_air_from_table_entry(
        &self,
        _config: &Plonky3StarkConfigV2,
        degree: usize,
        circuit_extension_degree: u32,
        entry: &NonPrimitiveTableEntry<Plonky3StarkConfigV2>,
    ) -> Result<DynamicAirEntry<Plonky3StarkConfigV2>, String> {
        if degree != 1
            || circuit_extension_degree != 1
            || entry.rows == 0
            || entry.lanes != 1
            || entry.public_values.len() != PUBLIC_FIELDS_V2
        {
            return Err("epoch JMT update table shape mismatch".into());
        }
        Ok(DynamicAirEntry::new(Box::new(
            JmtAirV2::<KoalaBear, 1>::new(Vec::new(), entry.rows),
        )))
    }

    fn air_with_committed_preprocessed(
        &self,
        committed: Vec<KoalaBear>,
        min_height: usize,
        lanes: usize,
        circuit_extension_degree: u32,
    ) -> Option<DynamicAirEntry<Plonky3StarkConfigV2>> {
        (lanes == 1
            && circuit_extension_degree == 1
            && !committed.is_empty()
            && committed.iter().all(|&value| value == KoalaBear::ONE))
        .then(|| {
            DynamicAirEntry::new(Box::new(JmtAirV2::<KoalaBear, 1>::new(
                committed, min_height,
            )))
        })
    }
}
