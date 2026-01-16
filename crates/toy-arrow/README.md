# toy-arrow: Learning Apache Arrow Bottom-Up

A learning implementation of Apache Arrow in Rust, built from scratch to understand the internals of the arrow-rs production library.

## Learning Philosophy

**Approach**: Read arrow-rs implementation → Understand "why" behind design → Implement simplified version → Learn through problems

**Reference**: arrow-rs at `~/Code/arrow-rs`

---

## Architecture Layers (Bottom-Up)

This roadmap follows the dependency hierarchy of arrow-rs, starting from the most foundational concepts and building up to advanced features.

---

## Layer 1: Memory Management & Buffers

**Crate**: `arrow-buffer`

Maps to arrow-rs structure:
- `buffer/` - Buffer types (immutable, mutable, scalar, boolean, null, offset, run, ops)
- `builder/` - Buffer builders
- `native.rs` - ArrowNativeType trait
- `alloc/` - Allocation infrastructure
- `util/` - Bit manipulation utilities
- `bytes.rs` - Internal Bytes struct (like our BufferInner)

---

### 1.1 Raw Buffer Fundamentals (`bytes.rs` / `BufferInner`)
- [x] Understanding alignment requirements for typed access
- [x] Why `Vec<u8>` is insufficient (alignment guarantees)
- [x] Custom allocation with `std::alloc::Layout`
- [x] `NonNull<u8>` for pointer safety
- [x] Memory safety: proper deallocation with `Layout`

```rust
// Our BufferInner (arrow-rs calls this Bytes)
struct BufferInner {
    ptr: NonNull<u8>,
    len: usize,
    capacity: usize,
    layout: Layout,
}
```

**Key Questions**:
- Why does arrow-rs store pointers instead of offsets in Buffer?
- How does alignment affect SIMD vectorization?
- What's the contract between `alloc()` and `dealloc()`?

---

### 1.2 Immutable Buffer (`buffer/immutable.rs`)
- [x] `Arc<BufferInner>` for reference counting
- [x] Zero-copy slicing (offset + length tracking)
- [x] Clone without data copy
- [x] Drop behavior with last reference
- [x] `slice(offset, len)` method
- [ ] `is_empty()` method
- [ ] `ptr_eq()` for pointer comparison
- [ ] `shrink_to_fit()` to free unused memory

---

### 1.3 Mutable Buffer (`buffer/mutable.rs`)
- [ ] `MutableBuffer`: Growing buffer during construction
- [ ] `with_capacity()` pre-allocation
- [ ] `push()`, `extend_from_slice()` for appending
- [ ] `reserve()` for capacity management
- [ ] `into_buffer()` conversion to immutable `Buffer`
- [ ] `freeze()` pattern (mutable → immutable)

**Pattern**: Mutable construction → immutable use

---

### 1.4 ArrowNativeType Trait (`native.rs`)
- [x] Sealed trait for safe primitive types
- [x] Implemented for: i8, i16, i32, i64, u8, u16, u32, u64, f32, f64
- [x] `get_byte_width()` method
- [x] `get_alignment()` method
- [ ] `from_usize()`, `to_usize()` conversions
- [x] Enables generic `ScalarBuffer<T>` and typed access

```rust
pub trait ArrowNativeType: Debug + Send + Sync + Copy + 'static {
    fn get_byte_width() -> usize;
    // ...
}
```

**Key Design**: Sealed trait prevents external implementations, ensuring safety

---

### 1.5 ScalarBuffer<T> (`buffer/scalar.rs`)
- [x] Type-safe wrapper over `Buffer`
- [x] Generic over `NativeType`
- [x] `slice()` returns typed slice
- [x] `Deref` to `&[T]` for easy access
- [x] Zero-copy conversion from `Vec<T>`
- [x] `Clone` for cheap copies

```rust
pub struct ScalarBuffer<T: ArrowNativeType> {
    buffer: Buffer,
    phantom: PhantomData<T>,
}
```

**Replaces**: Our current `as_i32_slice()`, `as_u8_slice()` with generic approach

---

