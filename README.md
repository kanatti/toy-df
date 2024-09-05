# Toy DF

A toy version of Apache DataFusion, written from scratch. Purpose is to understand the internals of DataFusion better.

## Architecture

```
                       ┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓         
╔═════════════╗        ┃ ┌──────┐ ┌────────────────┐  ┃         
║             ║        ┃ │      │ │                │  ┃         
║  Frontend   ║────────┃ │ SQL  │ │    DataFrame   │  ┃         
║             ║        ┃ │      │ │                │  ┃         
╚═════════════╝        ┃ └──────┘ └────────────────┘  ┃         
       │               ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛         
       │                                                        
       ▼                                                        
╔═════════════╗        ┌────────────────────────────────┐       
║             ║        │  1. Tree based                 │       
║ LogicalPlan ║────────│                                │       
║             ║        │  2. Plan Nodes and Expressions │       
╚═════════════╝        └────────────────────────────────┘       
       │                                                        
       │                                                        
       ▼                                                        
╔═════════════╗                                                 
║  Analyze    ║        ┌────────────────────────────────────────┐
║     +       ║────────│ Rules that tranverse tree and rewrites │
║  Optimize   ║        └────────────────────────────────────────┘
╚═════════════╝                                                 
       │                                                        
       │                                                        
       ▼                                                        
╔═════════════╗                                                 
║             ║                                                 
║PhysicalPlan ║                                                 
║             ║                                                 
╚═════════════╝                                                 
       │                                                        
       │                                                        
       ▼                                                        
╔═════════════╗                                                 
║             ║       ┌───────────────────────────────┐         
║  Runtime    ║───────│ Arrow Operator based runtime  │         
║             ║       └───────────────────────────────┘         
╚═════════════╝                                                 
```

## Plan

As a first milestone, we will try to get basic select + filter work end to end.
- [x] Expressions
- [x] DataFrame API
- [x] LogicalPlan
- [x] Execution Plan scaffolding
- [x] TableProvider and InMemory Table
- [ ] CsvTable and ParquetTable
- [ ] Physical Expression
- [ ] Scan execution
- [ ] Projection execution
- [ ] Filter execution
- [ ] Aggregate execution
- [ ] Join Execution
- [ ] Async and Streams
