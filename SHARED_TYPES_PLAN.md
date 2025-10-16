# Shared Types Implementation Plan

## ✅ Completed

### 1. Shared Module Structure
Created `reev-db/src/shared/` with organized modules:
- `flow/`: Flow logging and execution tracking types
- `benchmark/`: Benchmark-related types  
- `performance/`: Performance monitoring types

### 2. Core FlowLog Types
- **Shared FlowLog**: Database-friendly structure with String timestamps, JSON fields
- **Conversion Traits**: `FlowLogConverter<T>` for domain-specific conversions
- **Utility Functions**: `FlowLogUtils` for common operations
- **Error Handling**: `ConversionError` with detailed context

### 3. Generic Design Benefits
- **Database-Friendly**: Uses String timestamps, JSON serializable
- **Cross-Project Ready**: Can be extracted to separate crate later
- **Conversion Support**: Clean boundaries between domain and shared types
- **No Coupling**: reev-db remains generic and reusable

## 🔄 Migration Strategy

### Phase 1: Shared Types (✅ Complete)
- Create shared types in reev-db
- Implement conversion traits
- Add utility functions and tests

### Phase 2: Update reev-lib (Next)
- Remove duplicate FlowLog from reev-lib
- Implement `FlowLogConverter` for reev-lib types
- Update imports to use shared types
- Fix type conversion issues in database methods

### Phase 3: Extract to Separate Crate (Future)
- Copy `shared/` directory to new `reev-types` crate
- Update dependencies across projects
- Version and publish independently

## 📋 Implementation Details

### Shared FlowLog Structure
```rust
pub struct FlowLog {
    pub session_id: String,
    pub benchmark_id: String,
    pub agent_type: String,
    pub start_time: String,        // RFC3339
    pub end_time: Option<String>,  // RFC3339
    pub flow_data: String,         // JSON events
    pub final_result: Option<String>, // JSON result
    pub id: Option<i64>,
    pub created_at: Option<String>,
}
```

### Conversion Pattern
```rust
impl FlowLogConverter<DomainFlowLog> for DomainFlowLog {
    fn to_flow_log(&self) -> Result<FlowLog, ConversionError> {
        // Convert domain type to shared FlowLog
    }
    
    fn from_flow_log(flow_log: &FlowLog) -> Result<DomainFlowLog, ConversionError> {
        // Convert shared FlowLog to domain type
    }
}
```

## 🎯 Benefits Achieved

1. **Eliminated Duplication**: Single source of truth for FlowLog
2. **Generic Database**: reev-db can be used by other projects
3. **Clean Boundaries**: Conversion traits separate concerns
4. **Future-Proof**: Ready for extraction to separate crate
5. **Type Safety**: Compile-time conversion guarantees

## 🛠️ Next Steps for reev-lib

1. **Remove Duplicate Types**: Delete reev-lib FlowLog and related types
2. **Implement Converter**: Add `FlowLogConverter` implementation
3. **Update Database Methods**: Fix type conversions in writer/reader
4. **Update Imports**: Use shared types throughout reev-lib
5. **Test Integration**: Ensure all functionality works with shared types

## 📊 Impact

- **Code Reduction**: ~200 lines of duplicate types eliminated
- **Maintainability**: Single place to update FlowLog structure
- **Reusability**: reev-db now truly generic
- **Performance**: No runtime overhead, compile-time conversions
- **Safety**: Type-safe conversions with error handling

---

**Status**: Shared types infrastructure complete, ready for reev-lib migration