### 1.6 BooleanBuffer (`buffer/boolean.rs`)
- [x] Bit-packed boolean storage (1 bit per value)
- [x] Separate from NullBuffer (this is for boolean array values)
- [x] `len()` in bits, not bytes
- [x] `value(i)` to read individual bits
- [x] `from_bools()` constructor with bit-packing
- [ ] `set_bit()`, `unset_bit()` for mutation (via builder)
- [ ] `count_set_bits()` efficient counting
- [ ] Slicing with bit offset support

**Space Efficiency**: 8x memory savings vs byte-per-bool

**Design Note**: NullBuffer wraps BooleanBuffer internally, adding only cached `null_count` and inverted semantics.

---

### 1.7 NullBuffer (`buffer/null.rs`)
- [x] Validity bitmap with cached null count
- [x] Convention: `true` = valid, `false` = null
- [x] `null_count()` in O(1) (cached on construction)
- [x] `is_null(idx)` method
- [x] `from_bools()` constructor with bit-packing
- [ ] `union()` / `intersect()` operations for combining masks
- [ ] Efficient construction from iterators
- [ ] `contains_nulls()` quick check

---

### 1.8 OffsetBuffer<T> (`buffer/offset.rs`)
- [ ] For variable-length types (strings, lists)
- [ ] Generic over offset type (i32 for regular, i64 for Large variants)
- [ ] Validates monotonically increasing offsets
- [ ] `lengths()` iterator over element lengths
- [ ] Bounds checking on construction

```rust
pub struct OffsetBuffer<T: ArrowNativeType>(ScalarBuffer<T>);
```

---

### 1.9 RunEndBuffer<T> (`buffer/run.rs`)
- [ ] For run-length encoding
- [ ] Stores indices where runs end
- [ ] `get_physical_index()` to map logical → physical
- [ ] `get_run_end()` for a given index

---

### 1.10 Buffer Operations (`buffer/ops.rs`)
- [ ] `buffer_bin_and()` - bitwise AND
- [ ] `buffer_bin_or()` - bitwise OR
- [ ] `buffer_bin_xor()` - bitwise XOR
- [ ] `buffer_unary_not()` - bitwise NOT
- [ ] Operations on boolean/null buffers

---

### 1.11 Bit Utilities (`bit_util.rs`)
- [x] `get_bit()`: Read individual bit from buffer
- [x] `pack_bools()`: Pack boolean slice into bit-packed buffer
- [ ] `set_bit()`: Set individual bit in buffer
- [ ] `ceil()`, `round_upto_multiple_of_64()`
- [ ] `BitIterator` for iterating over bits
- [ ] `BitChunks` for 64-bit chunk iteration (SIMD friendly)

---

### 1.12 Buffer Builders (`builder/`)

**BufferBuilder<T>** (`builder/mod.rs`):
- [ ] Generic builder for `ScalarBuffer<T>`
- [ ] `append()`, `append_slice()`, `append_n()`
- [ ] `finish()` → `ScalarBuffer<T>`

**BooleanBufferBuilder** (`builder/boolean.rs`):
- [ ] Bit-level append operations
- [ ] `append()`, `append_n()`, `append_slice()`
- [ ] `finish()` → `BooleanBuffer`

**NullBufferBuilder** (`builder/null.rs`):
- [ ] Tracks validity during construction
- [ ] `append_null()`, `append_non_null()`, `append_n()`
- [ ] `finish()` → `Option<NullBuffer>` (None if all valid)

**OffsetBufferBuilder** (`builder/offset.rs`):
- [ ] Builds offset arrays for variable-length types
- [ ] `push_length()` to add element lengths
- [ ] `finish()` → `OffsetBuffer<T>`

---

### 1.13 Allocation Infrastructure (`alloc/`)
- [x] Custom allocation with `Layout` (in BufferInner)
- [ ] `ALIGNMENT` constant (64 bytes for SIMD)
- [ ] `Deallocation` enum (Standard vs Custom/FFI)
- [ ] `Allocation` trait for custom allocators

**Skip for learning**: FFI allocation support (Custom variant)

---

### 1.14 Specialized Types (Optional)
- [ ] `i256` (`bigint/`) - 256-bit integer for Decimal256
- [ ] `IntervalMonthDayNano`, `IntervalDayTime` (`interval.rs`)
- [ ] Checked/wrapping arithmetic macros (`arith.rs`)

