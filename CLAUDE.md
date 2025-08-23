# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a defense simulation project (`defsim`) that implements an agent-based simulation system written in Rust. The simulation models defensive scenarios with targets, command posts, sensors, launchers, and missiles using time-driven simulation with fixed Δt timesteps.

## Communication Language

**IMPORTANT: Always respond in Japanese (日本語) when working in this repository.** All explanations, error messages, and communication should be in Japanese to match the project's primary language and documentation.

## Git Workflow

**IMPORTANT: Always follow this Git workflow for any task:**

1. **Switch back to the main branch**: Always return to the main branch before starting work.
   ```bash
   git checkout main
   ````

2. **Pull the latest state**: Retrieve the latest state of the main branch.

   ```bash
   git pull origin main
   ```

3. **Create a branch**: Always create a new branch when starting a task.

   ```bash
   git checkout -b feature/task-description
   # or
   git checkout -b fix/issue-description
   ```

4. **Do your work**: Make the necessary changes on the branch.

5. **Commit your changes**: After completing your work, commit the changes.

   ```bash
   git add .
   git commit -m "Appropriate commit message"
   ```

6. **Push to origin**: After completing your work, always push to origin.

   ```bash
   git push -u origin branch-name
   ```

## Development Environment
We will develop using the Windows Command Prompt.
Therefore, CLI commands will follow Windows path conventions.

## Development Commands

Since this project is implemented in Rust, use the following standard Rust commands:

- **Build**: `cargo build`
- **Run**: `cargo run`
- **Test**: `cargo test`
- **Check**: `cargo check`
- **Format**: `cargo fmt`
- **Lint**: `cargo clippy`
- **Release build**: `cargo build --release`
- **Documentation**: `cargo doc --open`

## Architecture Overview

### Core Agent Types

The simulation is built using an agent-based architecture with the following entity types:

1. **Target (敵)**: Enemy entities that move from spawn points toward command post
2. **CommandPost (指揮所)**: Single command center that coordinates missile launches
3. **Sensor (センサ)**: Detection systems that identify targets within range
4. **Launcher (ランチャ)**: Platforms that fire missiles at assigned targets
5. **Missile (ミサイル)**: Guided projectiles using proportional navigation

### Interface Design

The system uses a facade pattern with common interfaces:

- `IAgent`: Base interface for all simulation entities (initialize, tick)
- `IMovable`: For entities that move (Target, Missile)
- `ISensor`: For detection capabilities (Sensor)
- `IPlatform`: For launch platforms (Launcher)
- `IMissile`: For missile-specific behavior (Missile)
- `ICollision`: For collision detection (Missile)
- `IAllocator`: For target assignment (CommandPost)

### Simulation Processing Order

Each simulation tick follows this sequence:
1. Target processing (movement, status updates)
2. Missile processing (guidance, movement, collision detection)
3. Sensor processing (target detection)
4. Command post processing (target prioritization, missile assignment)
5. Launcher processing (missile firing, cooldown management)

### Configuration System

The simulation uses YAML configuration files with two types of settings:
- **Scenario values**: Individual agent configurations (positions, counts, timing)
- **Performance values**: Common behavioral parameters for agent types

## Key Implementation Details

### Coordinate System
- 3D coordinate system: X (right), Y (up), Z (altitude)
- All distances in meters (m), no kilometers
- Angles: 0° = +X direction, positive angles = counter-clockwise from +Z view
- Simulation region: ±1,000,000m square centered on origin
- Altitude range: 0-5,000m (values clamped to this range)

### Missile Guidance
- Uses True 3D Proportional Navigation (PN) with N=3-4
- Update sequence per tick: guidance calculation → acceleration saturation → velocity integration → speed clamping → position update → attitude update
- Endgame conditions: miss distance increases for specified ticks when within 2× intercept radius

### Target Prioritization
- Primary: Time-to-go (Tgo) = max(0, (||r_xy|| - arrival_radius_m) / v_target)
- Tiebreakers: XY distance → ID ascending
- Multiple missiles can be assigned to high-endurance targets

### Formation Patterns
- Enemy groups spawn in concentric rings with equal angular spacing
- Ring radius = k × spacing_radius (k=1,2,...)
- Optional half-angle offset for outer rings

## Development Guidelines

- Maintain deterministic simulation behavior (avoid random numbers unless seeded)
- Use consistent unit system (meters, seconds, degrees for external interfaces)
- Implement proper boundary checking for simulation region
- Ensure thread-safe design for potential parallel processing
- Follow Rust best practices for memory safety and performance

## Testing Strategy

When implementing tests:
- Unit tests for individual agent behaviors
- Integration tests for multi-agent interactions
- Scenario validation tests using sample YAML configurations
- Performance benchmarks for different entity counts

## File Structure Expectations

When implementing the codebase:
- `src/agents/` - Individual agent implementations
- `src/interfaces/` - Trait definitions
- `src/simulation/` - Main simulation engine
- `src/config/` - Configuration loading and validation
- `src/math/` - Mathematical utilities (guidance, geometry)
- `scenarios/` - Sample YAML configuration files
- `tests/` - Test suites and benchmark scenarios