# Benchmarking Strategy for toy-arrow

This document outlines performance benchmarks aligned with each learning layer. Benchmarks validate design decisions and measure tradeoffs as implementation progresses.

**Philosophy**: Target metrics for key architectural decisions, compare against arrow-rs when possible.

---

## Layer 1: Memory Management & Buffers

### 1.1 Raw Buffer Fundamentals (`BufferInner`)

#### Benchmark 1.1.1: Allocation Overhead
**What**: Custom allocation cost vs `Vec<u8>` allocation
- **Metric**: Time to allocate N bytes (1KB, 1MB, 100MB)
- **Why**: Validates alignment justifies overhead
- **Expected**: Similar or better (alignment helps cache)
- **Size Variants**: 1KB, 1MB, 100MB
- **Implementation**: Compare `BufferInner::new()` vs `vec![0u8; size]`

#### Benchmark 1.1.2: Deallocation Safety
**What**: Proper deallocation with Layout cost
- **Metric**: Bulk allocation/deallocation cycles (10k iterations)
- **Why**: Ensure no leaks with custom allocator
- **Expected**: ~100ns per cycle (same as Vec)
- **Implementation**: Allocate, use, dealloc repeatedly

#### Benchmark 1.1.3: Pointer Arithmetic Optimization
**What**: Pointer storage vs offset arithmetic in hot loops
- **Metric**: Time to access elements in 1M-element buffer
- **Why**: Validates LLVM vectorization claim in README
- **Setup**: 
  - Implementation A: Store ptr directly, access via ptr + offset
  - Implementation B: Store offset, compute ptr each iteration
- **Expected**: A is 2-10% faster on SIMD-heavy access patterns
- **Size**: 1M i32 elements, sequential + random access patterns