**Skip for learning**: These are domain-specific, implement if needed later

---

## Layer 2: Type System & Schema

**Crate**: `arrow-schema`

### 2.1 DataType Enum
- [ ] Understanding Arrow's logical type system
- [ ] Primitive types: `Int8`, `Int32`, `Float64`, etc.
- [ ] Temporal types: `Timestamp`, `Date32`, `Duration`
- [ ] Binary types: `Utf8`, `Binary`, `LargeUtf8`
- [ ] Nested types: `List`, `Struct`, `Map`
- [ ] Special types: `Dictionary`, `Union`, `RunEndEncoded`

**Key Insight**: DataType describes logical structure, not physical layout

### 2.2 Field & Schema
- [ ] `Field`: Column descriptor (name, type, nullable, metadata)
- [ ] `Schema`: Collection of fields for table structure
- [ ] Schema metadata for custom properties
- [ ] `SchemaBuilder` pattern

### 2.3 Type Marker Pattern
- [ ] Zero-sized types for compile-time type safety
- [ ] `ArrowPrimitiveType` trait connecting Rust types to Arrow DataTypes
- [ ] Type aliases: `Int32Type`, `Float64Type`, etc.

```rust
pub trait ArrowPrimitiveType: 'static {
    type Native: ArrowNativeTypeOp;
    const DATA_TYPE: DataType;
}

// Usage enables static dispatch
struct PrimitiveArray<T: ArrowPrimitiveType> { ... }
```

**Design Genius**: Static dispatch with type safety, no runtime overhead

---

## Layer 3: Array Data Representation

**Crate**: `arrow-data`

### 3.1 Low-Level ArrayData
- [ ] Understanding physical memory layout
- [ ] `ArrayData` structure: buffers + child_data + nulls
- [ ] Offset into buffers for slicing
- [ ] Validation: ensuring Arrow spec compliance

```rust
struct ArrayData {
    data_type: DataType,
    len: usize,
    offset: usize,
    buffers: Vec<Buffer>,
    child_data: Vec<ArrayData>,  // For nested types
    nulls: Option<NullBuffer>,
}
```

**Safety**: `ArrayData` is unsafe to construct directly

### 3.2 ArrayDataBuilder
- [ ] Fluent builder API
- [ ] Validation at construction (`try_new()`)
- [ ] Unchecked construction for trusted sources
- [ ] Buffer layout per data type

**Learning Exercise**: How many buffers does each type need?
- Primitive: 1 (values) + optional null buffer
- String: 2 (offsets + values) + optional null buffer
- List: 1 (offsets) + child array + optional null buffer

---

## Layer 4: Primitive & Fixed-Size Arrays

**Crate**: `arrow-array`

### 4.1 PrimitiveArray<T>
- [ ] Generic over `ArrowPrimitiveType`
- [ ] Wraps `ArrayData` with typed access
- [ ] `values()` returns `&ScalarBuffer<T::Native>`
- [ ] `null_count()`, `is_null(i)`, `is_valid(i)`
- [ ] Type aliases: `Int32Array`, `Float64Array`

**Key Methods**:
```rust
impl<T: ArrowPrimitiveType> PrimitiveArray<T> {
    fn value(&self, i: usize) -> T::Native;
    fn values(&self) -> &ScalarBuffer<T::Native>;
    fn iter(&self) -> impl Iterator<Item = Option<T::Native>>;
}
```

### 4.2 BooleanArray
- [ ] Special handling for bit-packed storage
- [ ] Two buffers: null bitmap + value bitmap
- [ ] `value(i)` extracts single bit
- [ ] Efficient logical operations

### 4.3 FixedSizeBinaryArray
- [ ] Fixed-width byte sequences (e.g., UUID, hash)
- [ ] Single buffer with stride access
- [ ] Slice returns view with same stride

### 4.4 Decimal Arrays
- [ ] `Decimal128Array`, `Decimal256Array`
- [ ] Stored as primitive i128/i256
- [ ] Precision and scale metadata

---

