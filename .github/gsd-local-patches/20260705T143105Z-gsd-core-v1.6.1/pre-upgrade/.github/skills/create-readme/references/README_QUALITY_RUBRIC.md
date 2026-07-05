# README Quality Rubric

Use this rubric when you need a stricter review than the base checklist provides.

## Completeness Scorecard

```mermaid
pie
    title README Completeness
    "Essential Sections" : 45
    "Code Examples" : 20
    "Visual Elements" : 15
    "Contribution Guidance" : 10
    "Additional Resources" : 10
```

## Quality Rating Map

```mermaid
graph LR
    A[README Quality] --> B[Essential Sections]
    A --> C[Code Examples]
    A --> D[Visual Elements]
    A --> E[Contribution Guidance]

    B --> B1[Description and Value]
    B --> B2[Installation and Setup]
    B --> B3[Usage and Configuration]

    C --> C1[Working Minimal Example]
    C --> C2[Copy Paste Safety]

    D --> D1[Badges]
    D --> D2[Screenshots or Diagrams]

    E --> E1[Contribution Path]
    E --> E2[Support or Help Path]
```

## How To Use This Rubric

- use it for `review` mode when the user wants a deeper quality judgment
- treat low scores in essential sections as blockers
- treat low scores in visual elements as cleanup unless trust is harmed
- do not invent missing signals just to satisfy the rubric