#### Benchmark 1.1.4: Alignment Impact on SIMD
**What**: Memory layout affect on vectorized operations
- **Metric**: Throughput (GB/s) for SIMD operations (sum, min, max)
- **Why**: Understand alignment cost/benefit
- **Setup**:
  - Test with 64-byte alignment (arrow's default)
  - Test with natural alignment (8 bytes for i64)
- **Expected**: Aligned access 5-15% faster on SIMD loops
- **Implementation**: Use `#[repr(align(64))]` buffers

---

### 1.2 Immutable Buffer

#### Benchmark 1.2.1: Arc Cloning Overhead
**What**: Cost of zero-copy slice vs data copy
- **Metric**: Clone latency for various buffer sizes
- **Why**: Validates Arc-based sharing is free
- **Sizes**: 1KB, 1MB, 100MB
- **Expected**: Clone ~10ns (constant), independent of size
- **Implementation**: Measure `buffer.clone()` time

#### Benchmark 1.2.2: Slice Operations
**What**: `slice(offset, len)` cost
- **Metric**: Time to create N slices from 1M-element buffer
- **Why**: Zero-copy slicing should be O(1)
- **Expected**: ~50ns per slice
- **Implementation**: Create 1000 overlapping slices, measure total time

#### Benchmark 1.2.3: From<Vec<T>> Conversion
**What**: Reusing existing allocation cost
- **Metric**: Time to convert Vec of various sizes
- **Why**: Understand if reuse actually works
- **Sizes**: 1KB, 1MB, 100MB
- **Expected**: ~nanoseconds (just wrapping, no copy)
- **Implementation**: `Buffer::from(vec)` measurement

#### Benchmark 1.2.4: Reference Counting Contention
**What**: Arc behavior under concurrent access
- **Metric**: Operations/sec when cloning under contention (multi-threaded)
- **Why**: Identify when Arc becomes a bottleneck
- **Setup**: Spawn N threads, each repeatedly cloning same buffer
- **Expected**: Scale reasonably to ~8 cores
- **Sizes**: 1M element buffer, 8 threads

---

### 1.3 Mutable Buffer

#### Benchmark 1.3.1: Growth Strategy Performance
**What**: Cost of buffer reallocation during growth
- **Metric**: Time to append 1M elements one-by-one
- **Why**: Compare growth factors (1.5x, 2x, etc.)
- **Setup**: 
  - MutableBuffer with different growth factors
  - Compare vs Vec<u8>'s growth
- **Expected**: 1.5x growth = fewer allocations, 2x growth = less memory waste
- **Implementation**: Track allocation count + time

#### Benchmark 1.3.2: Reserve vs Append
**What**: Pre-allocation benefit
- **Metric**: Time to append 1M elements with vs without `reserve()`
- **Why**: Measure allocation savings
- **Expected**: ~50% faster with reserve
- **Implementation**: Two code paths, same final result

#### Benchmark 1.3.3: Freeze Conversion
**What**: `MutableBuffer::into_buffer()` cost
- **Metric**: Time to freeze (mutable → immutable)
- **Why**: Should be free (no copy)
- **Expected**: ~0ns (just Arc wrapping)
- **Implementation**: Measure conversion time

---

### 1.4 Boolean & Null Buffers

#### Benchmark 1.4.1: Bit-Packing Efficiency
**What**: Space + access cost of bit-packing vs byte-per-bool
- **Metric**:
  - Memory used: BooleanBuffer vs Vec<bool>
  - Access latency: Read 1M values
- **Why**: 8x memory claim needs validation
- **Setup**:
  - Store 10M booleans both ways
  - Sequential read all values
- **Expected**: 
  - Memory: 1.25MB (BooleanBuffer) vs 10MB (byte-per-bool)
  - Latency: BooleanBuffer 10-20% slower per-access, 95% faster per-byte
- **Implementation**: `BooleanBuffer::value(i)` in tight loop

#### Benchmark 1.4.2: NullBuffer Density Impact
**What**: Cost varies by sparsity of nulls
- **Metric**: Access + iteration speed by null density
- **Why**: Different densities need different strategies
- **Setup**: NullBuffer with 0%, 1%, 10%, 50%, 100% null values
- **Expected**: Sparse nulls (0-1%) much faster than dense
- **Implementation**: Measure iteration and `is_null()` calls

#### Benchmark 1.4.3: Bit Packing Construction
**What**: Speed of `pack_bools()` vs direct bool storage
- **Metric**: Time to pack 1M booleans from slice
- **Why**: Construction cost is one-time, worth optimizing
- **Expected**: ~100ns per MB for packing
- **Implementation**: Compare `pack_bools()` vs loop over Vec<bool>

#### Benchmark 1.4.4: Cached Null Count
**What**: O(1) null_count() vs scanning
- **Metric**: Time to call `null_count()` on large buffer
- **Why**: Validates caching benefit
- **Expected**: Cached ~5ns, scan ~100ms
- **Implementation**: Measure O(1) vs O(n) implementations

---

### 1.5 ScalarBuffer<T>

#### Benchmark 1.5.1: Type Safety Overhead
**What**: Generic `ScalarBuffer<T>` vs untyped `Buffer`
- **Metric**: Access latency for i32/f64 elements
- **Why**: Generics should have zero cost (monomorphization)
- **Expected**: Identical performance
- **Implementation**: Benchmark same operations on both

#### Benchmark 1.5.2: Slice Operation
**What**: `ScalarBuffer::slice()` to typed slice
- **Metric**: Time to create N typed slices
- **Why**: Should be O(1) like Buffer slice
- **Expected**: ~50ns per slice
- **Implementation**: Measure `slice(offset, len)` calls

---

### 1.6 OffsetBuffer<T>

#### Benchmark 1.6.1: Construction from Lengths
**What**: `OffsetBuffer::from_lengths()` performance
- **Metric**: Time to build offset buffer from 1M lengths
- **Why**: Common operation (strings, lists)
- **Expected**: O(n), ~10ns per length
- **Implementation**: From arrow-rs (offset.rs baseline)

#### Benchmark 1.6.2: Length Iteration
**What**: Cost to iterate derived lengths
- **Metric**: Time to read 1M lengths from OffsetBuffer
- **Why**: Iteration is frequent in filters/projections
- **Expected**: ~5ns per length
- **Implementation**: Measure `lengths()` iterator

#### Benchmark 1.6.3: Offset Validation
**What**: Cost of monotonic increase validation
- **Metric**: Time to validate 1M offsets (valid + invalid cases)
- **Why**: Trade-off between safety and speed
- **Expected**: ~15ns per offset (similar to construction)
- **Implementation**: Measure validation with black_box

---

### 1.7 Bit Utilities

#### Benchmark 1.7.1: get_bit() Performance
**What**: Speed of individual bit access
- **Metric**: Time to read 1M random bits
- **Why**: Core operation in BooleanBuffer
- **Expected**: ~2ns per bit (with branch prediction)
- **Implementation**: Tight loop with random indices

#### Benchmark 1.7.2: pack_bools() vs Loop
**What**: Optimized packing vs naive loop
- **Metric**: Time to pack 1M booleans
- **Why**: Validate optimization value
- **Expected**: Optimized 2-5x faster
- **Implementation**: From arrow-rs baseline

#### Benchmark 1.7.3: Bit Iteration Patterns
**What**: Different iteration strategies over bits
- **Metric**: Time to iterate 1M bits three ways:
  - Sequential `get_bit()` calls
  - 64-bit chunk reads
  - Custom `BitIterator`
- **Why**: Understand SIMD-friendly patterns
- **Expected**: Chunk iteration 10-100x faster
- **Implementation**: Compare three approaches

---

### 1.8 Buffer Operations (Bitwise)

#### Benchmark 1.8.1: Binary Operations
**What**: Bitwise AND/OR/XOR on null buffers
- **Metric**: Time to combine two 1M-bit buffers
- **Why**: Used in filter merging, NULL coalescing
- **Expected**: SIMD vectorized, ~2 cycles per 64 bits
- **Implementation**: From arrow-rs (buffer/ops.rs)

---

## Layer 2: Type System & Schema

### 2.1 DataType Representation

#### Benchmark 2.1.1: Type Enum Matching
**What**: Cost of pattern matching on DataType
- **Metric**: Time to dispatch on 10M type checks
- **Why**: Query planning does this frequently
- **Expected**: ~1ns per match (optimized enum)
- **Implementation**: Match on primitive vs nested types

---

## Phase Completion Criteria

Each phase is complete when:

1. **Correctness**: All tests pass
2. **Performance**: Benchmarks run and document expected behavior
3. **Arrow Parity**: Results within 10-20% of arrow-rs (where comparable)
4. **Documentation**: Findings added to README design questions

---

## Profiling Tools

**Local (macOS)**: Criterion for quick regression detection  
**Remote Linux**: Flamegraph, Cachegrind, dhat, perf for production-aligned profiling

### Tool Selection by Question

| Question | Tool | Platform | Notes |
|----------|------|----------|-------|
| Quick regression check? | **Criterion** | macOS local | Statistical feedback, not prod-comparable |
| Where does time go? | **Flamegraph** | Linux remote | Visual hot-path identification |
| Cache misses / SIMD? | **Cachegrind** | Linux remote | x86-64 cache analysis |
| Memory allocations? | **dhat** | Linux remote | Allocation profiling |
| CPU cycles / IPC? | **perf** | Linux remote | Hardware counters |

### 1. Criterion (Local macOS)

**Purpose**: Quick regression detection during development

**Running**:
```bash
cargo bench --bench buffers
cargo bench -- --save-baseline before
cargo bench -- --baseline before
```

**Limitation**: ARM64 results differ from x86-64 production. Use only for detecting large regressions locally.

**Output**: `target/criterion/` with graphs

---

### 2. Flamegraph (Remote Linux)

**Purpose**: Visual hot-path identification (where time actually goes)

**Running on remote**:
```bash
cargo install flamegraph
cargo flamegraph --bench buffers
scp remote:flamegraph.svg ./results/  # Copy back to macOS
```

**What to look for**:
- Width = time spent in function
- Height = call stack depth
- Expected hot paths: buffer access, bit operations
- x86-64: Look for SIMD vectorization (vpadd, vcmp, etc.)

**When to use**:
- Validating where time actually goes (pointer arithmetic vs offset)
- After optimization (confirm hot path improved)
- Comparing implementations (A vs B flamegraph)

---

### 3. Cachegrind (Remote Linux)

**Purpose**: Cache behavior, branch prediction, SIMD utilization

**Running on remote**:
```bash
cargo build --release --bench buffers
valgrind --tool=cachegrind ./target/release/deps/buffers-*
cg_annotate cachegrind.out.<pid> > cachegrind_report.txt
scp remote:cachegrind_report.txt ./results/
```

**Metrics to watch**:
- L1 cache miss rate (should be ~2-5%)
- LL cache miss rate (should be <1%)
- Branch misses (tight loops should predict well)

**When to use**:
- Validating SIMD alignment claims (1.1.4) - misaligned access spikes misses
- Bit-packing efficiency (1.4.1) - cache-optimal layout
- OffsetBuffer iteration (1.6.2) - contiguous access should be cache-friendly

---

### 4. dhat (Remote Linux)

**Purpose**: Memory allocation profiling with accurate sites

**Running on remote**:
```bash
cargo build --release --bench buffers
valgrind --tool=dhat ./target/release/deps/buffers-* > dhat.txt 2>&1
scp remote:dhat.txt ./results/
```

**Metrics**:
- Total allocations
- Peak memory usage
- Allocation sites (which function allocated most)

**When to use**:
- Validating bit-packing memory savings (1.4.1) - proves 8x reduction
- Builder growth patterns (1.3.1) - shows allocation frequency
- Mutable buffer reallocation (1.3.1) - tracks growth cost

---

### 5. perf (Remote Linux)

**Purpose**: Raw CPU counters - cycles, IPC, branch misses, cache events

**Running on remote**:
```bash
cargo build --release --bench buffers
perf stat ./target/release/deps/buffers-*
```

**Example output**:
```
Performance counter stats:
  5,234,123,456 cycles
  8,123,456,789 instructions
  1.55 insn per cycle
  123,456 cache-references
  12,345 cache-misses (9.98%)
```

**When to use**:
- Pointer vs offset arithmetic (1.1.3) - should see similar IPC
- SIMD efficiency (1.1.4) - vectorized code > scalar IPC
- Identifying unexpected bottlenecks (high cycle count for simple op)

---

## Running Benchmarks

**Local (macOS)**:
```bash
cargo bench --bench buffers
```

**Remote Linux**:
```bash
cargo flamegraph --bench buffers
valgrind --tool=cachegrind cargo build --release --bench buffers
valgrind --tool=dhat cargo build --release --bench buffers
perf stat ./target/release/deps/buffers-*
```

---

## Reference Benchmarks (arrow-rs)

Run these to establish baselines:
```bash
cd ~/Code/arrow-rs/arrow-buffer
cargo bench --bench bit_mask
cargo bench --bench offset
```

---

## Implementation Roadmap

### Phase 1: Buffer Fundamentals (1.1 - 1.3)
- [ ] 1.1.1: Allocation overhead
- [ ] 1.1.2: Deallocation safety
- [ ] 1.1.3: Pointer arithmetic (hottest path)
- [ ] 1.1.4: SIMD alignment impact
- [ ] 1.2.1: Arc cloning (validates core design)
- [ ] 1.2.2: Slice operations
- [ ] 1.3.1: Growth strategy

### Phase 2: Boolean/Null (1.4 - 1.7)
- [ ] 1.4.1: Bit-packing efficiency (space claim)
- [ ] 1.4.4: Cached null count
- [ ] 1.6.1: OffsetBuffer construction
- [ ] 1.7.1: get_bit() performance
- [ ] 1.7.3: Bit iteration patterns

### Phase 3: Advanced (1.8+)
- [ ] 1.8.1: Buffer bitwise operations
- [ ] 2.1.1: Type enum dispatch

---

## Design Decision Validation Matrix

| Decision | Benchmark | Pass Criteria | Status |
|----------|-----------|--------------|--------|
| Pointer instead of offset | 1.1.3 | 2-10% faster | 🔲 |
| Arc<BufferInner> sharing | 1.2.1 | ~10ns clone | 🔲 |
| Separate NullBuffer | 1.4.1 | 8x memory savings | 🔲 |
| Bit-packed booleans | 1.4.1 | 8x smaller | 🔲 |
| Cached null count | 1.4.4 | O(1) vs O(n) | 🔲 |
| OffsetBuffer for strings | 1.6.1 | Similar to arrow-rs | 🔲 |

---

## Notes

- Benchmarks use `black_box()` to prevent compiler optimizations hiding real costs
- All latency benchmarks run 1000+ iterations for stability
- SIMD benefits require `-C target-cpu=native` (check Cargo.toml with `[profile.bench]`)
- Focus on relative differences, not absolute numbers (vary by hardware)

---

## Updating This Document

As you complete benchmarks:
1. Run benchmark, record results in `benches/results/`
2. Update status checkboxes above
3. Add findings to README's "Key Design Questions" section
4. Note any surprises or deviations from expectations