## Layer 5: Variable-Length Arrays

### 5.1 String & Binary Arrays
- [ ] `GenericByteArray<OffsetSize>` abstraction
- [ ] Offset buffer (i32 or i64) + values buffer
- [ ] Type aliases: `StringArray`, `LargeStringArray`, `BinaryArray`
- [ ] UTF-8 validation for strings

**Physical Layout**:
```
Offsets: [0, 5, 5, 12]  (boundaries into values)
Values:  [h,e,l,l,o,w,o,r,l,d,!]
Logical: ["hello", "", "world!"]
```

**Challenge**: Implement `value(i)` to extract substring

### 5.2 ByteView Arrays (Modern Alternative)
- [ ] `GenericByteViewArray<T>`
- [ ] View structure (u128): length + data/pointer
- [ ] Inline short strings (≤12 bytes)
- [ ] Long strings: 4-byte prefix + buffer_index + offset
- [ ] Non-contiguous storage enables parallel appends

**When to use ByteView vs Offset**:
- ByteView: Better for comparisons, parallel construction, mixed lengths
- Offset: Simpler, better for sequential access

### 5.3 List Arrays
- [ ] `GenericListArray<OffsetSize>`
- [ ] Offset buffer + child `ArrayRef`
- [ ] Child can be any array type (recursively nested)
- [ ] Null handling at multiple levels

**Example**:
```rust
// [[1, 2], null, [3]]
ListArray {
    offsets: [0, 2, 2, 3],
    values: Int32Array([1, 2, 3]),
    nulls: [true, false, true],
}
```

### 5.4 FixedSizeList Arrays
- [ ] Fixed number of elements per list
- [ ] No offset buffer needed
- [ ] Stride-based indexing

---

## Layer 6: Nested & Complex Types

### 6.1 StructArray
- [ ] Multiple child arrays (one per field)
- [ ] All children have same length
- [ ] Null bitmap indicates entire struct is null
- [ ] Like a mini-RecordBatch

**Example**:
```rust
// Struct { x: i32, y: f64 }
StructArray {
    fields: [Field("x", Int32), Field("y", Float64)],
    values: [Int32Array([1, 2, 3]), Float64Array([1.0, 2.0, 3.0])],
}
```

### 6.2 Union Arrays
- [ ] Dense vs Sparse modes
- [ ] Type ID buffer indicates active field
- [ ] Dense: offset buffer into values
- [ ] Sparse: all fields same length

**Use Case**: Heterogeneous data (JSON-like)

### 6.3 Dictionary Arrays
- [ ] Keys array (indices) + values array (dictionary)
- [ ] Compression for repeated values
- [ ] Like enum/categorical encoding

**Example**:
```rust
// ["foo", "bar", "foo", "baz"] with dictionary
DictionaryArray {
    keys: Int8Array([0, 1, 0, 2]),
    values: StringArray(["foo", "bar", "baz"]),
}
```

### 6.4 RunEndEncoded Arrays
- [ ] Run-end indices + values
- [ ] Compression for repeated values
- [ ] Logical length != physical length

**Example**:
```rust
// [A, A, A, B, B, C]
RunArray {
    run_ends: [3, 5, 6],
    values: [A, B, C],
}
```

---

## Layer 7: The Array Trait & Type Erasure

### 7.1 Array Trait Design
- [ ] Common interface for all array types
- [ ] `as_any()` for downcasting
- [ ] `data_type()`, `len()`, `null_count()`
- [ ] `slice()` for zero-copy subsets
- [ ] Send + Sync for thread safety

### 7.2 ArrayRef & Type Erasure
- [ ] `type ArrayRef = Arc<dyn Array>`
- [ ] Enables `Vec<ArrayRef>` for heterogeneous columns
- [ ] `downcast_ref::<ConcreteArray>()` for typed access

**Pattern**:
```rust
fn process_column(arr: &ArrayRef) {
    match arr.data_type() {
        DataType::Int32 => {
            let typed = arr.as_any().downcast_ref::<Int32Array>().unwrap();
            // work with typed array
        }
        _ => todo!(),
    }
}
```

