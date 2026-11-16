//! Body-safe transport mapping for retained project instruction originals.
use chapchoo_domain::InstructionSourceKind;
use chapchoo_protocol::conversations as wire;

pub(crate) fn evidence(
    value: chapchoo_service::ProjectInstructionEvidence,
) -> wire::ProjectInstructionEvidence {
    wire::ProjectInstructionEvidence {
        agent_id: value.agent_id,
        conversation_id: value.conversation_id,
        operation_id: value.operation_id,
        execution_id: value.execution_id,
        sources: value
            .instructions
            .sources
            .into_iter()
            .map(|source| wire::InstructionSource {
                kind: match source.kind {
                    InstructionSourceKind::Product => wire::InstructionSourceKind::Product,
                    InstructionSourceKind::Function => wire::InstructionSourceKind::Function,
                    InstructionSourceKind::Owner => wire::InstructionSourceKind::Owner,
                    InstructionSourceKind::Execution => wire::InstructionSourceKind::Execution,
                } as i32,
                text: source.text,
            })
            .collect(),
        rendered_document: value.rendered_document,
        created_at: value.created_at,
        tool_catalog: value.tool_catalog,
        applied_target: value.applied_target.map(|target| wire::NativeTarget {
            snapshot_id: target.snapshot.as_str().into(),
            native_session_id: target.native_session.as_str().into(),
        }),
        applied_at: value.applied_at,
    }
}
