# Memory Safety, Binary, and Privileged Interface Patterns

Use this reference for C/C++, Rust `unsafe`, FFI, kernels, firmware, runtimes,
binary parsers, decoders, and privileged native interfaces.

## High-Value Classes

### Spatial Memory Errors

Re-derive allocation, element, offset, and copy units independently. Inspect
length subtraction, signed/unsigned conversion, multiplication overflow,
operator precedence, pointer-depth `sizeof` mistakes, fixed-buffer headroom,
terminators, and attacker-controlled loop bounds.

### Temporal Memory Errors

Trace every owner, alias, callback, timer, waiter, and view across release,
reallocation, cancellation, and error paths. Look for embedded wait structures
freed without draining, cached raw pointers after owner movement, and duplicated
dispatch paths with inconsistent retain/release behavior.

### Type and State Confusion

Check unchecked downcasts, tagged-union state, dispatch indexes, hierarchical
walkers, and code that reads one representation after validating another.
Determine whether the primitive enables controlled read, write, address
disclosure, or control flow rather than assuming impact.

### Uninitialized and Observable State

Compare actual initialized length with maximum buffer size, serialization size,
and attacker-controlled compare or copy length. Establish the observable oracle
or disclosure path.

### Privileged Interfaces and Double Fetch

For syscalls, ioctls, device nodes, management sockets, and FFI, verify both
request shape and caller authority. Identify facts read from mutable user memory
more than once across a validation/action boundary.

## Evidence Gate

1. show the exact attacker-controlled input and the native entrypoint
2. trace calculations and ownership to the faulting read, write, free, or dispatch
3. build a debuggable or sanitizer-enabled target when safe and in scope
4. use a minimal harness or fuzzer to confirm parser and memory behavior
5. distinguish a reproducible crash from a controlled security primitive
6. state architecture, allocator, build, and runtime assumptions
7. do not infer code execution from an out-of-bounds write without geometry and
   control evidence

Sanitizer silence is not proof of safety when the relevant access occurs in
uninstrumented assembly, JIT code, FFI, or within one allocation. Use an
additional observation method when the instrumentation boundary is incomplete.

## Primary References

- [CWE-787: Out-of-bounds Write](https://cwe.mitre.org/data/definitions/787.html)
- [CWE-416: Use After Free](https://cwe.mitre.org/data/definitions/416.html)
