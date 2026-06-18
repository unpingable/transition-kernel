# NEXT

```
next_action: GAP-2 continuation — next frontier, await operator
```

The summit `stage3b2-first-effect` is sealed, and its follow-on **Stage 3c — composed receipt snapshot
coherence** is now built and verified (see `SUMMIT.md` / `CORRESPONDENCE.md` → Stage 3c): the composed
durable receipt pins the exact admission snapshot that governed the effect. The one owned completeness gap
in front of continuation is closed.

The frontier that advances the thesis is **GAP-2 — continuation**: the
kernel deciding whether a governed agent earns *another* governed step (not merely one action). A registry
of safely-constitutional file operations is ordinary engineering and does not advance the thesis.

GAP-2 should follow the same two-layer custody pattern, or it re-opens the leaf-authenticity seam (see
`NON_CLAIMS.md` / README "Trust perimeter") wearing a new hat — preventing grant *reuse* while still
accepting a caller-*fabricated* grant:

```
ContinuationGrantWire        # open representation from the issuing office
VerifiedContinuationGrant    # sealed result of issuer verification, consumer-bound
AuthorizedContinuation       # sealed composition with the prior chain tip
```

The `Wire → Verified → Authorized` gradient mirrors `LaCapability → (VerifiedLaCapability) →
AuthorizedTransition`: open at the wire, sealed where this office owns the check, composed only at the end.

Frozen claim: *at-most-once authority consumption and replay-legible execution of one idempotent bounded
effect through the live AG supervisor.*
