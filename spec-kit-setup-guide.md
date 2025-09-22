# Spec-Kit Setup Guide

## What Was Added

After deleting `~/.claude` files, we set up spec-kit which added:

### New Directories
- `.claude/commands/` - Contains 5 slash command definitions
- `.specify/` - Spec-kit templates and scripts
- `spec-kit/` - Full spec-kit repository

### New Files
- `specs/constitution.md` - Project constitution
- `spec-kit-context.md` - Context documentation

## Available Slash Commands

Now working in Claude Code:
- `/constitution` - Create/update project principles
- `/specify` - Define feature requirements
- `/plan` - Create implementation plan
- `/tasks` - Generate task breakdown
- `/implement` - Execute implementation

## How It Works

1. Each command in `.claude/commands/` defines a workflow
2. Commands call bash scripts in `.specify/scripts/bash/`
3. Scripts create feature branches and spec files in `specs/`
4. Follow the spec-driven development process

## Quick Test

Try: `/constitution Create basic development principles`