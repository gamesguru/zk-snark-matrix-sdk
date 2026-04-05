# Architectural Paths for Matrix ZK-State Resolution

This document outlines the three distinct architectural paths to optimize the `ruma-state-res` Directed Acyclic Graph (DAG) traversal in a zkVM environment. They range from pragmatically bypassing VM memory bottlenecks to writing custom lowest-level polynomial constraints.

## Path A: Host-Offloading (Pragmatic Hack)

_Using standard SP1 RISC-V, but bypassing memory lookups in the guest._

### Concept

Run the heavy DAG traversal (like Kahn's Topological Sort or standard State Res v2) outside the zkVM on the host OS, and only verify the resulting topology inside the zkVM.

### Why it's useful

It provides an immediate cycle-count reduction in the RISC-V trace length, solving the LogUp memory bloat associated with hashing and dereferencing complex memory structures (`HashMap`, `BTreeMap`). This gives us concrete benchmark reductions (e.g. going from tens of millions of cycles to a tiny fraction of that) for theoretical performance bounds, proving the core hypothesis before diving into specialized compiler engineering.

### Workflow

1. **Host:** Execute `ruma-state-res::resolve()` natively. Extract the flattened topological array representing the correct state graph, bypassing deep HashMap traversal inside the VM. Serialize the array and pass it to the Guest.
2. **Guest:** Read the flattened edge list. Use a linear `for` loop to verify `hash(current) == next.parent_hash`. By checking $\mathcal{O}(1)$ edges sequentially, we eliminate pointer chasing.

---

## Path B: Custom SP1 Precompile (Hybrid Coprocessor)

_Using SP1 RISC-V, but offloading the DAG check to a hardware-accelerated Plonky3 Coprocessor._

### Concept

Modify the zkVM's core executor and machine to include a specialized `SYS_TOPOLOGICAL_ROUTE` syscall. This allows standard Rust code to run in the VM for string parsing and networking, but offloads the core topographical verification logic out of the RISC-V memory bus and into pure math.

### Why it's useful

This is the ultimate "production-grade" solution capable of running natively. It provides $\mathcal{O}(1)$ constraints over each edge, combining general-purpose language tools with specialized ASIC-like performance for exactly the hot loop.

### Workflow

1. **Fork SP1.**
2. **Define Syscall:** Add `SYS_TOPOLOGICAL_ROUTE` to bridge standard Rust execution directly into the coprocessor.
3. **Execution Logic:** Create a custom event trace vector in the CPU builder to log whenever the guest invokes the syscall.
4. **Plonky3 Constraints:** Write a `MachineAir` trait that takes the trace vector and constrains polynomial relations natively without HashMap abstractions. (e.g., degree-2 adjacency row relations).

---

## Path C: Pure Plonky3 Circuit (No RISC-V)

_Bypassing the VM completely._

### Concept

Abandon the concept of a RISC-V virtual machine or general-purpose language compiler and write constraints directly over a STARK prover field.

### Why it's useful (and its limitations)

While this is theoretically the fastest and most memory-efficient possible design for proving an algorithm, it lacks any capacity for standard Rust workflows (JSON parsing, API networking). You cannot easily parse strings or dynamically allocate structs. Everything is represented as raw mathematical fields.

### Workflow

1. Implement `p3_air::Air` directly in the `plonky3` ecosystem.
2. Define exact constraint identities over a custom trace layout for DAG merging.
3. Manually wire up the `p3_commit` and `p3_fri` architecture to generate the final zero-knowledge proof.
