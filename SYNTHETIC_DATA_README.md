# Synthetic Sargam Data Generation for OCR Training

## Overview

This project includes functionality to generate synthetic Sargam notation data for training Optical Character Recognition (OCR) models. The goal is to digitize handwritten manuscripts (MSS) of traditional Indian classical music notation.

## Background

Sargam notation (S R G M P D N) is a traditional Indian musical notation system that exists primarily in:
- Handwritten manuscripts 
- Scanned historical documents
- Traditional music teaching materials

Unlike Western notation, there is very little digitized Sargam notation available, making OCR training datasets scarce.

## Current Focus: Loops and Slurs

The synthetic data generation is currently focused on preparing datasets for training OCR models to recognize **loops and slurs** in Sargam notation:

- **Loops** - Curved ornamental markings around notes indicating musical ornaments
- **Slurs** - Curved lines connecting multiple notes to indicate smooth phrasing

These elements are particularly challenging for OCR because:
- Hand-drawn curves vary significantly between scribes
- They often overlap or intersect with note symbols  
- Different manuscript traditions use varying styles
- They carry important musical meaning that must be preserved

## Synthetic Data Generation

### Purpose
Generate training data for OCR models to recognize handwritten Sargam notation with emphasis on ornamental elements:
- Different handwriting styles for loops and slurs
- Various curve shapes and orientations
- Realistic manuscript aging/degradation effects
- Paper texture and ink bleeding variations

### Generated Files
- `synthetic_image_*.json` - Metadata and ground truth labels
- `synthetic_image_*.svg` - Vector graphics of synthetic notation
- Various overlay and processing files

### Integration with Main Parser
The synthetic data generation leverages the core notation parser to:
1. Parse structured Sargam input with ornaments
2. Generate spatial layouts including curves
3. Apply visual transformations
4. Export in formats suitable for ML training

## Usage Context

This synthetic data generation is **orthogonal** to the main CLI/webapp functionality:
- **Main Parser**: Processes digital Sargam text → structured output (LilyPond, etc.)
- **Synthetic Data**: Generates realistic manuscript images for OCR training on loops/slurs

## Files Related to Synthetic Data
- Data generation binary: `src/bin/data_generator.rs`
- Generated samples: `synthetic_image_*.json`, `synthetic_image_*.svg`
- Requirements doc: `SYNTHETIC_DATA_REQUIREMENTS.md`