<div align="center">

# Lolli ⊸

**Prove it linear, ship it safe.**

</div>

Lolli is a linear logic toolkit that turns resource specifications into verified Rust code. Describe how resources flow — what gets consumed, what gets produced, which operations are exclusive — and Lolli proves your specification is valid, extracts a program from the proof, and generates Rust where ownership enforces the invariants at compile time. No runtime checks, no manual discipline: if it compiles, the resources are handled correctly.

> [!NOTE]
> **Why "Lolli"?** 
> In linear logic, A ⊸ B is called the "lollipop" — consume A, produce B, exactly once. It's the connective that makes resource reasoning precise: no accidental copies, no forgotten cleanups, no use-after-free. Girard introduced it in 1987; Rust's ownership system operationalizes it today. Lolli is named for this symbol because the tool embodies what it represents: proving that resources flow correctly, then generating code that enforces it.
