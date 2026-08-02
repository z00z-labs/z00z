//! Batch-STARK registration for the packed canonical event-byte source.

use p3_circuit::tables::Traces;
use p3_field::extension::BinomialExtensionField;
use p3_koala_bear::KoalaBear;
use z00z_plonky3_circuit_prover::batch_stark_prover::{
    BatchTableInstance, DynamicAirEntry, NonPrimitiveTableEntry, TablePacking, TableProver,
};

use super::plonky3_epoch_event_source_columns::{
    EventSourceAirRoleV2, EventSourceAirV2, EventSourceTraceV2, CALL_FIELDS_V2, PUBLIC_FIELDS_V2,
};
use super::Plonky3StarkConfigV2;

#[derive(Clone, Copy, Debug)]
pub(super) struct EventSourceProverV2 {
    role: EventSourceAirRoleV2,
}

impl EventSourceProverV2 {
    pub(super) const fn new(role: EventSourceAirRoleV2) -> Self {
        Self { role }
    }

    fn batch_instance(
        &self,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        let trace = traces.non_primitive_trace::<EventSourceTraceV2>(&self.role.npo_type())?;
        if trace.role != self.role
            || trace.rows.is_empty()
            || trace.public_values.len() != PUBLIC_FIELDS_V2
            || trace
                .rows
                .iter()
                .any(|row| row.values.len() != CALL_FIELDS_V2)
        {
            return None;
        }
        Some(BatchTableInstance {
            op_type: self.role.npo_type(),
            air: DynamicAirEntry::new(Box::new(EventSourceAirV2::new(self.role))),
            trace: EventSourceAirV2::trace_to_matrix(&trace.rows),
            public_values: trace.public_values.clone(),
            rows: trace.rows.len(),
            lanes: 1,
        })
    }
}

impl TableProver<Plonky3StarkConfigV2> for EventSourceProverV2 {
    fn op_type(&self) -> p3_circuit::ops::NpoTypeId {
        self.role.npo_type()
    }

    fn batch_instance_d1(
        &self,
        _config: &Plonky3StarkConfigV2,
        _packing: &TablePacking,
        traces: &Traces<KoalaBear>,
    ) -> Option<BatchTableInstance<Plonky3StarkConfigV2>> {
        self.batch_instance(traces)
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
            || entry.op_type != self.role.npo_type()
        {
            return Err("epoch packed event-source table shape mismatch".into());
        }
        Ok(DynamicAirEntry::new(Box::new(EventSourceAirV2::new(
            self.role,
        ))))
    }
}
