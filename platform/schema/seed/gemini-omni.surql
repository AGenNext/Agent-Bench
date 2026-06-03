-- =============================================================================
-- Seed: run a bench on a real-world model. Subject = Gemini Omni (an image-
-- registry entry). Declarative only — the protocol/metrics are inputs (owned by
-- Agent-Metrics / the supplied protocol), Bench just orchestrates + measures.
-- Makes the "anything as a subject" path concrete end to end.
-- =============================================================================

-- --- Subject: Gemini Omni as a metamodel entity (image registry: a model) ----
UPSERT entity:`gemini-omni` SET
    did         = 'did:web:deepmind.google:models:gemini-omni',
    entity_type = 'model',
    name        = 'Gemini Omni',
    version     = 'next';

-- --- Attribute under test: cross-modal instruction following ------------------
-- Open like every other attribute; new attributes are rows, not code.
UPSERT attribute:`multimodal` SET key='multimodal', name='Cross-modal instruction following';
RELATE entity:`gemini-omni`->has_attribute->attribute:`multimodal`;

-- --- Protocol: supplied metric set + thresholds (formulas live in Agent-Metrics)
UPSERT protocol:`OMNI-001:1` SET
    key       = 'OMNI-001',
    version   = '1',
    attribute = 'multimodal',
    metrics   = [
        { key: 'cross_modal_fidelity',     direction: 'higher_is_better', threshold: { kind: 'gte',   value: 0.80 } },
        { key: 'instruction_adherence',    direction: 'higher_is_better', threshold: { kind: 'gte',   value: 0.85 } },
        { key: 'physics_consistency',      direction: 'higher_is_better', threshold: { kind: 'gte',   value: 0.70 } },
        { key: 'edit_turn_latency_ms',     direction: 'lower_is_better',  threshold: { kind: 'lte',   value: 4000 } }
    ];

-- --- Placement: cloud-agnostic, any registered conformant cluster -------------
UPSERT placement:`omni-any` SET providers=[], regions=[], failover='reschedule', max_retries=3;

-- --- Workload: kind=bench, the OCI image runs the multimodal eval harness -----
-- Reproducible from image@digest + inputs + protocol@version + placement.
UPSERT workload:`bench-gemini-omni` SET
    kind      = 'bench',
    subject   = entity:`gemini-omni`,
    protocol  = protocol:`OMNI-001:1`,
    image     = 'ghcr.io/agennext/harness-multimodal@sha256:0000000000000000000000000000000000000000000000000000000000000000',
    inputs    = { suite: 'omni-mixed-modality', modalities: ['text','image','audio','video'], turns: 8 },
    placement = placement:`omni-any`,
    budget    = { cpu: '4', mem: '16Gi', wall: '30m', cost: 5.00 };

RELATE workload:`bench-gemini-omni`->has_kind->workload_kind:bench;
RELATE workload:`bench-gemini-omni`->constrained_by->placement:`omni-any`;