### 7.3 ArrayAccessor Trait
- [ ] Generic iteration over arrays
- [ ] `type Item` associated type
- [ ] Enables writing generic algorithms

---

## Layer 8: RecordBatch & Tabular Data

### 8.1 RecordBatch Structure
- [ ] Schema + Vec<ArrayRef> columns
- [ ] All columns must have same length
- [ ] Columnar storage (vs row-based)

### 8.2 RecordBatch Operations
- [ ] Slicing
- [ ] Column projection (select subset of columns)
- [ ] Schema validation

**Unit of Processing**: Most I/O operations work with RecordBatch

---

## Layer 9: Builder Pattern

### 9.1 Primitive Builders
- [ ] `PrimitiveBuilder<T>`
- [ ] `append_value()`, `append_null()`, `append_slice()`
- [ ] `finish()` consumes builder, returns array
- [ ] Capacity management and resizing

### 9.2 Variable-Length Builders
- [ ] `GenericByteBuilder<T>` for strings/binary
- [ ] Two internal buffers: offsets + values
- [ ] `append_value()` updates both
- [ ] UTF-8 validation on append

### 9.3 Nested Builders
- [ ] `GenericListBuilder<O, T>` for lists
- [ ] `values()` returns inner builder
- [ ] `append(true/false)` marks list boundaries
- [ ] Recursive building pattern

### 9.4 Struct & Union Builders
- [ ] Field builders managed separately
- [ ] Coordinate appends across fields
- [ ] Complexity grows with nesting

---

## Layer 10: Nullability Deep Dive

### 10.1 Three-Level Null Architecture
- [ ] Physical: `NullBuffer` bitmask
- [ ] Logical: Optional nulls on each array type
- [ ] Operational: Null propagation in compute

### 10.2 Null Semantics by Type
- [ ] Primitive: Direct validity bitmap
- [ ] String: Null at array level, not in values buffer
- [ ] List: Parent null vs child nulls (independent)
- [ ] Struct: Null struct vs null field values
- [ ] Dictionary: Nulls in keys, not values

### 10.3 Null Buffer Operations
- [ ] `union()`: Combine null masks
- [ ] Null propagation in binary operations
- [ ] Efficient null checking with cached count

**Key Insight**: Zero overhead for non-null arrays (Optional)

---

## Layer 11: Compute Kernels

### 11.1 Arithmetic Operations
- [ ] Element-wise add, subtract, multiply, divide
- [ ] Type-safe via generics
- [ ] Null propagation (null + any = null)
- [ ] Overflow handling strategies

### 11.2 Comparison Operations
- [ ] equal, not_equal, less_than, greater_than
- [ ] Returns BooleanArray
- [ ] Null handling (null == null → null)

### 11.3 Cast Operations
- [ ] Type conversion with validation
- [ ] Lossy vs lossless casts
- [ ] Cast matrix (which types can convert)

### 11.4 Selection Kernels
- [ ] `filter`: Select rows where boolean mask is true
- [ ] `take`: Gather elements by indices
- [ ] `concat`: Combine arrays end-to-end
- [ ] Handling nulls during selection

### 11.5 Aggregate Operations
- [ ] min, max, sum, mean
- [ ] Null-skipping behavior
- [ ] Efficient SIMD implementations

### 11.6 String Operations
- [ ] length, substring, concatenate
- [ ] Case conversion
- [ ] Regular expression matching

---

## Layer 12: Ordering & Sorting

### 12.1 Comparison Infrastructure
- [ ] `SortOptions`: ASC/DESC, NULLS FIRST/LAST
- [ ] Total ordering with null handling
- [ ] Multi-column comparison

### 12.2 Sort Algorithms
- [ ] `sort_to_indices`: Generate permutation
- [ ] Apply indices to reorder array
- [ ] Stable vs unstable sorts

### 12.3 Row Format for Comparison
- [ ] Comparable byte representation
- [ ] Multi-column sorting without type dispatch
- [ ] Efficient comparison in sort algorithms

---

## Layer 13: I/O & Serialization

### 13.1 Arrow IPC Format
- [ ] Schema serialization (Flatbuffers)
- [ ] RecordBatch serialization
- [ ] File format: schema + batches
- [ ] Stream format: sequential batches

