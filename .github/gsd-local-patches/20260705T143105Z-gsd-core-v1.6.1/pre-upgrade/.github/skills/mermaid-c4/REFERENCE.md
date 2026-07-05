# Mermaid C4 Reference

Use this file when the user explicitly wants C4 views or when a Mermaid answer
should follow the C4 model rather than generic diagram selection.

## Official C4 Hierarchy

The C4 model is abstraction-first.

- People use software systems.
- A software system is made of one or more containers.
- A container contains one or more components.
- Components are implemented by code elements such as classes, interfaces,
  objects, files, or functions.

The core static structure diagrams are:

1. System context diagram
2. Container diagram
3. Component diagram
4. Code diagram

The supporting diagrams are:

1. System landscape diagram
2. Dynamic diagram
3. Deployment diagram

You do not need to use all four static levels every time. System context and
container are often sufficient unless the prompt needs a deeper zoom level.

## C4 View Selection Matrix

| User signal | C4 view | Mermaid form | Add when needed | Why |
| --- | --- | --- | --- | --- |
| users, business actors, system of interest, external systems | system context | `flowchart LR` | `dynamic`, `container` | Shows people and neighboring systems around the target system |
| multiple peer systems, portfolio map, enterprise landscape | system landscape | `flowchart LR` | `dynamic` | Shows relationships between software systems at a broad level |
| services, web apps, APIs, workers, databases, queues inside one system | container | `flowchart LR` or `architecture-beta` | `dynamic`, `deployment` | Shows deployable applications and data stores |
| modules, adapters, handlers, stores, ports inside one container | component | `flowchart LR` | `dynamic`, `code` | Shows internal building blocks inside a single container |
| classes, traits, interfaces, packages, files, functions | code | `classDiagram` or `graph LR` | none | Shows code-level structure only when that level adds value |
| runtime scenario, request path, handshake, end-to-end flow | dynamic | `sequenceDiagram` | pair with the matching static view | Shows how elements collaborate over time |
| node, cluster, region, environment, VM, pod, runtime placement | deployment | `architecture-beta` or `flowchart LR` | `dynamic` | Shows where containers run |

## C4 Bundle Recipes

### Onboarding Pack

Use when the user wants a broad architectural explanation first.

1. System context
2. Container
3. Dynamic only if one scenario matters

### Service Deep Dive Pack

Use when one service or application is the real target.

1. Container
2. Component
3. Dynamic at container or component level

### Portfolio Pack

Use when multiple systems must be positioned together.

1. System landscape
2. System context for the system of interest

### Deployment Pack

Use when runtime placement matters.

1. Container
2. Deployment
3. Dynamic only when traffic or failover path matters

### Code Pack

Use when architecture discussion has already narrowed to one container and the
user needs code-level shape.

1. Component
2. Code

## C4 Modeling Rules

- Keep one primary abstraction level per static diagram.
- Do not place component-level internals directly on a system context diagram.
- Do not let runtime nodes masquerade as containers; deployment is a separate
  concern.
- Label each element with a short responsibility.
- Include technology tags for containers when known.
- Write relationship labels as short verb phrases.
- Distinguish internal versus external elements clearly.
- Use assumptions sparingly and label them when the prompt is incomplete.

## Shared Semantic Palette

This palette is intentionally identical to `mermaid-spectrum` so both skills
stay visually compatible.

| Role | Fill | Stroke | Text |
| --- | --- | --- | --- |
| Public API / User | `#E3F2FD` | `#1E88E5` | `#0D47A1` |
| Domain logic | `#F3E5F5` | `#8E24AA` | `#4A148C` |
| Infrastructure / Runtime | `#FFF3E0` | `#FB8C00` | `#E65100` |
| External / Cross-crate | `#E8F5E9` | `#43A047` | `#1B5E20` |
| Danger / Failure / Attack | `#FFE0E0` | `#D32F2F` | `#B71C1C` |
| Neutral / Support | `#ECEFF1` | `#546E7A` | `#263238` |
| Crypto / Proof | `#EDE7F6` | `#5E35B1` | `#311B92` |
| Storage / DA layer | `#FFE0B2` | `#F57C00` | `inherit` |

Validation and test nodes should reuse the same green family as the external
role: `fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20`.

## C4 Role Mapping To The Shared Palette

| C4 element | Preferred palette role | Notes |
| --- | --- | --- |
| Person / actor | Public API / User | Use blue for humans or user-facing actors |
| System of interest | Domain logic | Purple by default for the central system |
| Internal application container | Domain logic or Infrastructure / Runtime | Pick based on whether it is business logic or platform runtime |
| Database, queue, event store, object store | Storage / DA layer | Use the storage palette consistently |
| External software system or external container | External / Cross-crate | Green distinguishes outside boundaries |
| Deployment node or environment | Infrastructure / Runtime | Orange for clusters, nodes, regions, VMs, or runtimes |
| Security gate or failure path | Danger / Failure / Attack | Red only for failure, risk, or blocked paths |
| Crypto-heavy subsystem | Crypto / Proof | Use when privacy or proof logic is central |

## Mermaid Patterns

### System Context

Use one system boundary and keep internals out of the diagram.

