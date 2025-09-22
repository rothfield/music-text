# Spec-Kit Context

## Overview
Spec-Kit is GitHub's open-source toolkit for Spec-Driven Development (SDD), enabling structured development where specifications define intent before implementation.

## Installation & Usage
```bash
# Run spec-kit commands
uvx --from git+https://github.com/github/spec-kit.git specify <command>

# Check tools
uvx --from git+https://github.com/github/spec-kit.git specify check
```

## Core Commands
- `/constitution` - Establish project principles (quality, testing, performance)
- `/specify` - Define requirements and "what" to build
- `/plan` - Generate technical implementation approach
- `/tasks` - Break down into actionable items
- `/implement` - Execute implementation with AI

## Workflow
1. **Constitution** → Set non-negotiable principles
2. **Specify** → Write intent-focused specifications
3. **Plan** → AI creates technical roadmap
4. **Tasks** → Generate implementable task list
5. **Implement** → AI executes based on specs

## Key Principles
- Specifications are source of truth, not code
- Focus on "what" and "why", not "how"
- Multi-step refinement over one-shot generation
- Intent-driven development with AI execution

## Project Status
- Initialized in this project
- 30+ specification files in `/specs/` directory
- Ready for spec-driven development workflow
- Compatible with Claude Code as AI agent

## Benefits
- Traceable path from requirements to implementation
- Consistent, maintainable code generation
- Iterative development without expensive rewrites
- Quality guardrails through constitutional principles