### 13.2 CSV I/O
- [ ] Schema inference from CSV
- [ ] Streaming reader for large files
- [ ] Type parsing and validation

### 13.3 JSON I/O
- [ ] JSON to Arrow conversion
- [ ] Nested structure mapping
- [ ] Streaming reader/writer

### 13.4 Parquet Integration
- [ ] Columnar storage format
- [ ] Compression codecs
- [ ] Predicate pushdown
- [ ] Column pruning

---

## Layer 14: Advanced Topics

### 14.1 FFI (Foreign Function Interface)
- [ ] C Data Interface for Arrow
- [ ] Zero-copy sharing across languages
- [ ] Schema/array export/import

### 14.2 Flight Protocol
- [ ] gRPC-based data transfer
- [ ] Streaming large datasets
- [ ] DoGet, DoPut, DoExchange

### 14.3 Compute Expression Engine
- [ ] Expression trees
- [ ] Optimization passes
- [ ] Vectorized execution

---

## Design Patterns to Understand

### Pattern 1: Generic Programming with Zero-Cost
- [ ] Trait-based generics with monomorphization
- [ ] Static dispatch for performance
- [ ] No runtime type checking overhead

### Pattern 2: Type Erasure for Flexibility
- [ ] `dyn Array` for heterogeneous collections
- [ ] Downcast when specific type needed
- [ ] Balance between static safety and dynamic flexibility

### Pattern 3: Builder Pattern for Complexity
- [ ] Mutable accumulation phase
- [ ] Immutable usage phase
- [ ] Resource cleanup on `finish()`

### Pattern 4: Reference Counting for Sharing
- [ ] `Arc<T>` for shared ownership
- [ ] Zero-copy operations via clone
- [ ] Automatic cleanup when last reference dropped

### Pattern 5: Unsafe with Safe Boundaries
- [ ] Unsafe implementation details
- [ ] Safe public API with validation
- [ ] Invariant maintenance

### Pattern 6: Buffer Layout Optimization
- [ ] Contiguous memory for cache efficiency
- [ ] Bit-packing for space savings
- [ ] Alignment for SIMD vectorization
- [ ] Pointer storage to avoid arithmetic in hot loops

---

## Key Design Questions to Answer

1. **Why pointer instead of offset in Buffer?**
   - LLVM vectorization optimization
   - Avoids pointer arithmetic in hot loops

2. **Why Arc<BufferInner> instead of Box?**
   - Enables zero-copy slicing
   - Multiple arrays can share same buffer

3. **Why separate NullBuffer from values?**
   - Optional: zero overhead for non-null arrays
   - Separate bit-packing: 8x space savings

4. **Why OffsetBuffer instead of Vec<Vec<T>>?**
   - Contiguous memory layout
   - Better cache locality
   - Single allocation vs many

5. **Why ByteView for strings?**
   - Faster comparisons (prefix matching)
   - Non-contiguous storage (parallel appends)
   - Inline short strings (cache efficiency)

6. **Why ArrayData vs just Array trait?**
   - Low-level representation for FFI
   - Validation layer
   - Type-erased but structured

7. **Why RunEndEncoded?**
   - Native compression for repeated values
   - Query engines can work directly on compressed data
   - Avoid decompress/recompress cycles

---

## Implementation Roadmap

### Phase 1a: Buffer Foundation ✅
- [x] BufferInner with custom allocation
- [x] Buffer with Arc-based sharing
- [x] Zero-copy slicing
- [x] NullBuffer basics

### Phase 1b: Type System for Buffers (Current)
- [ ] ArrowNativeType trait (sealed, for primitives)
- [ ] ScalarBuffer<T> (generic typed buffer)
- [ ] BooleanBuffer (bit-packed, separate from NullBuffer)
- [ ] Bit utilities (get_bit, set_bit, iterators)

### Phase 1c: Mutable Buffers & Builders
- [ ] MutableBuffer (growable)
- [ ] BufferBuilder<T>
- [ ] BooleanBufferBuilder
- [ ] NullBufferBuilder