```mermaid
flowchart LR
  Customer[Customer\nUses the platform]
  Support[Support Agent\nHandles exceptions]
  Billing[Billing Platform\nExternal billing system]
  Target[Payment Platform\nProcesses payments and refunds]

  Customer -->|submits payment| Target
  Support -->|reviews disputes| Target
  Target -->|charges cards| Billing

  style Customer fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style Support fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style Billing fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style Target fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
```

### Container Diagram

Use a software-system boundary and show deployable applications plus data stores.

```mermaid
flowchart LR
  User[Customer]
  Ext[Email Provider\nExternal system]

  subgraph System[Notification Platform]
    Api[API Service\nRust HTTP API]
    Worker[Delivery Worker\nAsync job processor]
    Db[(Notification DB\nPostgreSQL)]
    Queue[(Job Queue\nRedis Streams)]
  end

  User -->|creates notification| Api
  Api -->|stores request| Db
  Api -->|enqueues job| Queue
  Worker -->|reads jobs| Queue
  Worker -->|loads templates| Db
  Worker -->|sends message| Ext

  style User fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style Ext fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
  style Api fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Worker fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Db fill:#FFE0B2,stroke:#F57C00,stroke-width:1px
  style Queue fill:#FFE0B2,stroke:#F57C00,stroke-width:1px
```

### Component Diagram

Keep the scope inside one container and show only major building blocks.

```mermaid
flowchart LR
  subgraph Container[API Service]
    Http[HTTP Controller\nAccepts requests]
    App[Notification Service\nCoordinates workflow]
    Policy[Policy Guard\nValidates channel rules]
    Repo[Template Repository\nLoads templates]
  end

  Queue[(Job Queue)]
  Db[(Notification DB)]

  Http -->|calls| App
  App -->|checks rules| Policy
  App -->|loads template| Repo
  App -->|persists request| Db
  App -->|publishes job| Queue

  style Http fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style App fill:#F3E5F5,stroke:#8E24AA,stroke-width:1px,color:#4A148C
  style Policy fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  style Repo fill:#ECEFF1,stroke:#546E7A,stroke-width:1px,color:#263238
  style Db fill:#FFE0B2,stroke:#F57C00,stroke-width:1px
  style Queue fill:#FFE0B2,stroke:#F57C00,stroke-width:1px
```

### Dynamic Diagram

Keep the participants at the same level as the paired static diagram.

```mermaid
sequenceDiagram
  box rgb(227,242,253) Actor
    participant User
  end
  box rgb(243,229,245) System Containers
    participant Api as API Service
    participant Worker as Delivery Worker
  end
  box rgb(255,224,178) Storage
    participant Db as Notification DB
    participant Queue as Job Queue
  end
  box rgb(232,245,233) External
    participant Mail as Email Provider
  end

  User->>Api: POST /notifications
  Api->>Db: store request
  Api->>Queue: enqueue delivery job
  Worker->>Queue: claim job
  Worker->>Db: load template
  Worker->>Mail: send email
  Mail-->>Worker: accepted
  Worker-->>Api: delivery status
  Api-->>User: request accepted
```

### Deployment Diagram

Prefer `architecture-beta` when it is clear and supported. Fall back to a
styled `flowchart LR` with environment subgraphs when styling or rendering is
more reliable there.

```mermaid
flowchart LR
  User[Customer]

  subgraph Region[Cloud Region eu-west-1]
    subgraph Cluster[Kubernetes Cluster]
      ApiPod[API Pod]
      WorkerPod[Worker Pod]
    end
    Redis[(Redis)]
    Pg[(PostgreSQL)]
  end

  Mail[Email Provider]

  User -->|HTTPS| ApiPod
  ApiPod -->|enqueue| Redis
  WorkerPod -->|dequeue| Redis
  ApiPod -->|read/write| Pg
  WorkerPod -->|read| Pg
  WorkerPod -->|SMTP API| Mail

  style User fill:#E3F2FD,stroke:#1E88E5,stroke-width:1px,color:#0D47A1
  style ApiPod fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style WorkerPod fill:#FFF3E0,stroke:#FB8C00,stroke-width:1px,color:#E65100
  style Redis fill:#FFE0B2,stroke:#F57C00,stroke-width:1px
  style Pg fill:#FFE0B2,stroke:#F57C00,stroke-width:1px
  style Mail fill:#E8F5E9,stroke:#43A047,stroke-width:1px,color:#1B5E20
```

## Output Heuristics

### Prefer a single view when

- one C4 level fully answers the ask
- the user explicitly asks for one named level
- additional views would restate the same truth

### Prefer a paired set when

- a static C4 level and one scenario both matter
- a deployment explanation needs the container view first
- a component explanation needs one dynamic path to make the collaboration clear

### Prefer a compact pack when

- the user needs broad orientation plus one deeper zoom level
- the prompt spans architecture boundaries and runtime behavior
- omitting a supporting view would make the explanation misleading

## Fast Mapping Rules

- `who uses the system` -> system context
- `which systems exist around it` -> system landscape
- `which apps and data stores exist inside it` -> container
- `which building blocks exist inside one app` -> component
- `which classes or files implement it` -> code
- `how one scenario flows` -> dynamic
- `where it runs` -> deployment
