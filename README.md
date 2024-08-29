# Query Engine

Hacking on query-engine in rust from scratch. Totally based on DataFusion, infact purpose is to understand datafusion internals better.

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

1. Start with DataFrame based API only for frontend.
    1. Read csv, parquet.
2. Model PlanNodes, Expressions and Tree Traversal for Logical Plan.
3. Analyzer and Optimizer Rules that can tranverse and rewrite the tree.
4. TBD