### Phase 1d: Specialized Buffers
- [ ] OffsetBuffer<T> (for strings/lists)
- [ ] OffsetBufferBuilder
- [ ] RunEndBuffer<T> (for RLE)
- [ ] Buffer bitwise operations (AND, OR, NOT)

### Phase 2: Type System & Schema
- [ ] DataType enum (primitives first)
- [ ] Field and Schema basics
- [ ] ArrowPrimitiveType trait (connects Rust types to DataType)

### Phase 3: Arrays
- [ ] PrimitiveArray<T> implementation
- [ ] BooleanArray
- [ ] Int32Array, Float64Array type aliases
- [ ] Array trait and ArrayRef

### Phase 4: Variable-Length Arrays
- [ ] GenericByteArray for strings
- [ ] StringArray, BinaryArray
- [ ] GenericListArray
- [ ] ArrayData structure

### Phase 5: Advanced Arrays
- [ ] StructArray
- [ ] DictionaryArray
- [ ] ByteView arrays
- [ ] RunEndEncoded arrays

### Phase 6: Operations
- [ ] Basic compute kernels (arithmetic)
- [ ] Filter and Take
- [ ] RecordBatch

### Phase 7: I/O
- [ ] Arrow IPC format
- [ ] CSV reader

---

## Learning Resources

- **Arrow Specification**: https://arrow.apache.org/docs/format/Columnar.html
- **arrow-rs Repository**: `~/Code/arrow-rs`
- **Exploration Notes**: `~/Documents/kanatti-notes/arrow/exploration.md`

---

## Usage Patterns

### Pattern: Read → Understand → Implement
```
1. Pick a concept from roadmap
2. Read arrow-rs implementation
3. Ask: "Why this design?"
4. Implement simplified version
5. Test with edge cases
6. Document learnings
```

### Pattern: Problem-Based Learning
```
1. Define a scenario (e.g., "store nullable integers")
2. Implement naive solution
3. Identify problems (alignment, nulls, copying)
4. Refine design
5. Compare with arrow-rs approach
```

**Important: Help user naturally understand concepts by asking questions and guiding simplified implementation without giving he solution directly.**

---

## Current Status

**Layer 1: Memory Management & Buffers** - In Progress

### ✅ Completed
- **BufferInner** (1.1): Custom allocation with Layout, NonNull, alignment, deallocation, from_raw_parts
- **Buffer** (1.2): Arc-based sharing, zero-copy slicing, clone without copy, From<Vec<T>>
- **NativeType** (1.4): Sealed trait, get_byte_width(), get_alignment(), implemented for all primitives
- **ScalarBuffer<T>** (1.5): Type-safe wrapper, Deref to &[T], slice(), From<Vec<T>>, Clone
- **BooleanBuffer** (1.6): Bit-packed boolean storage, value(), from_bools(), len()
- **NullBuffer** (1.7): Wraps BooleanBuffer, cached null_count, is_null, from_bools
- **Bit Utilities** (1.11): `get_bit()`, `pack_bools()` in `bit_util.rs`

### 📋 Not Yet Implemented
| Section | Items |
|---------|-------|
| 1.2 Buffer | `is_empty()`, `ptr_eq()`, `shrink_to_fit()` |
| 1.3 MutableBuffer | Entire section |
| 1.4 NativeType | `from_usize()`, `to_usize()` conversions |
| 1.6 BooleanBuffer | `slice()`, `count_set_bits()`, iterators |
| 1.7 NullBuffer | `union()`, `intersect()`, `contains_nulls()` |
| 1.8 OffsetBuffer | Entire section |
| 1.9 RunEndBuffer | Entire section |
| 1.10 Buffer Ops | AND, OR, XOR, NOT |
| 1.11 Bit Utilities | `set_bit()`, iterators |
| 1.12 Builders | All builders |
| 1.13 Allocation | ALIGNMENT constant, Deallocation enum |

### Recommended Next Steps
1. **MutableBuffer** (1.3) - Needed for builders
2. **Builders** (1.12) - Construction patterns
3. **Layer 2: Type System** - DataType, Schema, ArrowPrimitiveType

**Test Status**: 26/26 tests passing ✅

---

*This is a living document. Update as you progress through concepts.*
